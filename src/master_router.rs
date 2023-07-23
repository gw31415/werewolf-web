use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex, RwLock},
};
use thiserror::Error;

use actix::prelude::*;
use werewolf::{master::Token, state::Name, Master};

#[derive(Debug, Clone)]
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
    /// マスターの一覧
    masters: Arc<Mutex<HashMap<MasterName, Master>>>,
    /// トークンからマスターへのマップ
    routes: Arc<RwLock<HashMap<Token, MasterName>>>,
    /// コネクションが保たれているユーザーがどのマスターに所属しているか。
    /// マスターの掃除に使う
    online: Arc<Mutex<HashMap<MasterName, HashSet<Token>>>>,
}

impl MasterRouter {
    pub fn new() -> MasterRouter {
        MasterRouter {
            masters: Default::default(),
            online: Default::default(),
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
        let (token, mastername) = match msg.0 {
            Identifier::Signup {
                name,
                master: mastername,
            } => {
                // 新規サインアップ
                let mut masters = self.masters.lock().unwrap();
                let master = if let Some(master) = masters.get_mut(&mastername) {
                    // 既にmasterが存在している場合
                    master
                } else {
                    // masterの新規作成
                    masters.insert(mastername.clone(), Default::default());

                    // online欄の作成と初期化
                    self.online
                        .lock()
                        .unwrap()
                        .insert(mastername.clone(), HashSet::new());
                    // masterを返す
                    masters.get_mut(&mastername).unwrap()
                };
                let token = master.signup(name)?;
                (token, mastername)
            }
            Identifier::Token(token) => {
                // 再接続
                (token, {
                    let routes = self.routes.read().unwrap();
                    if let Some(mastername) = routes.get(&token) {
                        mastername.to_owned()
                    } else {
                        // 存在の確認
                        return Err(SessionError::InvalidToken);
                    }
                })
            }
        };

        // onlineの追加
        self.online
            .lock()
            .unwrap()
            .get_mut(&mastername)
            .unwrap()
            .insert(token);
        // routesの追加
        self.routes.write().unwrap().insert(token, mastername);

        Ok(token.to_vec())
    }
}

impl Handler<Disconnect> for MasterRouter {
    type Result = Result<(), SessionError>;

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) -> Self::Result {
        let mastername = {
            let routes = self.routes.read().unwrap();
            let Some(mastername) = routes.get(&msg.token) else {
                return Err(SessionError::InvalidToken);
            };
            mastername.to_owned()
        };

        let online_members_is_empty = {
            let mut online = self.online.lock().unwrap();
            let online_members = online.get_mut(&mastername).unwrap();
            online_members.remove(&msg.token);
            online_members.is_empty()
        };

        if online_members_is_empty {
            // Remove master
            self.masters.lock().unwrap().remove(&mastername);
            println!("Master named '{mastername}' removed.");

            {
                // Cleaning routes
                let mut keys = Vec::<Token>::new();
                for (k, v) in self.routes.read().unwrap().iter() {
                    if v == &mastername {
                        keys.push(*k);
                    }
                }
                for k in keys {
                    self.routes.write().unwrap().remove(&k);
                }
            }

            // Remove empty record of online
            self.online.lock().unwrap().remove(&mastername);
        }
        Ok(())
    }
}
