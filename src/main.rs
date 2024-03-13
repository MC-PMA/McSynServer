use actix::*;

use actix_files::Directory;
use actix_web::{
    web::{self},
    App, HttpServer,
};

mod api;
use api::pe::chatserver;

use crate::{
    api::pe::{player::PlayerManager, player_api::player_config, session::chatserver_config},
    config::ServerConfig,
};

mod config;

use rust_i18n::t;

rust_i18n::i18n!("locales");

use crate::config::init_log;
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

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(server.clone()))
            .app_data(web::Data::new(players.clone()))
            .service(
                web::scope("/api/pe")
                    .configure(player_config)
                    .configure(chatserver_config),
            )
    })
    .workers(2)
    .bind(("127.0.0.1", config.v4port))?
    .bind(("[::1]", config.v6port))?
    .run()
    .await
}
