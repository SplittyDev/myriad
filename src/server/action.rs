use std::{io::{BufWriter, Write}, net::TcpStream};
use irc_rust::{Message, MessageBuilder};

use super::server_query::ServerQuery;
use crate::numerics::*;

const SOFTWARE_NAME: &'static str = env!("CARGO_PKG_NAME");
const SOFTWARE_VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub enum Action {
    Error { code: &'static str },
    Pong,
    SetNick { nickname: String },
    ChangeNick { prev_nickname: String, nickname: String },
    SetUserAndRealName { username: String, realname: String },
    SendWelcomeSequence,
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
                println!("[Server] NICK [client={}, new_nick='{}']", user_host, nickname);
                query.user_mut().nickname = Some(nickname.clone());
            }

            Action::ChangeNick { prev_nickname, nickname } => {
                println!("[Server] NICK [client={}, from='{}', to='{}']", user_host, prev_nickname, nickname);
                query.user_mut().nickname = Some(nickname.clone());
            }

            Action::SetUserAndRealName { username, realname } => {
                println!("[Server] USER [client={}, username='{}', realname='{}']", user_host, username, realname);
                query.user_mut().username = Some(username.clone());
                query.user_mut().realname = Some(realname.clone());
                Action::SendWelcomeSequence.dispatch(query, writer);
            }

            Action::SendWelcomeSequence => {
                println!("[Server] #welcome[client={}]", user_host);
                let rpl_welcome = MessageBuilder
                    ::new(RPL_WELCOME)
                    .trailing(&format!(
                        "Welcome to {servername}, {nickname}",
                        servername=query.server_name(),
                        nickname=query.user().nickname.clone().unwrap(),
                    ))
                    .build();
                let rpl_yourhost = MessageBuilder
                    ::new(RPL_YOURHOST)
                    .trailing(&format!(
                        "Your host is Myriad, running version {software_version}",
                        software_version=SOFTWARE_VERSION
                    ))
                    .build();
                let rpl_created = MessageBuilder
                    ::new(RPL_CREATED)
                    .trailing(&format!(
                        "This server was created {server_startup_time}",
                        server_startup_time=query.server_startup_time()
                    ))
                    .build();
                send(rpl_welcome);
                send(rpl_yourhost);
                send(rpl_created);
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