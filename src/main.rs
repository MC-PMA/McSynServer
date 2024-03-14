use std::sync::Arc;

use actix::{fut::ok, *};

use actix_web::{dev::Service, guard, web, App, HttpResponse, HttpServer};
use futures_util::future::FutureExt;
mod api;
use api::pe::chatserver;
use log::info;

use rusqlite::Connection;

use crate::{
    api::{money::money::money_config, pe::{
        player::PlayerManager, player_api::player_config, session::chatserver_config,
        world::world_config,
    }},
    config::ServerConfig,
    sql::{multi_economy::MultiMoneySqlite, single_economy::SingleMoneySqlite},
};

mod config;
mod sql;

use rust_i18n::t;

rust_i18n::i18n!("locales");

use crate::config::init_log;

const DIR_PATH_PLAYER_NBT: &str = "./api/pe/player";
const DIR_PATH_WORLD: &str = "./api/pe/world";
const DIR_PATH_SQLITE: &str = "./api/sql/sqlite";

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    init_log();
    // t!("hello", locale = "zh-CN");
    let config = ServerConfig::default();

    // start chat server actor
    let server = chatserver::ChatServer::new().start();
    let players = PlayerManager::new().start();

    // println!("{}", t!("messages.hello","name" => "world", locale => "zh-CN"));

    log::warn!(
        "IPv4支持, 端口: {}, http://localhost:{}: 用于接口和局域网发现",
        config.v4port,
        config.v4port
    );
    log::warn!(
        "IPv6支持, 端口: {}, http://[::1]:{}: 用于接口",
        config.v6port,
        config.v4port
    );
    tokio::fs::create_dir_all(DIR_PATH_PLAYER_NBT).await.err();
    tokio::fs::create_dir_all(DIR_PATH_WORLD).await.err();
    tokio::fs::create_dir_all(DIR_PATH_SQLITE).await.err();

    //单经济
    // 多经济
    let single_money_sqlite = SingleMoneySqlite::init(200).await.start();

    let multi_money_sqlite = MultiMoneySqlite::default();
    multi_money_sqlite.init().await;

    let token = config.token.clone();
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(server.clone()))
            .app_data(web::Data::new(players.clone()))
            .app_data(web::Data::new(token.clone()))
            .app_data(web::Data::new(single_money_sqlite.clone()))
            .app_data(web::Data::new(multi_money_sqlite.clone()))
            .service(
                web::scope("/api/pe")
                    .configure(player_config)
                    .configure(chatserver_config)
                    .configure(world_config),
            )
            .service(
                web::scope("/api")
                    .configure(money_config)
            ) 
    })
    .workers(2)
    .bind(("127.0.0.1", config.v4port))?
    .bind(("[::1]", config.v6port))?
    .run()
    .await
}
