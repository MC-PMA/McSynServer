use std::collections::HashMap;

use actix::{Actor, Context, Handler, Message};
use serde::{Deserialize, Serialize};

/// 玩家基础数据
#[derive(Serialize, Deserialize, Clone)]
pub struct Player {
    /// 消息
    data: String,

    /// 玩家名
    name: String,

    /// 服务器
    server: String,
}

pub struct PlayerManager {
    players: HashMap<String, Player>,
}

impl Actor for PlayerManager {
    type Context = Context<Self>;
}

impl PlayerManager {
    pub fn new() -> PlayerManager {
        PlayerManager {
            players: HashMap::new(),
        }
    }
}

impl PlayerManager {
    /// 添加玩家
    pub fn add_player(&mut self, player: Player) {
        self.players.insert(player.clone().name, player);
    }
    /// 移除某个玩家
    pub fn remove_player(&mut self, player: Player) {
        self.players.remove(&player.name);
    }
    /// 获取所有玩家名字的列表
    pub fn get_players(&self) -> Vec<String> {
        self.players.keys().cloned().collect()
    }
    /// 移除所有玩家
    pub fn _remove_player_all(&mut self) {
        self.players.clear();
    }
    /// 删除指定服务端下的所有玩家
    fn remove_players_by_server(&mut self, server: &str) {
        self.players.retain(|_, player| player.server != server);
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct PlayerLeft {
    pub player: Player,
}

impl Handler<PlayerLeft> for PlayerManager {
    type Result = ();

    fn handle(&mut self, player: PlayerLeft, _: &mut Context<Self>) {
        self.remove_player(player.player)
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct PlayerJoin {
    pub player: Player,
}

impl Handler<PlayerJoin> for PlayerManager {
    type Result = ();

    fn handle(&mut self, player: PlayerJoin, _: &mut Context<Self>) {
        self.add_player(player.player)
    }
}

#[derive(Message)]
#[rtype(result = "Vec<String>")]
pub struct PlayersGet;
impl Handler<PlayersGet> for PlayerManager {
    type Result = Vec<String>;

    fn handle(&mut self, _: PlayersGet, _: &mut Context<Self>) -> Vec<String> {
        self.get_players()
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct PlayersRemoveByServer {
    pub server: String,
}
impl Handler<PlayersRemoveByServer> for PlayerManager {
    type Result = ();

    fn handle(&mut self, players_remove_by_server: PlayersRemoveByServer, _: &mut Context<Self>) {
        self.remove_players_by_server(&players_remove_by_server.server);
    }
}
