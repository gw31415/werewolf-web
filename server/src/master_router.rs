use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use actix::prelude::*;
use futures::{future::join_all, FutureExt};
use werewolf::{
    master::Token,
    state::{Name, State},
    Master,
};

use crate::session::{Response, ResponseErr, ResponseOk};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Identifier {
    Token(String),
    Signup { name: Name, master: MasterName },
}

/// 接続する。ゲーム情報の登録を行うのと同義。
#[derive(Message)]
#[rtype(result = "Result<Box<[u8]>, ResponseErr>")]
pub struct Connect {
    /// ログイン or サインアップ情報
    /// Tokenの場合はログイン、名前とマスター名の場合はサインアップを行う
    pub id: Identifier,
    /// 通信のためのアドレス
    pub addr: Recipient<Response>,
}

/// 切断する
#[derive(Message)]
#[rtype(result = "Result<(), ResponseErr>")]
pub struct Disconnect {
    pub token: Token,
}

/// Werewolfゲームマスターにリクエストを送信する
#[derive(Message)]
#[rtype(result = "()")]
pub struct Werewolf {
    pub token: Token,
    pub body: werewolf::request::Any,
}

impl Handler<Werewolf> for MasterRouter {
    type Result = ();
    fn handle(&mut self, msg: Werewolf, _: &mut Self::Context) -> Self::Result {
        let Werewolf { token, body } = msg;
        let MasterInstance { master, online } = {
            let name = self.routes.get(&token).unwrap();
            self.masters.get_mut(name).unwrap()
        };
        let mut master = master.lock().unwrap();

        {
            // Stateの更新
            let connection = online.get(&token).unwrap();
            let Ok(permission) = master.login(&token) else {
                connection.addr.do_send(Response::Error(ResponseErr::Session(werewolf::master::Error::AuthenticationFailed)));
                return;
            };
            if let Err(err) = permission.execute(body) {
                connection
                    .addr
                    .do_send(Response::Error(crate::session::ResponseErr::Werewolf(err)));
                return;
            }
        }

        // Stateの配信
        for (token, connection) in online.iter_mut() {
            let permission = master.login(token).unwrap();
            let state = permission.view_state();
            connection.update_state(state);
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

/// マスターの実体と付随する接続中ユーザーリストを保持。
#[derive(Clone, Default)]
struct MasterInstance {
    /// マスターの実体
    master: Arc<Mutex<Master>>,
    /// コネクションが保たれているユーザーのリスト
    online: HashMap<Token, Connection>,
}

/// 現在オンラインの接続先
#[derive(Clone)]
struct Connection {
    /// アドレス
    addr: Recipient<Response>,
    /// 前回送信したState
    prev_state: Option<State>,
}

impl Connection {
    /// 新規 Connection
    fn new(addr: Recipient<Response>) -> Self {
        Self {
            addr,
            prev_state: None,
        }
    }

    /// Stateの更新がある場合送信する。
    /// Stateのフィルタリングはしないので注意すること。
    fn update_state(&mut self, state: State) {
        if Some(&state) != self.prev_state.as_ref() {
            self.addr
                .do_send(Response::Success(ResponseOk::State(Box::new(
                    state.clone(),
                ))));
            self.prev_state = Some(state);
        }
    }
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
    type Result = Result<Box<[u8]>, ResponseErr>;

    fn handle(&mut self, msg: Connect, ctx: &mut Context<Self>) -> Self::Result {
        // データの取り出し
        let (token, master, online, new_members, mastername, name) = match msg.id {
            Identifier::Signup {
                name,
                master: mastername,
            } => {
                // 新規サインアップ
                let MasterInstance { master, online } =
                    self.masters.entry(mastername.clone()).or_default();

                let token = master.lock().unwrap().signup(name.clone())?;
                // routesの追加
                self.routes.insert(token, mastername.clone());
                (token, master, online, true, mastername, name)
            }
            Identifier::Token(token) => {
                use base64::Engine as _;
                if let Ok(vec) = general_purpose::STANDARD.decode(token) {
                    let token = {
                        if vec.len() != 32 {
                            // 存在の確認
                            return Err(werewolf::master::Error::AuthenticationFailed)?;
                        }
                        let mut array: std::mem::MaybeUninit<[u8; 32]> =
                            std::mem::MaybeUninit::uninit();
                        unsafe {
                            array
                                .as_mut_ptr()
                                .copy_from(vec.as_ptr() as *const [u8; 32], 32);
                            array.assume_init()
                        }
                    };
                    // 再接続
                    if let Some(mastername) = self.routes.get(&token) {
                        let MasterInstance { online, master } =
                            self.masters.get_mut(mastername).unwrap();
                        let name = master.lock().unwrap().get_name(&token).unwrap().to_owned();
                        (token, master, online, false, mastername.to_owned(), name)
                    } else {
                        // 存在の確認
                        return Err(werewolf::master::Error::AuthenticationFailed)?;
                    }
                } else {
                    // 存在の確認
                    return Err(werewolf::master::Error::AuthenticationFailed)?;
                }
            }
        };

        // トークンの通知
        use base64::{engine::general_purpose, Engine as _};
        msg.addr
            .do_send(Response::Success(ResponseOk::AuthenticationSuccess {
                token: general_purpose::STANDARD.encode(token),
                master: mastername,
                name,
            }));

        // onlineの追加
        online.insert(token, Connection::new(msg.addr));

        // オンラインメンバーおよび新規メンバーの通知
        {
            let online_members: HashSet<String> = online
                .iter_mut()
                .map(|(m, _)| master.lock().unwrap().get_name(m).unwrap().to_owned())
                .collect();

            let members = master
                .lock()
                .unwrap()
                .players()
                .into_iter()
                .map(|m| m.to_owned())
                .collect::<HashSet<_>>();

            join_all(online.iter_mut().flat_map(|(key, addr)| {
                let mut fs = vec![addr.addr.send(Response::Success(ResponseOk::Online(
                    online_members.clone(),
                )))];
                if new_members || key == &token {
                    // 新規メンバーがいる場合はMember一覧も通知する
                    fs.push(
                        addr.addr
                            .send(Response::Success(ResponseOk::Members(members.clone()))),
                    );
                }
                fs
            }))
            .then(|_| fut::ready(()))
            .into_actor(self)
            .wait(ctx);
        }

        Ok(Box::new(token))
    }
}

impl Handler<Disconnect> for MasterRouter {
    type Result = Result<(), ResponseErr>;

    fn handle(&mut self, msg: Disconnect, ctx: &mut Context<Self>) -> Self::Result {
        let mastername = {
            let Some(mastername) = self.routes.get(&msg.token) else {
                Err(werewolf::master::Error::AuthenticationFailed)?
            };
            mastername.to_owned()
        };

        let MasterInstance { online, master } = self.masters.get_mut(&mastername).unwrap();

        online.remove(&msg.token);

        if online.is_empty() {
            // Remove master
            self.masters.remove(&mastername);

            // Clean routes
            self.routes.retain(|_, v| v != &mastername);
        } else {
            // メンバーがまだいる場合はオンラインメンバーの変更通知
            let online_members: HashSet<String> = online
                .iter_mut()
                .map(|(m, _)| master.lock().unwrap().get_name(m).unwrap().to_owned())
                .collect();

            join_all(online.iter_mut().map(|(_, addr)| {
                addr.addr.send(Response::Success(ResponseOk::Online(
                    online_members.clone(),
                )))
            }))
            .then(|_| fut::ready(()))
            .into_actor(self)
            .wait(ctx);
        }

        Ok(())
    }
}
