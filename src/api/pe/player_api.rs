use actix::*;

use actix_web::{
    web::{self, Json},
    HttpResponse,
};

use futures_util::StreamExt;
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};

use crate::DIR_PATH_PLAYER_NBT;

use super::{
    chatserver,
    player::{Player, PlayerJoin, PlayerLeft, PlayerManager, PlayersGet},
};

pub async fn post_chat(
    player: web::Json<Player>,
    srv: web::Data<Addr<chatserver::ChatServer>>,
    token_path: web::Path<String>,
    token: web::Data<String>,
) -> HttpResponse {
    if token_path.as_str() != token.as_str() {
        return HttpResponse::Unauthorized().json(ResponseMessage {
            r#type: "error".to_string(),
            message: "token错误".to_string(),
        });
    }
    let msg = chatserver::BroadcastMessage {
        msg: json!(player).to_string(),
    };
    let _ = srv.as_ref().clone().do_send(msg);
    // HttpResponse
    HttpResponse::Ok().json(ResponseMessage {
        r#type: "success".to_string(),
        message: "消息发送成功".to_string(),
    })
}

pub async fn player_join(
    player: web::Json<Player>,
    players: web::Data<Addr<PlayerManager>>,
    token_path: web::Path<String>,
    token: web::Data<String>,
) -> HttpResponse {
    if token_path.as_str() != token.as_str() {
        return HttpResponse::Unauthorized().json(ResponseMessage {
            r#type: "error".to_string(),
            message: "token错误".to_string(),
        });
    }
    let player_manager = players.as_ref().clone();
    let player_join = PlayerJoin { player: player.0 };
    player_manager.do_send(player_join);
    HttpResponse::Ok().json(ResponseMessage {
        r#type: "success".to_string(),
        message: "玩家加入".to_string(),
    })
}

pub async fn player_left(
    player: web::Json<Player>,
    players: web::Data<Addr<PlayerManager>>,
    token_path: web::Path<String>,
    token: web::Data<String>,
) -> HttpResponse {
    if token_path.as_str() != token.as_str() {
        return HttpResponse::Unauthorized().json(ResponseMessage {
            r#type: "error".to_string(),
            message: "token错误".to_string(),
        });
    }
    let player_manager = players.as_ref().clone();
    let player_join = PlayerLeft { player: player.0 };
    player_manager.do_send(player_join);
    HttpResponse::Ok().json(ResponseMessage {
        r#type: "success".to_string(),
        message: "玩家离开".to_string(),
    })
}

pub async fn players_get(
    players: web::Data<Addr<PlayerManager>>,
    token: web::Data<String>,
    token_path: web::Path<String>,
) -> Json<Vec<String>> {
    if token_path.as_str() != token.as_str() {
        return Json(vec!["".to_string()]);
    }
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
) -> HttpResponse {
    let player_name = path.as_str();
    let this = players.as_ref().clone();
    let msg = PlayersGet {};
    let players = this.send(msg).await.unwrap();
    for player in players {
        if player == player_name {
            return HttpResponse::Ok().json(ResponseMessage {
                r#type: "success".to_string(),
                message: "玩家在线".to_string(),
            });
        } else {
            return HttpResponse::NotFound().json(ResponseMessage {
                r#type: "error".to_string(),
                message: "玩家不在线".to_string(),
            });
        }
    }
    HttpResponse::NotFound().json(ResponseMessage {
        r#type: "error".to_string(),
        message: "玩家不在线".to_string(),
    })
}

pub fn player_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/player")
            .route("/chat/{token_path}", web::post().to(post_chat))
            .route("/join/{token_path}", web::post().to(player_join))
            .route("/left/{token_path}", web::post().to(player_left))
            .route(
                "/nbt/{player}/upload/{token_path}",
                web::post().to(player_nbt_upload),
            )
            .route(
                "/nbt/{player}/get/{token_path}",
                web::get().to(player_nbt_get),
            )
            .route("/get/{token_path}", web::get().to(players_get))
            .route("/check_online/{player}", web::get().to(player_check_online)),
    );
}

// 接受玩家nbt文件
pub async fn player_nbt_upload(
    //路径参数
    path: web::Path<String>,
    mut binary: web::Payload,
    token_path: web::Path<String>,
    token: web::Data<String>,
) -> HttpResponse {
    if token_path.as_str() != token.as_str() {
        return HttpResponse::Unauthorized().json(ResponseMessage {
            r#type: "error".to_string(),
            message: "token错误".to_string(),
        });
    }
    let player_name = path.as_str();

    let file = format!("{}/{}.nbt", DIR_PATH_PLAYER_NBT, player_name);

    //创建文件夹
    let mut file = File::create(file).await.unwrap();

    //写入文件
    let mut bytes = web::BytesMut::new();
    while let Some(item) = binary.next().await {
        bytes.extend_from_slice(&item.unwrap());
    }
    match file.write_all(&bytes.to_vec()).await {
        Ok(_) => HttpResponse::Ok().json(ResponseMessage {
            r#type: "success".to_string(),
            message: "文件写入成功".to_string(),
        }),
        Err(_err) => {
            return HttpResponse::InternalServerError().json(ResponseMessage {
                r#type: "error".to_string(),
                message: "文件写入失败".to_string(),
            });
        }
    }
}

// 提供玩家nbt文件
pub async fn player_nbt_get(
    //路径参数
    path: web::Path<String>,
    token_path: web::Path<String>,
    token: web::Data<String>,
) -> HttpResponse {
    if token_path.as_str() != token.as_str() {
        return HttpResponse::NotFound().json(ResponseMessage {
            r#type: "error".to_string(),
            message: "token错误".to_string(),
        });
    }
    let player_name = path.as_str();
    let file = format!("{}/{}.nbt", DIR_PATH_PLAYER_NBT, player_name);
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
        Err(_err) => {
            return HttpResponse::NotFound().json(ResponseMessage {
                r#type: "error".to_string(),
                message: "文件不存在".to_string(),
            });
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ResponseMessage {
    pub r#type: String,
    pub message: String,
}
