use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex, RwLock},
};

use actix::prelude::*;
use werewolf::{master::Token, state::Name, Master};

#[derive(Debug, Clone)]
pub enum Identifier {
    Token(Token),
    Signup { name: Name, master: MasterName },
}

#[derive(Message)]
#[rtype(result = "Vec<u8>")]
pub struct Connect(pub Identifier);

#[derive(Message)]
#[rtype(result = "()")]
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
    type Result = Vec<u8>;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        let (token, mastername) = match msg.0 {
            Identifier::Signup {
                name,
                master: mastername,
            } => {
                // 新規サインアップ
                let mut masters = self.masters.lock().unwrap();
                let master = if let Some(master) = masters.get_mut(&mastername) {
                    println!("Master named '{mastername}' already exists.");
                    // 既にmasterが存在している場合
                    master
                } else {
                    println!("Master named '{mastername}' created.");

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
                let Ok(token) = master.signup(name) else {
                    todo!("名前重複Errorを返す")
                };
                (token, mastername)
            }
            Identifier::Token(token) => {
                // 再接続
                (token, {
                    let routes = self.routes.read().unwrap();
                    let Some(mastername) = routes.get(&token) else {
                        // 存在の確認
                        todo!("InvalidToken Errorを返す")
                    };
                    mastername.to_owned()
                })
            }
        };

        {
            // Debug Log
            // token.iter().map(|b| format!("{:02x}", b)).collect::<String>()
            const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";
            let mut v = vec![0u8; 64];
            for (i, &token) in token.iter().enumerate() {
                v[i * 2] = HEX_CHARS[(token >> 4) as usize];
                v[i * 2 + 1] = HEX_CHARS[(token & 0xf) as usize];
            }
            println!("New Connection: {}", unsafe {
                std::str::from_utf8_unchecked(&v)
            });
            println!("  Master: {mastername}");
        }

        // onlineの追加
        self.online
            .lock()
            .unwrap()
            .get_mut(&mastername)
            .unwrap()
            .insert(token);
        // routesの追加
        self.routes.write().unwrap().insert(token, mastername);

        token.to_vec()
    }
}

impl Handler<Disconnect> for MasterRouter {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        let mastername = {
            let routes = self.routes.read().unwrap();
            let Some(mastername) = routes.get(&msg.token) else {
                todo!("InvalidToken Errorを返す")
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
    }
}
