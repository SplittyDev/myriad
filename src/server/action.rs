use std::{io::{BufWriter, Write}, net::TcpStream};
use irc_rust::{Message, MessageBuilder};
use guard::guard;

use super::server_query::ServerQuery;

pub enum Action {
    Error { code: &'static str },
    Pong,
    SetNick { nickname: String },
    ChangeNick { prev_nickname: String, nickname: String },
    SetUserAndRealName { username: String, realname: String },
}

impl Action {
    pub fn dispatch(&self, query: &mut ServerQuery, writer: &mut BufWriter<TcpStream>) {
        let mut send = |message: Message| {
            println!("[Dispatch] {}", message);
            let msg = format!("{}\r\n", message.to_string());
            if let Err(err) = writer.write_all(msg.as_ref()) {
                println!("[Dispatch] Error: {}", err);
            }
        };

        let server_host = query.server_host();
        let user_host = query.user_host();

        match self {

            // Send PING response
            Action::Pong => {
                let message = MessageBuilder
                    ::new("PONG")
                    .param(server_host)
                    .build();
                send(message);
            }

            Action::SetNick { nickname } => {
                println!("[Server] NICK [client={}, new_nick={}]", user_host, nickname);
                query.user_mut().nickname = Some(nickname.clone());
            }

            Action::ChangeNick { prev_nickname, nickname } => {
                println!("[Server] NICK [client={}, from={}, to={}]", user_host, prev_nickname, nickname);
                query.user_mut().nickname = Some(nickname.clone());
            }

            Action::SetUserAndRealName { username, realname } => {
                unimplemented!()
            }

            Action::Error { code } => {
                let message = MessageBuilder
                    ::new(code)
                    .prefix(server_host, None, Some(user_host))
                    .build();
                send(message);
            }
        }
    }
}