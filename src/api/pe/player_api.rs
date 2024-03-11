use actix::*;

use actix_web::{
    web::{self, Json},
    HttpResponse, HttpResponseBuilder,
};

use serde_json::json;

use super::{
    player::{Player, PlayerJoin, PlayerLeft, PlayerManager, PlayersGet},
    chatserver,
};

pub fn player_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/peplayer")
            .route("/chat", web::post().to(post_chat))
            .route("/join", web::post().to(player_join))
            .route("/left", web::post().to(player_left))
            .route("/get", web::get().to(players_get)),
    );
}

pub async fn post_chat(
    player: web::Json<Player>,
    srv: web::Data<Addr<chatserver::ChatServer>>,
) -> HttpResponseBuilder {
    let msg = chatserver::BroadcastMessage {
        msg: json!(player).to_string(),
    };
    let _ = srv.as_ref().clone().do_send(msg);
    // HttpResponseBuilder
    HttpResponse::Ok()
}

pub async fn player_join(
    player: web::Json<Player>,
    players: web::Data<Addr<PlayerManager>>,
) -> HttpResponseBuilder {
    let player_manager = players.as_ref().clone();
    let player_join = PlayerJoin { player: player.0 };
    player_manager.do_send(player_join);
    HttpResponse::Ok()
}

pub async fn player_left(
    player: web::Json<Player>,
    players: web::Data<Addr<PlayerManager>>,
) -> HttpResponseBuilder {
    let player_manager = players.as_ref().clone();
    let player_join = PlayerLeft { player: player.0 };
    player_manager.do_send(player_join);
    HttpResponse::Ok()
}

pub async fn players_get(players: web::Data<Addr<PlayerManager>>) -> Json<Vec<String>> {
    let this = players.as_ref().clone();
    let msg = PlayersGet {};
    let players = this.send(msg).await;
    match players {
        Ok(players) => Json(players),
        Err(_) => Json(vec!["".to_string()]),
    }
}
