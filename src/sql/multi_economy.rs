use actix::Actor;
use log::info;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::DIR_PATH_SQLITE;

#[derive(Debug, Clone)]
pub struct MultiMoneySqlite {
    //默认经济名
    pub default_economy: String,
    // 负债额度
    pub debt_limit: i32,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct MultiMoney {
    // 经济名
    pub money: String,
    pub key: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MultiPlayerMoney {
    uuid: String,
    money: String,
    balance: i32,
}
impl Default for MultiMoneySqlite {
    fn default() -> Self {
        Self {
            default_economy: "money".to_string(),
            debt_limit: -200,
        }
    }
}
impl MultiMoneySqlite {
    /// 初始化,创建money.db
    /// init create money.db
    /// # Example
    /// ```rust
    /// use crate::sql::money::MultiMoneySqlite;
    /// let multi_money_sqlite=MultiMoneySqlite::default();
    /// let multi_money_sqlite = multi_money_sqlite.init().await;
    /// ```
    pub async fn init(&self) {
        tokio::fs::create_dir_all(DIR_PATH_SQLITE).await.err();
        let conn = Connection::open(DIR_PATH_SQLITE.to_owned() + "/money.db").unwrap();
        let stmt = include_str!("../sql/sqlite/multi_economy/init.sql");
        conn.execute_batch(stmt).unwrap();
    }

    /// 添加一种经济
    /// add a kind of money
    /// # Example
    /// ```rust
    /// use crate::sql::money::MultiMoneySqlite;
    /// let multi_money_sqlite = MultiMoneySqlite::init().await;
    /// let multi_money = MultiMoney {
    ///    money: "money".to_string(),
    ///   key: "money".to_string(),
    /// };
    /// multi_money_sqlite.add_money(multi_money);
    /// ```
    pub fn add_money(&self, multi_economy: MultiMoney) -> Result<usize, rusqlite::Error> {
        let conn = Connection::open(DIR_PATH_SQLITE.to_owned() + "/money.db").unwrap();
        let stmt = include_str!("../sql/sqlite/multi_economy/add_money.sql");
        conn.execute(stmt, params![multi_economy.money, multi_economy.key])
    }

    // 删除一种经济体
    pub fn delete_money_key(&self, money: MultiMoney) -> Result<usize, rusqlite::Error> {
        let conn = Connection::open(DIR_PATH_SQLITE.to_owned() + "/money.db").unwrap();
        let stmt = include_str!("../sql/sqlite/multi_economy/delete_money_key.sql");
        let _ = conn.execute(stmt, params![money.money, money.key]);
        self.delete_money(money.money)
    }

    // 删除经济->玩家经济
    fn delete_money(&self, money: String) -> Result<usize, rusqlite::Error> {
        let conn = Connection::open(DIR_PATH_SQLITE.to_owned() + "/money.db").unwrap();
        let stmt = include_str!("../sql/sqlite/multi_economy/delete_money.sql");
        conn.execute(stmt, params![money])
    }

    // 修改一种经济体
    pub fn update_money_key(
        &self,
        multi_economy: MultiMoney,
        money: String,
    ) -> Result<usize, rusqlite::Error> {
        if self.get_money_key(money.clone()).unwrap() {
            let conn = Connection::open(DIR_PATH_SQLITE.to_owned() + "/money.db").unwrap();
            let stmt = include_str!("../sql/sqlite/multi_economy/update_money_key_name.sql");
            let _ = conn.execute(stmt, params![money, multi_economy.money, multi_economy.key]);
            self.update_money(multi_economy, money)
        } else {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
    }

    // 修改玩家经济名
    pub fn update_money(
        &self,
        multi_economy: MultiMoney,
        money: String,
    ) -> Result<usize, rusqlite::Error> {
        let conn = Connection::open(DIR_PATH_SQLITE.to_owned() + "/money.db").unwrap();
        let stmt = include_str!("../sql/sqlite/multi_economy/update_money_name.sql");
        conn.execute(stmt, params![money, multi_economy.money])
    }

    // 检查某个经济是否存在
    pub fn get_money_key(&self, money_name: String) -> Result<bool, rusqlite::Error> {
        let conn = Connection::open(DIR_PATH_SQLITE.to_owned() + "/money.db").unwrap();
        let stmt = include_str!("../sql/sqlite/multi_economy/get_money_key.sql");
        let mut stmt = conn.prepare(stmt).unwrap();
        let mut rows = stmt.query(params![money_name]).unwrap();
        if let Some(_) = rows.next().unwrap() {
            return Ok(true);
        } else {
            return Ok(false);
        }
    }

    // 获取所有经济
    pub fn get_all_money(&self) -> Result<Vec<MultiMoney>, rusqlite::Error> {
        let conn = Connection::open(DIR_PATH_SQLITE.to_owned() + "/money.db").unwrap();
        // let stmt = include_str!("../sql/sqlite/multi_economy/get_all_money.sql");
        let stmt = "SELECT * FROM multi_economy_key";
        let mut stmt = conn.prepare(stmt).unwrap();
        let mut rows = stmt.query(params![]).unwrap();
        let mut multi_moneys = Vec::new();
        while let Some(row) = rows.next().unwrap() {
            multi_moneys.push(MultiMoney {
                money: row.get(0).unwrap(),
                key: row.get(1).unwrap(),
            });
        }
        Ok(multi_moneys)
    }

    // 添加一条玩家经济
    pub fn init_pl_money(&self, player: MultiPlayerMoney) -> Result<usize, rusqlite::Error> {
        if self.get_money_key(player.money.clone()).unwrap() {
            let conn = Connection::open(DIR_PATH_SQLITE.to_owned() + "/money.db").unwrap();
            let stmt = include_str!("../sql/sqlite/multi_economy/add_pl_money.sql");
            // let stmt = "INSERT INTO multi_economy (uuid, money, balance) VALUES (?, ?, ?)";
            conn.execute(stmt, params![player.uuid, player.money, &player.balance])
        } else {
            Err(rusqlite::Error::QueryReturnedNoRows)
        }
    }

    // 获取玩家经济
    pub fn get_pl_money(&self, uuid: String, money: String) -> Result<i32, rusqlite::Error> {
        let conn = Connection::open(DIR_PATH_SQLITE.to_owned() + "/money.db").unwrap();
        let stmt = include_str!("../sql/sqlite/multi_economy/get_money.sql");
        conn.query_row(stmt, params![uuid, money], |row| row.get(0))
    }

    // 更新玩家经济
    pub fn update_pl_money(&self, player: MultiPlayerMoney) -> Result<usize, rusqlite::Error> {
        if self.get_money_key(player.money.clone()).unwrap() {
            let conn = Connection::open(DIR_PATH_SQLITE.to_owned() + "/money.db").unwrap();
            let stmt = include_str!("../sql/sqlite/multi_economy/updata_money.sql");
            conn.execute(stmt, params![player.balance, player.uuid, player.money])
        } else {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
    }

    // 增加玩家经济
    pub fn add_pl_money(&self, player: MultiPlayerMoney) -> Result<usize, rusqlite::Error> {
        let balance = self
            .get_pl_money(player.uuid.clone(), player.money.clone());
        match balance {
            Ok(balance) => {
                let player = MultiPlayerMoney {
                    uuid: player.uuid,
                    money: player.money,
                    balance: balance + player.balance,
                };
                self.update_pl_money(player)
            }
            Err(_err) => {
                Err(_err)
            }
        }
        
    }

    // 减少玩家经济
    pub fn reduce_pl_money(&self, player: MultiPlayerMoney) -> Result<usize, rusqlite::Error> {
        let balance = self
            .get_pl_money(player.uuid.clone(), player.money.clone());
        match balance {
            Ok(balance) => {
                let player = MultiPlayerMoney {
                    uuid: player.uuid,
                    money: player.money,
                    balance: balance - player.balance,
                };
                self.update_pl_money(player)
            }
            Err(_err) => {
                Err(_err)
            }
        }
    }

    // 玩家经济转账
    pub fn transfer_pl_money(
        &self,
        player1_uuid: String,
        player2_uuid: String,
        balance1: i32,
        money: String,
    ) -> bool {
        let balance = self
            .get_pl_money(player1_uuid.clone(), money.clone())
            .unwrap();
        println!("{:?}", balance);
        println!("{:?}",self. debt_limit);
        if balance >= self.debt_limit{
            let player1 = MultiPlayerMoney {
                uuid: player1_uuid,
                money: money.clone(),
                balance: balance - balance1,
            };
            let balance = self
                .get_pl_money(player2_uuid.clone(), money.clone())
                .unwrap();
            let player2 = MultiPlayerMoney {
                uuid: player2_uuid,
                money: money,
                balance: balance + balance1,
            };
            match self.update_pl_money(player1) {
                Ok(_) => match self.update_pl_money(player2) {
                    Ok(_) => return true,
                    Err(_) => return false,
                },
                Err(_) => return false,
            }
        }else {
            return false;
        }
    }
}

impl Actor for MultiMoneySqlite {
    type Context = actix::Context<Self>;
}

#[tokio::test]
async fn test() {
    let multi_money_sqlite = MultiMoneySqlite::default();
    multi_money_sqlite.init();
    let multi_money = MultiMoney {
        money: "73".to_string(),
        key: "money".to_string(),
    };

    let is = multi_money_sqlite.get_money_key("57".to_string());
    println!("{:?}", is);
}
