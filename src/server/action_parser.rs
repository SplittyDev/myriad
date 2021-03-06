use crate::models::ChannelRef;
use crate::numerics::*;
use guard::guard;
use irc_rust::Message;
use itertools::Itertools;

use super::{action::Action, server_query::ServerQuery};

pub struct ActionParser;

impl ActionParser {
    pub fn parse(message: Message, query: &mut ServerQuery) -> Option<Action> {
        match message.command() {
            "PING" => {
                // Validate params
                if let Some(params) = message.params() {
                    if let Some(challenge) = params.iter().nth(0) {
                        return Some(Action::Pong {
                            challenge: Some(challenge.to_string()),
                        });
                    }
                }

                return Some(Action::Pong { challenge: None });
            }

            // NICK <nickname>
            "NICK" => {
                // Validate params
                guard!(let Some(params) = message.params() else {
                    return Some(Action::Error { code: ERR_NONICKNAMEGIVEN })
                });
                guard!(let Some(nickname) = params.iter().nth(0) else {
                    return Some(Action::Error { code: ERR_NONICKNAMEGIVEN })
                });

                // Check if user already has a nickname
                if let Some(old_nickname) = &query.user().nickname {
                    // Check if nickname collides with current one
                    if old_nickname == nickname {
                        return Some(Action::Error {
                            code: ERR_NICKNAMEINUSE,
                        });
                    }

                    // Dispatch nick change
                    Some(Action::ChangeNick {
                        prev_nickname: old_nickname.clone(),
                        nickname: nickname.to_string(),
                    })
                } else {
                    // Dispatch initial nick change
                    Some(Action::SetNick {
                        nickname: nickname.to_string(),
                    })
                }
            }

            // USER <username> 0 * <realname>
            // USER <username> 0 * :<realname>
            "USER" => {
                // Validate params
                guard!(let Some(params) = message.params() else {
                    return Some(Action::Error { code: ERR_NEEDMOREPARAMS })
                });
                let mut params_iter = params.iter();
                guard!(let Some(username) = params_iter.next() else {
                    return Some(Action::Error { code: ERR_NEEDMOREPARAMS })
                });
                if let Some(param) = params_iter.next() {
                    if param != "0" {
                        println!("USER: Nonstandard param. Should be '0', was {}", param);
                    }
                } else {
                    return Some(Action::Error {
                        code: ERR_NEEDMOREPARAMS,
                    });
                }
                if let Some(param) = params_iter.next() {
                    if param != "*" {
                        println!("USER: Nonstandard param. Should be '*', was {}", param);
                    }
                } else {
                    return Some(Action::Error {
                        code: ERR_NEEDMOREPARAMS,
                    });
                }
                let realname = {
                    if let Some(realname) = params_iter.next() {
                        realname
                    } else if let Some(realname) = params.trailing() {
                        realname
                    } else {
                        return Some(Action::Error {
                            code: ERR_NEEDMOREPARAMS,
                        });
                    }
                };

                // Check if user is already registered
                if query.user().username.is_some() {
                    return Some(Action::Error {
                        code: ERR_ALREADYREGISTRED,
                    });
                }

                // Dispatch registration
                Some(Action::SetUserAndRealName {
                    username: format!("~{}", username),
                    realname: realname.to_string(),
                })
            }

            "MOTD" => {
                // Dispatch MOTD reply
                Some(Action::Motd)
            }

            "JOIN" => {
                guard!(let Some(params) = message.params() else {
                    return Some(Action::Error { code: ERR_NEEDMOREPARAMS })
                });

                let mut params_iter = params.iter();
                let channels = params_iter.next().unwrap().split(",");
                let channel_keys = params_iter.next().map(|keys| keys.split(",").collect_vec());

                let channel_refs = channels
                    .zip_longest(channel_keys.unwrap_or_default())
                    .map(ChannelRef::from)
                    .collect_vec();

                Some(Action::Join {
                    channels: channel_refs,
                })
            }

            "PRIVMSG" => {
                guard!(let Some(params) = message.params() else {
                    return Some(Action::Error { code: ERR_NEEDMOREPARAMS });
                });
                let mut params_iter = params.iter();
                guard!(let Some(targets) = params_iter.next().map(|s| s.split(",").collect_vec()) else {
                    return Some(Action::Error { code: ERR_NEEDMOREPARAMS });
                });
                let message = {
                    if let Some(message) = params_iter.next() {
                        Some(message)
                    } else if let Some(message) = params.trailing() {
                        Some(message)
                    } else {
                        None
                    }
                };
                guard!(let Some(message) = message else {
                    return Some(Action::Error { code: ERR_NEEDMOREPARAMS });
                });

                let channel_targets = targets
                    .iter()
                    .filter(|target| {
                        !target.is_empty()
                            && vec!['#', '&']
                                .iter()
                                .any(|&c| c == target.chars().next().unwrap())
                    })
                    .map(|s| s.to_string())
                    .collect_vec();

                let user_targets = targets
                    .iter()
                    .filter(|target| !channel_targets.contains(&target.to_string()))
                    .map(|s| s.to_string())
                    .collect_vec();

                Some(Action::PrivateMessage {
                    message: message.to_string(),
                    channels: channel_targets,
                    users: user_targets,
                })
            }

            "QUIT" => {
                let reason = {
                    if let Some(params) = message.params() {
                        params
                            .iter()
                            .next()
                            .or_else(|| params.trailing())
                            .map(ToString::to_string)
                    } else {
                        None
                    }
                };
                Some(Action::Quit { reason })
            }

            command => {
                println!("Unimplemented: {}", command);
                None
            }
        }
    }
}
