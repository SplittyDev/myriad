use std::{io::{BufWriter, Write}, net::TcpStream};
use irc_rust::{Message, MessageBuilder};

use super::server_query::ServerQuery;

pub enum Action {
    Pong,
}

impl Action {
    pub fn dispatch(&self, query: &ServerQuery, writer: &mut BufWriter<TcpStream>) {
        let mut send = |message: Message| {
            println!("[Dispatch] {}", message);
            let msg = format!("{}\r\n", message.to_string());
            if let Err(err) = writer.write_all(msg.as_ref()) {
                println!("[Dispatch] Error: {}", err);
            }
        };

        match self {
            Action::Pong => {
                let host = query.server_host();
                let message = MessageBuilder::new("PONG").param(host).build();
                send(message);
            }
        }
    }
}