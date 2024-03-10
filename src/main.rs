use std::collections::HashMap;

use actix::*;

use actix_web::{
    dev::ResourcePath, middleware::Logger, web::{self, Json, Path}, App, Error, HttpRequest, HttpResponse, HttpResponseBuilder, HttpServer
};
use actix_web_actors::ws;

mod api;

use api::pe::{
    player::{Player, PlayerJoin, PlayerLeft, PlayersGet},
    server, session,
};
use serde::Deserialize;
use serde_json::json;

use crate::{api::pe::player::PlayerManager, config::ServerConfig};

#[derive(Deserialize,Debug)]
struct ServerName{
    server_name:String
}
async fn ws_route(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<server::ChatServer>>,
    players: web::Data<Addr<PlayerManager>>,
    server_name: web::Query<ServerName>
) -> Result<HttpResponse, Error> {
    let name=&server_name.server_name;
    ws::start(
        session::WsSession {
            id: 0,
            name: name.to_string(),
            addr: srv.get_ref().clone(),
            playermanager: players.get_ref().clone(),
        },
        &req,
        stream,
    )
}

async fn post_chat(
    player: web::Json<Player>,
    srv: web::Data<Addr<server::ChatServer>>,
) -> HttpResponseBuilder {
    let msg = server::BroadcastMessage {
        msg: json!(player).to_string(),
    };
    let _ = srv.as_ref().clone().do_send(msg);
    // HttpResponseBuilder
    HttpResponse::Ok()
}
async fn player_join(
    player: web::Json<Player>,
    players: web::Data<Addr<PlayerManager>>,
) -> HttpResponseBuilder {
    let player_manager = players.as_ref().clone();
    let player_join = PlayerJoin { player: player.0 };
    player_manager.do_send(player_join);
    HttpResponse::Ok()
}

async fn player_left(
    player: web::Json<Player>,
    players: web::Data<Addr<PlayerManager>>,
) -> HttpResponseBuilder {
    let player_manager = players.as_ref().clone();
    let player_join = PlayerLeft { player: player.0 };
    player_manager.do_send(player_join);
    HttpResponse::Ok()
}

async fn players_get(players: web::Data<Addr<PlayerManager>>) -> Json<Vec<String>> {
    let this = players.as_ref().clone();
    let msg = PlayersGet {};
    let players = this.send(msg).await;
    match players {
        Ok(players) => Json(players),
        Err(_) => Json(vec!["".to_string()]),
    }
}

mod config;

use rust_i18n::t;

rust_i18n::i18n!("locales");

#[macro_use]
extern crate log;
extern crate env_logger;

//初始化日志输出
fn init_log() {
    use chrono::Local;
    use std::io::Write;

    let env = env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "warn");
    let mut builder = env_logger::Builder::from_env(env);
    println!("builder = {:?}", builder);
    builder
        .format(|buf, record| {
            let level = { buf.default_level_style(record.level()) };
            write!(buf, "{}", format_args!("{:<5}", level));
            writeln!(
                buf,
                " {} {} [{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.module_path().unwrap_or("<unnamed>"),
                &record.args()
            )
        })
        .init();
    info!("env_logger initialized.");
}

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    init_log();
    // t!("hello", locale = "zh-CN");
    let config = ServerConfig::default();

    // start chat server actor
    let server = server::ChatServer::new().start();
    let players = PlayerManager::new().start();

    // println!("{}", t!("messages.hello","name" => "world", locale => "zh-CN"));

    log::warn!(
        "IPv4支持, 端口: {}, http://localhost:{} : 用于接口和局域网发现",
        config.v4port,
        config.v4port
    );
    log::warn!(
        "IPv6支持, 端口: {}, http://[::1]:{}     : 用于接口",
        config.v6port,
        config.v4port
    );
    // log::info!("IPv6 supported, port: {}: Used for api",config.v6port);
    // log::info!("IPv4 supported, port: {}: Used for api and LAN discovery",config.v4port);
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(server.clone()))
            .app_data(web::Data::new(players.clone()))
            .service(
                web::scope("/api/pe")
                    .route("/player/chat", web::post().to(post_chat))
                    .route("/ws", web::get().to(ws_route))
                    .route("/player/join", web::post().to(player_join))
                    .route("/player/left", web::post().to(player_left))
                    .route("/player/get", web::get().to(players_get))
                    .wrap(Logger::default()),
            )
    })
    .workers(2)
    .bind(("127.0.0.1", config.v4port))?
    .bind(("[::1]", config.v6port))?
    .run()
    .await
}
