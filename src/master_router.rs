use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};
use thiserror::Error;

use actix::prelude::*;
use werewolf::{master::Token, state::Name, Master};

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

#[derive(Message)]
#[rtype(result = "Result<Vec<u8>, SessionError>")]
pub struct Connect(pub Identifier);

#[derive(Message)]
#[rtype(result = "Result<(), SessionError>")]
pub struct Disconnect {
    pub token: Token,
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
    online: HashSet<Token>,
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
        let (token, online) = match msg.0 {
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
        online.insert(token);

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
