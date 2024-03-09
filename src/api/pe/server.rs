use std::collections::HashMap;

use actix::prelude::*;
use rand::{rngs::ThreadRng, Rng};

#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

#[derive(Debug)]
pub struct ChatServer {
    sessions: HashMap<usize, Recipient<Message>>,
    rng: ThreadRng,
}

impl Actor for ChatServer {
    type Context = Context<Self>;
}

impl ChatServer {
    pub fn new() -> ChatServer {
        ChatServer {
            sessions: HashMap::new(),
            rng: rand::thread_rng(),
        }
    }
}

impl ChatServer {
    fn send_message(&self, message: &str, skip_id: usize) {
        for (id, _) in &self.sessions {
            if *id != skip_id {
                if let Some(addr) = self.sessions.get(&id) {
                    addr.do_send(Message(message.to_owned()));
                }
            }
        }
    }
}



#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<Message>,
}

impl Handler<Connect> for ChatServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        let id = self.rng.gen::<usize>();
        self.sessions.insert(id, msg.addr);
        id
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessage {
    pub id: usize,
    pub msg: String,
}
impl Handler<ClientMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
        self.send_message(msg.msg.as_str(), msg.id);
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct BroadcastMessage {
    pub msg: String,
}

impl Handler<BroadcastMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: BroadcastMessage, _: &mut Context<Self>) {
        self.send_message(msg.msg.as_str(), 0);
    }
}
