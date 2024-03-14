use actix_web::{web, HttpResponse, HttpResponseBuilder};
use futures_util::StreamExt;
use log::info;
use tokio::{fs::File, io::{AsyncReadExt, AsyncWriteExt}};

use crate::DIR_PATH_WORLD;

pub fn world_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/world")
            .route("/{world}/upload/{token_path}", web::post().to(world_upload))
            .route("/{world}/get/{token_path}", web::get().to(world_nbt_get)),
    );
}

// 世界存档上传
pub async fn world_upload(
    //路径参数
    path_world: web::Path<String>,
    mut binary: web::Payload,
    token_path: web::Path<String>,
    token: web::Data<String>,
) -> HttpResponseBuilder {
    if token_path.as_str() != token.as_str() {
        return HttpResponse::Unauthorized();
    }
    info!("收到世界存档上传请求");
    let player_name = path_world.as_str();

    let file = format!("{}/{}.zip", DIR_PATH_WORLD, player_name);

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

// 世界存档获取
pub async fn world_nbt_get(
    //路径参数
    path_world: web::Path<String>,
    token_path: web::Path<String>,
    token: web::Data<String>,
) -> HttpResponse {
    if token_path.as_str() != token.as_str() {
        return HttpResponse::NotFound().finish();
    }
    info!("收到世界存档获取请求");
    let player_name = path_world.as_str();
    let file = format!("{}/{}.zip", DIR_PATH_WORLD, player_name);
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
