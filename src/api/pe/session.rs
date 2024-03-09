use std::time::{Duration, Instant};

use actix::prelude::*;
use actix_web_actors::ws;
use log::info;

use crate::api::pe::player::PlayersRemoveByServer;

use super::{player::PlayerManager, server};

#[derive(Debug)]
pub struct WsSession {
    pub id: usize,

    pub name: String,
    /// Chat server
    pub addr: Addr<server::ChatServer>,
    pub playermanager: Addr<PlayerManager>,
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("{} 已连接", self.name);
        // across all routes within application
        let addr = ctx.address();
        self.addr
            .send(server::Connect {
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => act.id = res,
                    // something is wrong with chat server
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        let server_name=&self.name;
        self.playermanager.do_send(PlayersRemoveByServer {
            server:server_name.to_string(),
        });
        info!("{} 已断开", server_name);
        Running::Stop
    }
}

/// Handle messages from chat server, we simply send it to peer websocket
impl Handler<server::Message> for WsSession {
    type Result = ();

    fn handle(&mut self, msg: server::Message, ctx: &mut Self::Context) {
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
            ws::Message::Ping(msg) => {}
            ws::Message::Pong(_) => {}
            ws::Message::Text(text) => {
                let msg: String = text.trim().to_owned();
                self.addr
                    .do_send(server::ClientMessage { id: self.id, msg })
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
