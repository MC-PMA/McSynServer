use std::io::Write;

use actix::*;

use actix_web::{
    web::{self, Json},
    HttpResponse, HttpResponseBuilder,
};

use futures_util::StreamExt;
use log::{debug, info};
use serde::Deserialize;
use serde_json::json;
use tokio::{
    fs::{create_dir_all, File},
    io::{AsyncReadExt, AsyncWriteExt},
};

use crate::SAVE_DIR;

use super::{
    chatserver,
    player::{Player, PlayerJoin, PlayerLeft, PlayerManager, PlayersGet},
};

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

//检查玩家是否在线
pub async fn player_check_online(
    path: web::Path<String>,
    players: web::Data<Addr<PlayerManager>>,
) -> HttpResponseBuilder {
    let player_name = path.as_str();
    let this = players.as_ref().clone();
    let msg = PlayersGet {};
    let players = this.send(msg).await.unwrap();
    for player in players {
        if player == player_name {
            return HttpResponse::Ok();
        } else {
            return HttpResponse::NotFound();
        }
    }
    HttpResponse::NotFound()
}


pub fn player_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/player")
            .route("/chat", web::post().to(post_chat))
            .route("/join", web::post().to(player_join))
            .route("/left", web::post().to(player_left))
            .route("/nbt/{player}/upload", web::post().to(player_nbt_upload))
            .route("/nbt/{player}/get", web::get().to(player_nbt_get))
            .route("/get", web::get().to(players_get))
            .route("/check_online/{player}", web::get().to(player_check_online)),
    );
}

// 接受玩家nbt文件
pub async fn player_nbt_upload(
    //路径参数
    path: web::Path<String>,
    mut binary: web::Payload,
) -> HttpResponseBuilder {
    info!("收到nbt上传请求");
    let player_name = path.as_str();

    let file = format!("{}/{}.nbt", SAVE_DIR, player_name);

    //创建文件夹
    let mut file = File::create(file).await.unwrap();

    //写入文件
    let mut bytes = web::BytesMut::new();
    while let Some(item) = binary.next().await {
        bytes.extend_from_slice(&item.unwrap());
    }
    match file.write_all(&bytes.to_vec()).await {
        Ok(_) => HttpResponse::Ok(),
        Err(err) => {
            return HttpResponse::InternalServerError();
        }
    }
}

// 提供玩家nbt文件
pub async fn player_nbt_get(
    //路径参数
    path: web::Path<String>,
) -> HttpResponse {
    info!("收到nbt获取请求");
    let player_name = path.as_str();
    let file = format!("{}/{}.nbt", SAVE_DIR, player_name);
    let file = File::open(file).await;
    match file {
        Ok(file) => {
            let mut bytes = web::BytesMut::new();
            let mut reader = tokio::io::BufReader::new(file);
            let _ = reader.read_buf(&mut bytes).await.unwrap();
            return HttpResponse::Ok()
                .content_type("application/octet-stream")
                .body(bytes.to_vec());
        }
        Err(err) => {
            return HttpResponse::NotFound().finish();
        }
    }
}
