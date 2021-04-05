use guard::guard;
use irc_rust::{Message, MessageBuilder};
use itertools::Itertools;
use std::{
    io::{BufWriter, Write},
    net::TcpStream,
};

use super::server_query::ServerQuery;
use crate::{
    models::{ChannelRef, User},
    numerics::*,
};

const SOFTWARE_VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub enum Action {
    Error {
        code: &'static str,
    },
    Pong {
        challenge: Option<String>,
    },
    SetNick {
        nickname: String,
    },
    ChangeNick {
        prev_nickname: String,
        nickname: String,
    },
    SetUserAndRealName {
        username: String,
        realname: String,
    },
    SendWelcomeSequence,
    Motd,
    Quit {
        reason: Option<String>,
    },
    Join {
        channels: Vec<ChannelRef>,
    },
    JoinInform {
        channel: String,
    },
    PrivateMessage {
        message: String,
        users: Vec<String>,
        channels: Vec<String>,
    },
    PrivateMessageUser {
        message: String,
        from_nickname: String,
    },
    PrivateMessageChannel {
        message: String,
        channel: String,
        from_nickname: String,
    },
}

impl Action {
    pub fn _dispatch_multi_by_user_ref(&self, root_query: &mut ServerQuery, users: &[&User]) {
        for user in users {
            let mut query = ServerQuery::new(root_query.server_mut(), user.client_id);
            let mut writer = {
                let writer = user
                    .stream
                    .try_clone()
                    .ok()
                    .map(|stream| BufWriter::new(stream));
                guard!(let Some(mut writer) = writer else { return });
                writer
            };
            self.dispatch(&mut query, &mut writer);
        }
    }

    pub fn dispatch_for_client_id(&self, root_query: &mut ServerQuery, client: u64) {
        let mut query = ServerQuery::new(root_query.server_mut(), client);
        let mut writer = {
            let writer = query
                .user()
                .stream
                .try_clone()
                .ok()
                .map(|stream| BufWriter::new(stream));
            guard!(let Some(mut writer) = writer else { return });
            writer
        };
        self.dispatch(&mut query, &mut writer);
    }

    pub fn dispatch_multi_by_client_id(&self, root_query: &mut ServerQuery, clients: &[u64]) {
        for client in clients {
            self.dispatch_for_client_id(root_query, *client);
        }
    }

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
        let client_id = query.user().client_id;

        match self {
            // Send PING response
            Action::Pong { challenge } => {
                let mut message = MessageBuilder::new("PONG");
                if let Some(challenge) = challenge {
                    message = message.param(challenge);
                }
                let message = message.build();
                send(message);
            }

            Action::SetNick { nickname } => {
                println!(
                    "[Server] NICK [client={}, new_nick='{}']",
                    user_host, nickname
                );
                query.user_mut().nickname = Some(nickname.clone());
            }

            Action::ChangeNick {
                prev_nickname,
                nickname,
            } => {
                println!(
                    "[Server] NICK [client={}, from='{}', to='{}']",
                    user_host, prev_nickname, nickname
                );
                query.user_mut().nickname = Some(nickname.clone());
            }

            Action::SetUserAndRealName { username, realname } => {
                println!(
                    "[Server] USER [client={}, username='{}', realname='{}']",
                    user_host, username, realname
                );
                query.user_mut().username = Some(username.clone());
                query.user_mut().realname = Some(realname.clone());
                Action::SendWelcomeSequence.dispatch(query, writer);
            }

            Action::SendWelcomeSequence => {
                println!("[Server] #welcome[client={}]", user_host);
                let nickname = query.user().nickname.clone().unwrap();
                let rpl_welcome = MessageBuilder::new(RPL_WELCOME)
                    .param(&nickname)
                    .trailing(&format!(
                        "Welcome to {servername}, {nickname}",
                        servername = query.server_name(),
                        nickname = query.user().nickname.clone().unwrap(),
                    ))
                    .build();
                let rpl_yourhost = MessageBuilder::new(RPL_YOURHOST)
                    .param(&nickname)
                    .trailing(&format!(
                        "Your host is Myriad, running version {software_version}",
                        software_version = SOFTWARE_VERSION
                    ))
                    .build();
                let rpl_created = MessageBuilder::new(RPL_CREATED)
                    .param(&nickname)
                    .trailing(&format!(
                        "This server was created {server_startup_time}",
                        server_startup_time = query.server_startup_time()
                    ))
                    .build();
                // TODO: RPL_MYINFO
                let _rpl_myinfo = MessageBuilder::new(RPL_MYINFO).param(&nickname).build();
                let rpl_isupport = MessageBuilder::new(RPL_ISUPPORT)
                    .param(&nickname)
                    .param(&format!("AWAYLEN={}", query.server_config().feat_awaylen))
                    .param(&format!(
                        "CASEMAPPING={}",
                        query.server_config().feat_casemap.to_string()
                    ))
                    .trailing("are supported by this server")
                    .build();
                let rpl_lusers = MessageBuilder::new(RPL_LUSERCLIENT)
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
                let motd_start = MessageBuilder::new(RPL_MOTDSTART)
                    .param(&nickname)
                    .trailing(&format!("- {} Message of the day - ", query.server_name()))
                    .build();
                let motd = MessageBuilder::new(RPL_MOTD)
                    .param(&nickname)
                    .trailing(&query.server_config().motd)
                    .build();
                let motd_end = MessageBuilder::new(RPL_ENDOFMOTD)
                    .param(&nickname)
                    .trailing("End of /MOTD command.")
                    .build();
                send(motd_start);
                send(motd);
                send(motd_end);
            }

            Action::Join { channels } => {
                let nickname = query.user().nickname.clone().unwrap();

                for channel_ref in channels {
                    // Create channel if it doesn't exist
                    let channel = query.channel_get_or_create(&channel_ref.name);

                    // Join client into channel
                    channel.join_user(client_id);

                    // Send topic
                    if !channel.topic().is_empty() {
                        let rpl_topic = MessageBuilder::new(RPL_TOPIC)
                            .param(&nickname)
                            .param(&channel_ref.name)
                            .trailing(channel.topic())
                            .build();
                        send(rpl_topic);
                    }

                    // Inform other users of join
                    let channel_users = query
                        .channel_users(&channel_ref.name)
                        .map(|users| users.iter().map(|user| user.client_id).collect_vec());
                    if let Some(users) = channel_users {
                        Action::JoinInform {
                            channel: channel_ref.name.clone(),
                        }
                        .dispatch_multi_by_client_id(query, &users[..]);
                    }
                }
            }

            Action::JoinInform { channel } => {
                let nickname = query.user().nickname.clone().unwrap();
                let join_command = MessageBuilder::new("JOIN")
                    .prefix(&nickname, None, None)
                    .param(channel)
                    .build();
                send(join_command);
            }

            Action::PrivateMessage {
                message,
                users,
                channels,
            } => {
                let nickname = query.user().nickname.clone().unwrap();

                // Collect target user client IDs
                let target_clients = users
                    .iter()
                    .flat_map(|nickname| {
                        if let Some(user) = query.user_find_by_nickname(nickname) {
                            Some(user.client_id)
                        } else {
                            None
                        }
                    })
                    .collect_vec();

                // Dispatch private message to target clients
                for target_client in target_clients {
                    Action::PrivateMessageUser {
                        message: message.clone(),
                        from_nickname: nickname.clone(),
                    }
                    .dispatch_for_client_id(query, target_client);
                }

                // Iterate over target channels
                for channel_name in channels {
                    // Collect clients in channel (except sender client)
                    let clients = query.channel_find(&channel_name).map(|channel| {
                        channel
                            .clients()
                            .iter()
                            .filter(|target_client_id| client_id != **target_client_id)
                            .map(|client_id| *client_id)
                            .collect_vec()
                    });

                    // Dispatch private message to all users of channel
                    if let Some(clients) = clients {
                        Action::PrivateMessageChannel {
                            message: message.clone(),
                            channel: channel_name.clone(),
                            from_nickname: nickname.clone(),
                        }
                        .dispatch_multi_by_client_id(query, &clients[..]);
                    }
                }
            }

            Action::PrivateMessageUser {
                message,
                from_nickname,
            } => {
                let nickname = query.user().nickname.clone().unwrap();
                let privmsg_command = MessageBuilder::new("PRIVMSG")
                    .prefix(&from_nickname, None, None)
                    .param(&nickname)
                    .trailing(&message)
                    .build();
                send(privmsg_command);
            }

            Action::PrivateMessageChannel {
                message,
                channel,
                from_nickname,
            } => {
                let privmsg_command = MessageBuilder::new("PRIVMSG")
                    .prefix(&from_nickname, None, None)
                    .param(channel)
                    .trailing(&message)
                    .build();
                send(privmsg_command);
            }

            Action::Quit { reason } => {
                let user = query.user();
                let nickname = user.nickname.clone().unwrap_or_default();

                // Terminate client
                if user.stream.shutdown(std::net::Shutdown::Both).is_ok() {
                    println!(
                        "[Server] Terminated connection of {nickname}@{host} (QUIT: {reason})",
                        nickname = nickname,
                        host = user_host,
                        reason = reason.as_ref().unwrap_or(&String::new())
                    );
                }

                // Remove client from user list
                let server_mut = query.server_mut();
                let result = server_mut
                    .users
                    .iter()
                    .position(|user| user.client_id == user.client_id)
                    .map(|index| server_mut.users.swap_remove(index));
                if result.is_some() {
                    println!(
                        "[Server] Removed {nickname}@{host} from client list",
                        nickname = nickname,
                        host = user_host
                    )
                }
            }

            Action::Error { code } => {
                let message = MessageBuilder::new(code)
                    .prefix(server_host, None, Some(&user_host))
                    .build();
                send(message);
            }
        }
    }
}
