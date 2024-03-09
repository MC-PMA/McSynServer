use actix::*;

use actix_web::{
    middleware::Logger,
    web::{self, Json, Path},
    App, Error, HttpRequest, HttpResponse, HttpResponseBuilder, HttpServer,
};
use actix_web_actors::ws;

mod api;

use api::pe::{
    player::{Player, PlayerJoin, PlayerLeft, PlayersGet},
    server, session,
};
use serde_json::json;

use crate::api::pe::player::PlayerManager;
async fn ws_route(
    req: HttpRequest,
    stream: web::Payload,
    path: Path<String>,
    srv: web::Data<Addr<server::ChatServer>>,
    players: web::Data<Addr<PlayerManager>>
) -> Result<HttpResponse, Error> {
    ws::start(
        session::WsSession {
            id: 0,
            name: path.to_string(),
            addr: srv.get_ref().clone(),
            playermanager:players.get_ref().clone()
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
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // start chat server actor
    let server = server::ChatServer::new().start();
    let players = PlayerManager::new().start();

    log::info!("starting HTTP server api at http://localhost:2000");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(server.clone()))
            .app_data(web::Data::new(players.clone()))
            .service(
                web::scope("/api/pe")
                    .route("/player/chat", web::post().to(post_chat))
                    .route("/ws/{server_name}", web::get().to(ws_route))
                    .route("/player/join", web::post().to(player_join))
                    .route("/player/left", web::post().to(player_left))
                    .route("/player/get", web::get().to(players_get))
                    .wrap(Logger::default()),
            )
    })
    .workers(2)
    .bind(("127.0.0.1", 2000))?
    .run()
    .await
}
