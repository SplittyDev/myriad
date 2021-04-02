use irc_rust::Message;
use guard::guard;
use crate::numerics::*;

use super::{action::Action, server_query::ServerQuery};

pub struct ActionParser;

impl ActionParser {
    pub fn parse(message: Message, query: &mut ServerQuery) -> Option<Action> {
        match message.command() {
            "PING" => {

                // Validate params
                if let Some(params) = message.params() {
                    if let Some(challenge) = params.iter().nth(0) {
                        return Some(Action::Pong { challenge: Some(challenge.to_string()) })
                    }
                }

                return Some(Action::Pong { challenge: None })
            },

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
                        return Some(Action::Error { code: ERR_NICKNAMEINUSE })
                    }

                    // Dispatch nick change
                    Some(Action::ChangeNick {
                        prev_nickname: old_nickname.clone(),
                        nickname: nickname.to_string()
                    })
                } else {

                    // Dispatch initial nick change
                    Some(Action::SetNick { nickname: nickname.to_string() })
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
                } else { return Some(Action::Error { code: ERR_NEEDMOREPARAMS }) }
                if let Some(param) = params_iter.next() {
                    if param != "*" {
                        println!("USER: Nonstandard param. Should be '*', was {}", param);
                    }
                } else { return Some(Action::Error { code: ERR_NEEDMOREPARAMS }) }
                let realname = {
                    if let Some(realname) = params_iter.next() {
                        realname
                    } else if let Some(realname) = params.trailing() {
                        realname
                    } else {
                        return Some(Action::Error { code: ERR_NEEDMOREPARAMS })
                    }
                };

                // Check if user is already registered
                if query.user().username.is_some() {
                    return Some(Action::Error { code: ERR_ALREADYREGISTRED })
                }

                // Dispatch registration
                Some(Action::SetUserAndRealName {
                    username: format!("~{}", username),
                    realname: realname.to_string()
                })
            }
            command => {
                println!("Unimplemented: {}", command);
                None
            }
        }
    }
}