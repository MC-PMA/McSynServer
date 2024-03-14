use actix::Actor;
use rusqlite::{params, Connection};

use crate::DIR_PATH_SQLITE;

struct SinglePlayerMoney {
    uuid: String,
    balance: i32,
}

pub struct SingleMoneySqlite {
    // 负债额度
    pub debt_limit: i32,
}

impl SingleMoneySqlite {
    pub async fn init(debt_limit: i32) -> Self {
        tokio::fs::create_dir_all(DIR_PATH_SQLITE).await.err();
        let conn = Connection::open(DIR_PATH_SQLITE.to_owned() + "/money.db").unwrap();
        // 初始化创建单经济表
        let stmt = include_str!("../sql/sqlite/single_economy/init.sql");
        conn.execute_batch(stmt).unwrap();
        Self {
            debt_limit: -debt_limit,
        }
    }

    // 设置负债额度
    fn _setdebt_limit(&mut self, debt_limit: i32) {
        self.debt_limit = -debt_limit;
    }

    // 添加一条玩家经济
    pub fn init_pl_money(&self, player: SinglePlayerMoney) -> Result<usize, rusqlite::Error> {
        let conn = Connection::open(DIR_PATH_SQLITE.to_owned() + "/money.db").unwrap();
        let stmt = include_str!("../sql/sqlite/single_economy/add_pl.sql");
        conn.execute(stmt, params![player.uuid, &player.balance])
    }

    // 获取玩家经济
    pub fn get_pl_money(&self, uuid: String) -> Result<i32, rusqlite::Error> {
        let conn = Connection::open(DIR_PATH_SQLITE.to_owned() + "/money.db").unwrap();
        let stmt = include_str!("../sql/sqlite/single_economy/get_money.sql");
        conn.query_row(stmt, params![uuid], |row| row.get(0))
    }

    // 更新玩家经济
    pub fn update_pl_money(&self, player: SinglePlayerMoney) -> Result<usize, rusqlite::Error> {
        let conn = Connection::open(DIR_PATH_SQLITE.to_owned() + "/money.db").unwrap();
        let stmt = include_str!("../sql/sqlite/single_economy/updata_money.sql");
        conn.execute(stmt, params![player.balance, player.uuid])
    }

    // 增加玩家经济
    pub fn add_pl_money(&self, player: SinglePlayerMoney) -> Result<usize, rusqlite::Error> {
        let balance = self.get_pl_money(player.uuid.clone()).unwrap();
        let player = SinglePlayerMoney {
            uuid: player.uuid,
            balance: balance + player.balance,
        };
        self.update_pl_money(player)
    }

    // 减少玩家经济
    pub fn reduce_pl_money(&self, player: SinglePlayerMoney) -> Result<usize, rusqlite::Error> {
        let balance = self.get_pl_money(player.uuid.clone()).unwrap();
        let player = SinglePlayerMoney {
            uuid: player.uuid,
            balance: balance - player.balance,
        };
        self.update_pl_money(player)
    }

    // 玩家经济转账
    pub fn transfer_pl_money(
        &self,
        player1_uuid: String,
        player2_uuid: String,
        balance1: i32,
    ) -> bool {
        let balance = self.get_pl_money(player1_uuid.clone()).unwrap();
        if balance <= self.debt_limit {
            return false;
        } else {
            let player1 = SinglePlayerMoney {
                uuid: player1_uuid,
                balance: balance - balance1,
            };
            let balance = self.get_pl_money(player2_uuid.clone()).unwrap();
            let player2 = SinglePlayerMoney {
                uuid: player2_uuid,
                balance: balance + balance1,
            };
            match self.update_pl_money(player1) {
                Ok(_) => match self.update_pl_money(player2) {
                    Ok(_) => return true,
                    Err(_) => return false,
                },
                Err(_) => return false,
            }
        }
    }
}

impl Actor for SingleMoneySqlite {
    type Context = actix::Context<Self>;
}
