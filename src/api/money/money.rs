/// This module provides functionality related to money in the API.
use crate::{
    api::pe::player_api::ResponseMessage,
    sql::multi_economy::{MultiMoney, MultiMoneySqlite, MultiPlayerMoney},
};
use actix_web::{
    web::{self, Json},
    HttpResponse,
};

pub fn money_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/money")
            // 经济体
            .route("/addmoney/{token_path}", web::post().to(add_money))
            .route("/delete_money/{token_path}", web::delete().to(delete_money))
            .route(
                "/update_money/{money_name}/{token}",
                web::put().to(update_money),
            )
            .route(
                "/get_money_check/{money_name}",
                web::get().to(get_money_check),
            )
            .route("/get_all_money/{path_token}", web::get().to(get_all_money))
            //  玩家经济
            .route("/add_player_init/{path_token}", web::post().to(add_player_init))
            .route(
                "/get_player_money/{path_token}/{player_uuid}/{money}",
                web::get().to(get_player_money),
            )
            .route("/update_player_balance/{path_token}", web::put().to(update_player_money))
            .route("/add_player_balance/{path_token}", web::put().to(add_player_balance))
            .route("/reduce_player_balance/{path_token}", web::put().to(reduce_player_balance))
            .route(
                "/transfer_player_balance/{path_token}/{player1_uuid}/{player2_uuid}/{balance}/{money}",
                web::put().to(transfer_player_balance),
            ),
    );
}

//经济体API

// 添加一种经济体
pub async fn add_money(
    data: web::Data<MultiMoneySqlite>,
    token_path: web::Path<String>,
    multi_economy: web::Json<MultiMoney>,
    token: web::Data<String>,
) -> HttpResponse {
    if token_path.as_str() != token.as_str() {
        return HttpResponse::Unauthorized().json(ResponseMessage {
            r#type: "error".to_string(),
            message: "token错误".to_string(),
        });
    }
    HttpResponse::Ok().json(data.add_money(multi_economy.0))
}

// 删除一种经济体
pub async fn delete_money(
    data: web::Data<MultiMoneySqlite>,
    token_path: web::Path<String>,
    money: web::Json<MultiMoney>,
    token: web::Data<String>,
) -> HttpResponse {
    if token_path.as_str() != token.as_str() {
        return HttpResponse::Unauthorized().json(ResponseMessage {
            r#type: "error".to_string(),
            message: "token错误".to_string(),
        });
    }
    HttpResponse::Ok().json(data.delete_money_key(money.0))
}

// 修改经济体名
pub async fn update_money(
    data: web::Data<MultiMoneySqlite>,
    token: web::Data<String>,
    multi_economy: web::Json<MultiMoney>,
    path_str: web::Path<(String, String)>,
) -> HttpResponse {
    if path_str.1.as_str() != token.as_str() {
        return HttpResponse::Unauthorized().json(ResponseMessage {
            r#type: "error".to_string(),
            message: "token错误".to_string(),
        });
    }
    HttpResponse::Ok().json(data.update_money_key(multi_economy.0, path_str.0.clone()))
}

// 检查某个经济体是否存在
pub async fn get_money_check(
    data: web::Data<MultiMoneySqlite>,
    money_name: web::Path<String>,
) -> HttpResponse {
    match data.get_money_key(money_name.clone()) {
        true => HttpResponse::Ok().json(ResponseMessage {
            r#type: "success".to_string(),
            message: "存在".to_string(),
        }),
        false => HttpResponse::Ok().json(ResponseMessage {
            r#type: "error".to_string(),
            message: "不存在".to_string(),
        }),
    }
}

// 获取所有经济体
pub async fn get_all_money(
    data: web::Data<MultiMoneySqlite>,
    token_path: web::Path<String>,
    token: web::Data<String>,
) -> Json<Vec<String>> {
    if token_path.as_str() != token.as_str() {
        return Json(vec![]);
    }
    match data.get_all_money() {
        Ok(multi_moneys) => {
            let mut money = Vec::new();
            for multi_money in multi_moneys {
                money.push(multi_money.money);
            }
            Json(money)
        }
        Err(_e) => {
            //查询失败
            Json(vec![])
        }
    }
}

//初始化玩家经济
pub async fn add_player_init(
    data: web::Data<MultiMoneySqlite>,
    token_path: web::Path<String>,
    player_money: web::Json<MultiPlayerMoney>,
    token: web::Data<String>,
) -> HttpResponse {
    if token_path.as_str() != token.as_str() {
        return HttpResponse::Unauthorized().json(ResponseMessage {
            r#type: "error".to_string(),
            message: "token错误".to_string(),
        });
    }
    HttpResponse::Ok().json(data.init_pl_money(player_money.0))
}

// 获取玩家经济余额
pub async fn get_player_money(
    data: web::Data<MultiMoneySqlite>,
    path_data: web::Path<(String, String, String)>,
    token: web::Data<String>,
) -> HttpResponse {
    if path_data.0.as_str() != token.as_str() {
        return HttpResponse::Unauthorized().json(ResponseMessage {
            r#type: "error".to_string(),
            message: "token错误".to_string(),
        });
    }
    match data.get_pl_money(path_data.1.clone(), path_data.2.clone()) {
        Ok(multi_moneys) => HttpResponse::Ok().json(ResponseMessage {
            r#type: "success".to_string(),
            message: multi_moneys.to_string(),
        }),
        Err(_e) => {
            //查询失败
            HttpResponse::InternalServerError().json(ResponseMessage {
                r#type: "error".to_string(),
                message: "查询失败".to_string(),
            })
        }
    }
}

// 更新玩家经济余额
pub async fn update_player_money(
    data: web::Data<MultiMoneySqlite>,
    path_token: web::Path<String>,
    player_money: web::Json<MultiPlayerMoney>,
    token: web::Data<String>,
) -> HttpResponse {
    if path_token.as_str() != token.as_str() {
        return HttpResponse::Unauthorized().json(ResponseMessage {
            r#type: "error".to_string(),
            message: "token错误".to_string(),
        });
    }

    HttpResponse::Ok().json(data.update_pl_money(player_money.0))
}

// 增加玩家经济
pub async fn add_player_balance(
    data: web::Data<MultiMoneySqlite>,
    token_path: web::Path<String>,
    player_money: web::Json<MultiPlayerMoney>,
    token: web::Data<String>,
) -> HttpResponse {
    if token_path.as_str() != token.as_str() {
        return HttpResponse::Unauthorized().json(ResponseMessage {
            r#type: "error".to_string(),
            message: "token错误".to_string(),
        });
    }
    HttpResponse::Ok().json(data.add_pl_money(player_money.0))
}

// 减少玩家经济
pub async fn reduce_player_balance(
    data: web::Data<MultiMoneySqlite>,
    token_path: web::Path<String>,
    player_money: web::Json<MultiPlayerMoney>,
    token: web::Data<String>,
) -> HttpResponse {
    if token_path.as_str() != token.as_str() {
        return HttpResponse::Unauthorized().json(ResponseMessage {
            r#type: "error".to_string(),
            message: "token错误".to_string(),
        });
    }
    HttpResponse::Ok().json(data.reduce_pl_money(player_money.0))
}

// 玩家经济转账
pub async fn transfer_player_balance(
    data: web::Data<MultiMoneySqlite>,
    path_data: web::Path<(String, String, String, i32, String)>,
    token: web::Data<String>,
) -> HttpResponse {
    if path_data.0.as_str() != token.as_str() {
        return HttpResponse::Unauthorized().json(ResponseMessage {
            r#type: "error".to_string(),
            message: "token错误".to_string(),
        });
    }

    HttpResponse::Ok().json(data.transfer_pl_money(
        path_data.1.clone(),
        path_data.2.clone(),
        path_data.3,
        path_data.4.clone(),
    ))
}
