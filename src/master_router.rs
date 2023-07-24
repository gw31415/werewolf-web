use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use thiserror::Error;

use actix::prelude::*;
use werewolf::{master::Token, state::Name, Master};

use crate::session::Response;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Identifier {
    Token(Token),
    Signup { name: Name, master: MasterName },
}

#[derive(MessageResponse, Debug, Error)]
pub enum SessionError {
    #[error("WerewolfError: {0}")]
    MasterError(#[from] werewolf::master::Error),
    #[error("Invalid token.")]
    InvalidToken,
}

/// 接続する。ゲーム情報の登録を行うのと同義。
#[derive(Message)]
#[rtype(result = "Result<Vec<u8>, SessionError>")]
pub struct Connect {
    /// ログイン or サインアップ情報
    /// Tokenの場合はログイン、名前とマスター名の場合はサインアップを行う
    pub id: Identifier,
    /// 通信のためのアドレス
    pub addr: Recipient<Response>,
}

/// 切断する
#[derive(Message)]
#[rtype(result = "Result<(), SessionError>")]
pub struct Disconnect {
    pub token: Token,
}

/// 切断する
#[derive(Message)]
#[rtype(result = "()")]
pub struct Werewolf {
    pub token: Token,
    pub body: werewolf::request::Any,
}

impl Handler<Werewolf> for MasterRouter {
    type Result = ();
    fn handle(&mut self, msg: Werewolf, _: &mut Self::Context) -> Self::Result {
        // TODO: Stateを各ユーザに配信する
        let Werewolf { token, body } = msg;
        let (master, addr) = {
            let name = self.routes.get(&token).unwrap();
            let MasterInstance { master, online } = self.masters.get_mut(name).unwrap();
            let addr = online.get(&token).unwrap();
            (master, addr)
        };
        let mut master = master.lock().unwrap();
        let Ok(permission) = master.login(&token) else {
            addr.do_send(Response::Error(crate::session::ResponseErr::InvalidToken));
            return;
        };
        if let Err(err) = permission.execute(body) {
            addr.do_send(Response::Error(crate::session::ResponseErr::Werewolf(err)));
        }
    }
}

type MasterName = String;

/// マスターへマップするActor。
#[derive(Clone)]
pub struct MasterRouter {
    /// 待機中マスターの一覧
    masters: HashMap<MasterName, MasterInstance>,
    /// トークンからマスターへのマップ
    routes: HashMap<Token, MasterName>,
}

#[derive(Clone, Default)]
struct MasterInstance {
    /// マスターの実体
    master: Arc<Mutex<Master>>,
    /// コネクションが保たれているユーザーのリスト
    online: HashMap<Token, Recipient<Response>>,
}

impl MasterRouter {
    pub fn new() -> MasterRouter {
        MasterRouter {
            masters: Default::default(),
            routes: Default::default(),
        }
    }
}

impl Actor for MasterRouter {
    type Context = Context<Self>;
}

impl Handler<Connect> for MasterRouter {
    type Result = Result<Vec<u8>, SessionError>;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        let (token, online) = match msg.id {
            Identifier::Signup {
                name,
                master: mastername,
            } => {
                // 新規サインアップ
                let MasterInstance { master, online } =
                    self.masters.entry(mastername.clone()).or_default();

                let token = master.lock().unwrap().signup(name)?;
                // routesの追加
                self.routes.insert(token, mastername);

                (token, online)
            }
            Identifier::Token(token) => {
                // 再接続
                (token, {
                    if let Some(mastername) = self.routes.get(&token) {
                        let MasterInstance { online, .. } =
                            self.masters.get_mut(mastername).unwrap();
                        online
                    } else {
                        // 存在の確認
                        return Err(SessionError::InvalidToken);
                    }
                })
            }
        };

        // onlineの追加
        online.insert(token, msg.addr);

        Ok(token.to_vec())
    }
}

impl Handler<Disconnect> for MasterRouter {
    type Result = Result<(), SessionError>;

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) -> Self::Result {
        let mastername = {
            let Some(mastername) = self.routes.get(&msg.token) else {
                return Err(SessionError::InvalidToken);
            };
            mastername.to_owned()
        };

        let MasterInstance { online, .. } = self.masters.get_mut(&mastername).unwrap();

        online.remove(&msg.token);

        if online.is_empty() {
            // Remove master
            self.masters.remove(&mastername);

            // Clean routes
            self.routes.retain(|_, v| v != &mastername);
        }
        Ok(())
    }
}
