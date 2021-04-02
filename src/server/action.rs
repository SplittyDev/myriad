use std::{io::{BufWriter, Write}, net::TcpStream};
use irc_rust::{Message, MessageBuilder};

use super::server_query::ServerQuery;
use crate::numerics::*;

const SOFTWARE_NAME: &'static str = env!("CARGO_PKG_NAME");
const SOFTWARE_VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub enum Action {
    Error { code: &'static str },
    Pong { challenge: Option<String> },
    SetNick { nickname: String },
    ChangeNick { prev_nickname: String, nickname: String },
    SetUserAndRealName { username: String, realname: String },
    SendWelcomeSequence,
    Motd,
    Quit { reason: Option<String> },
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
            Action::Pong { challenge } => {
                let mut message = MessageBuilder
                    ::new("PONG");
                if let Some(challenge) = challenge {
                    message = message.param(challenge);
                }
                let message = message.build();
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
                let nickname = query.user().nickname.clone().unwrap();
                let rpl_welcome = MessageBuilder
                    ::new(RPL_WELCOME)
                    .param(&nickname)
                    .trailing(&format!(
                        "Welcome to {servername}, {nickname}",
                        servername=query.server_name(),
                        nickname=query.user().nickname.clone().unwrap(),
                    ))
                    .build();
                let rpl_yourhost = MessageBuilder
                    ::new(RPL_YOURHOST)
                    .param(&nickname)
                    .trailing(&format!(
                        "Your host is Myriad, running version {software_version}",
                        software_version=SOFTWARE_VERSION
                    ))
                    .build();
                let rpl_created = MessageBuilder
                    ::new(RPL_CREATED)
                    .param(&nickname)
                    .trailing(&format!(
                        "This server was created {server_startup_time}",
                        server_startup_time=query.server_startup_time()
                    ))
                    .build();
                let rpl_myinfo = MessageBuilder
                    ::new(RPL_MYINFO)
                    .param(&nickname)
                    .build();
                let rpl_isupport = MessageBuilder
                    ::new(RPL_ISUPPORT)
                    .param(&nickname)
                    .param(&format!("AWAYLEN={}", query.server_config().feat_awaylen))
                    .param(&format!("CASEMAPPING={}", query.server_config().feat_casemap.to_string()))
                    .trailing("are supported by this server")
                    .build();
                let rpl_lusers = MessageBuilder
                    ::new(RPL_LUSERCLIENT)
                    .param(&nickname)
                    .trailing(&format!(
                        "There are {user_count} users and {invisible_count} invisible on 1 server",
                        user_count = query.user_count(),
                        invisible_count = 0
                    ))
                    .build();
                send(rpl_welcome);
                send(rpl_yourhost);
                send(rpl_created);
                // send(rpl_myinfo);
                send(rpl_isupport);
                send(rpl_lusers);
                Action::Motd.dispatch(query, writer);
            }

            Action::Motd => {
                let nickname = query.user().nickname.clone().unwrap();
                let motd_start = MessageBuilder
                    ::new(RPL_MOTDSTART)
                    .param(&nickname)
                    .trailing(&format!("- {} Message of the day - ", query.server_name()))
                    .build();
                let motd = MessageBuilder
                    ::new(RPL_MOTD)
                    .param(&nickname)
                    .trailing(&query.server_config().motd)
                    .build();
                let motd_end = MessageBuilder
                    ::new(RPL_ENDOFMOTD)
                    .param(&nickname)
                    .trailing("End of /MOTD command.")
                    .build();
                send(motd_start);
                send(motd);
                send(motd_end);
            }

            Action::Quit { reason } => {

                // Terminate client
                if query.user_mut().stream.shutdown(std::net::Shutdown::Both).is_ok() {
                    let nickname = query.user().nickname.clone().unwrap_or_default();
                    println!(
                        "[Server] Terminated connection of {nickname}@{host} (QUIT: {reason})",
                        nickname = nickname,
                        host = query.user_host(),
                        reason = reason.as_ref().unwrap_or(&String::new())
                    );
                }
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