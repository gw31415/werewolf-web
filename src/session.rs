use std::time::{Duration, Instant};

use actix::prelude::*;
use actix_web_actors::ws;

use werewolf::master::Token;

use crate::master_router::{self, Identifier, SessionError};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug)]
pub struct WsPlayerSession {
    id: Identifier,
    hb: Instant,
    addr: Addr<master_router::MasterRouter>,
}

impl WsPlayerSession {
    /// 新規接続を行う
    pub fn new(name: String, master: String, addr: Addr<master_router::MasterRouter>) -> Self {
        WsPlayerSession {
            id: Identifier::Signup { name, master },
            hb: Instant::now(),
            addr,
        }
    }
    /// 再接続を行う
    pub fn restore(token: Token, addr: Addr<master_router::MasterRouter>) -> Self {
        WsPlayerSession {
            id: Identifier::Token(token),
            hb: Instant::now(),
            addr,
        }
    }
    /// ハートビート
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                if let Identifier::Token(id) = act.id {
                    act.addr.do_send(master_router::Disconnect { token: id });
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
        self.hb(ctx);
        self.addr
            .send(master_router::Connect(self.id.clone()))
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(Ok(res)) => {
                        act.id = Identifier::Token({
                            use std::mem::MaybeUninit;
                            let mut token: [MaybeUninit<u8>; 32] =
                                unsafe { MaybeUninit::uninit().assume_init() };
                            for (i, slot) in token.iter_mut().enumerate() {
                                *slot = MaybeUninit::new(res[i]);
                            }
                            unsafe { std::mem::transmute(token) }
                        });
                    }
                    Ok(Err(err)) => {
                        use ws::*;
                        use SessionError::*;
                        let reason = match err {
                            MasterError(werewolf::master::Error::NameAlreadyRegistered(name)) => {
                                CloseReason {
                                    code: CloseCode::Error,
                                    description: Some(format!(
                                        "name '{name}' is already registered."
                                    )),
                                }
                            }
                            InvalidToken => CloseReason {
                                code: CloseCode::Error,
                                description: Some("invalid token.".to_string()),
                            },
                            MasterError(werewolf::master::Error::GameAlreadyStarted) => {
                                CloseReason {
                                    code: CloseCode::Error,
                                    description: Some("the game has already started.".to_string()),
                                }
                            }
                            MasterError(werewolf::master::Error::AuthenticationFailed) => {
                                CloseReason {
                                    code: CloseCode::Error,
                                    description: Some("authentication failed.".to_string()),
                                }
                            }
                            MasterError(err) => CloseReason {
                                code: CloseCode::Error,
                                description: Some(format!("the error of master-side: {:0}.", err)),
                            },
                        };
                        ctx.close(Some(reason));
                        ctx.stop();
                    }
                    _ => {
                        ctx.stop();
                    }
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        if let Identifier::Token(token) = self.id {
            self.addr.do_send(master_router::Disconnect { token });
        }
        Running::Stop
    }
}

/// WebSocket message handler
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
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                let _m = text.trim();
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
