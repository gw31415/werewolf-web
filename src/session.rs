use std::{
    collections::HashSet,
    time::{Duration, Instant},
};

use actix::prelude::*;
use actix_web_actors::ws;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use werewolf::{
    master::Token,
    state::{Name, State},
};

use crate::master_router::{self, Identifier};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug)]
pub struct WsPlayerSession {
    token: Option<Token>,
    timestamp: Instant,
    addr: Addr<master_router::MasterRouter>,
}

impl WsPlayerSession {
    /// 新規接続を行う
    pub fn new(addr: Addr<master_router::MasterRouter>) -> Self {
        WsPlayerSession {
            token: None,
            timestamp: Instant::now(),
            addr,
        }
    }
    /// ハートビート
    fn start_heartbeat(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.timestamp) > CLIENT_TIMEOUT {
                if let Some(token) = act.token {
                    act.addr.do_send(master_router::Disconnect { token });
                }
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }
}

impl Actor for WsPlayerSession {
    type Context = ws::WebsocketContext<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        self.start_heartbeat(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        if let Some(token) = self.token {
            self.addr.do_send(master_router::Disconnect { token });
        }
        Running::Stop
    }
}

/// WebSocket message handler from Client to Master
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsPlayerSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        use ws::Message::*;
        match msg {
            Ping(msg) => {
                self.timestamp = Instant::now();
                ctx.pong(&msg);
            }
            Pong(_) => {
                self.timestamp = Instant::now();
            }
            Text(text) => {
                if let Ok(req) = serde_json::from_str::<Request>(&text) {
                    match req {
                        Request::Connect(id) => {
                            if self.token.is_some() {
                                // 既に接続済みの場合
                                Handler::handle(
                                    self,
                                    Response::Error(ResponseErr::AlreadyLoggedIn),
                                    ctx,
                                );
                            } else {
                                // 未接続の場合
                                self.addr
                                    .send(master_router::Connect {
                                        id,
                                        addr: ctx.address().recipient(),
                                    })
                                    .into_actor(self)
                                    .then(|res, act, ctx| {
                                        match res {
                                            Ok(Ok(res)) => {
                                                act.token = Some({
                                                    // Vec<u8> を Token ([u8; 32]) に変換する
                                                    unsafe {
                                                        *Box::from_raw(
                                                            Box::into_raw(res) as *mut [u8; 32]
                                                        )
                                                    }
                                                });
                                            }
                                            Ok(Err(err)) => {
                                                let error = Response::Error(err);
                                                let res_str =
                                                    serde_json::to_string(&error).unwrap();
                                                ctx.text(res_str);
                                            }
                                            _ => {
                                                ctx.stop();
                                            }
                                        }
                                        fut::ready(())
                                    })
                                    .wait(ctx);
                            }
                        }
                        Request::Werewolf(_any) => {
                            todo!("Implementation of the action flow for Werewolf.");
                        }
                    }
                } else {
                    Handler::handle(
                        self,
                        Response::Error(ResponseErr::JsonParse(text.to_string())),
                        ctx,
                    );
                }
            }
            Binary(_) => println!("Unexpected binary"),
            Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            Continuation(_) => {
                ctx.stop();
            }
            Nop => (),
        }
    }
}

/// クライアントからのリクエストの構造体
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Request {
    Connect(Identifier),
    Werewolf(werewolf::request::Any),
}

/// クライアントへ送る構造体
#[derive(Serialize, Message)]
#[rtype("()")]
#[serde(rename_all = "camelCase")]
pub enum Response {
    Success(ResponseOk),
    Error(ResponseErr),
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ResponseOk {
    /// 状態の更新
    State(Box<State>),
    /// オンラインのメンバー一覧
    Online(HashSet<Name>),
    /// 部屋にいるメンバー一覧
    Members(HashSet<Name>),
}

#[derive(Serialize, Error, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ResponseErr {
    #[error("Json parse error: {0}")]
    JsonParse(String),
    #[error("You are already logged in.")]
    AlreadyLoggedIn,
    #[error("Error of werewolf game: {0}")]
    Werewolf(#[from] werewolf::Error),
    #[error("Session error: {0}")]
    Session(#[from] werewolf::master::Error),
}

/// Message handler from Master to Client
impl Handler<Response> for WsPlayerSession {
    type Result = ();
    fn handle(&mut self, msg: Response, ctx: &mut Self::Context) -> Self::Result {
        let res_str = serde_json::to_string(&msg).unwrap();
        ctx.text(res_str);
    }
}
