use std::time::{Duration, Instant};

use actix::prelude::*;
use actix_web_actors::ws;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use werewolf::master::Token;

use crate::master_router::{self, Identifier, SessionError};

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

        match msg {
            ws::Message::Ping(msg) => {
                self.timestamp = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.timestamp = Instant::now();
            }
            ws::Message::Text(text) => {
                if let Ok(req) = serde_json::from_str::<Request>(&text) {
                    match req {
                        Request::Connect(id) => {
                            if self.token.is_some() {
                                // 既に接続済みの場合
                                let error: Response = Err(ResponseErr::AlreadyLoginnedIn);
                                let res_str = serde_json::to_string(&error).unwrap();
                                ctx.text(res_str);
                            } else {
                                // 未接続の場合
                                self.addr
                                    .send(master_router::Connect(id))
                                    .into_actor(self)
                                    .then(|res, act, ctx| {
                                        match res {
                                            Ok(Ok(res)) => {
                                                act.token = Some({
                                                    use std::mem::MaybeUninit;
                                                    let mut token: [MaybeUninit<u8>; 32] = unsafe {
                                                        MaybeUninit::uninit().assume_init()
                                                    };
                                                    for (i, slot) in token.iter_mut().enumerate() {
                                                        *slot = MaybeUninit::new(res[i]);
                                                    }
                                                    unsafe { std::mem::transmute(token) }
                                                });
                                            }
                                            Ok(Err(err)) => {
                                                use SessionError::*;
                                                let err = match err {
                                                MasterError(
                                                    werewolf::master::Error::NameAlreadyRegistered(
                                                        name,
                                                    ),
                                                ) => ResponseErr::NameAlreadyRegistered(name),
                                                InvalidToken => ResponseErr::InvalidToken,
                                                MasterError(
                                                    werewolf::master::Error::GameAlreadyStarted,
                                                ) => ResponseErr::GameAlreadyStarted,
                                                MasterError(
                                                    werewolf::master::Error::AuthenticationFailed,
                                                ) => ResponseErr::AuthenticationFailed,
                                                _ => {
                                                    return fut::ready(());
                                                }
                                            };
                                                let error: Response = Err(err);
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
                    let error: Response = Err(ResponseErr::JsonParse(text.to_string()));
                    let res_str = serde_json::to_string(&error).unwrap();
                    ctx.text(res_str);
                }
            }
            ws::Message::Binary(_) => println!("Unexpected binary"),
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
        }
    }
}

/// クライアントからのリクエストの構造体
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Request {
    Connect(Identifier),
    Werewolf(werewolf::request::Any),
}

/// クライアントへ送る構造体
pub type Response = Result<ResponseOk, ResponseErr>;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ResponseOk {

}

#[derive(Serialize, Deserialize, Error, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ResponseErr {
    #[error("Json parse error: {0}")]
    JsonParse(String),
    #[error("the game has already started.")]
    InvalidToken,
    #[error("name '{0}' is already registered.")]
    NameAlreadyRegistered(String),
    #[error("authentication failed.")]
    AuthenticationFailed,
    #[error("the game has already started.")]
    GameAlreadyStarted,
    #[error("You are already logged in.")]
    AlreadyLoginnedIn,
}

// /// Message handler from Master to Client
// impl Handler<master_router::State> for WsPlayerSession {
//     fn handle(&mut self, msg: master_router::State, ctx: &mut Self::Context) -> Self::Result {
//     }
// }
