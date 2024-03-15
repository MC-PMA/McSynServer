use actix::prelude::*;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use log::info;
use serde::Deserialize;

use crate::api::pe::player::PlayersRemoveByServer;

use super::{chatserver, player::PlayerManager};

pub fn chatserver_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/ws").route(web::get().to(ws_route)));
}

#[derive(Deserialize, Debug)]
pub struct ServerName {
    server_name: String,
}
pub async fn ws_route(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<chatserver::ChatServer>>,
    players: web::Data<Addr<PlayerManager>>,
    server_name: web::Query<ServerName>,
) -> Result<HttpResponse, Error> {
    let name = &server_name.server_name;
    if name.is_empty() {
        return Ok(HttpResponse::BadRequest().finish());
    }
    ws::start(
        WsSession {
            id: 0,
            name: name.to_string(),
            addr: srv.get_ref().clone(),
            playermanager: players.get_ref().clone(),
        },
        &req,
        stream,
    )
}

#[derive(Debug)]
pub struct WsSession {
    pub id: usize,
 
    pub name: String,
    /// Chat server
    pub addr: Addr<chatserver::ChatServer>,
    pub playermanager: Addr<PlayerManager>,
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("{} 已连接", self.name);
        // across all routes within application
        let addr = ctx.address();
        self.addr
            .send(chatserver::Connect {
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res: Result<usize, MailboxError>, act: &mut WsSession, ctx: &mut ws::WebsocketContext<WsSession>| {
                match res {
                    Ok(res) => act.id = res,
                    // something is wrong with chat server
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        let server_name = &self.name;
        self.playermanager.do_send(PlayersRemoveByServer {
            server: server_name.to_string(),
        });
        info!("{} 已断开", server_name);
        Running::Stop
    }
}

/// Handle messages from chat server, we simply send it to peer websocket
impl Handler<chatserver::Message> for WsSession {
    type Result = ();

    fn handle(&mut self, msg: chatserver::Message, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

/// WebSocket message handler
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };
        log::debug!("WEBSOCKET MESSAGE: {msg:?}");
        match msg {
            ws::Message::Ping(_msg) => {}
            ws::Message::Pong(_) => {}
            ws::Message::Text(text) => {
                let msg: String = text.trim().to_owned();
                self.addr
                    .do_send(chatserver::ClientMessage { id: self.id, msg })
            }
            ws::Message::Binary(_) => println!("Unexpected binary"),
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
        }
    }
}
