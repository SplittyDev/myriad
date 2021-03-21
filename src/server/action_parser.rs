use irc_rust::Message;
use guard::guard;
use crate::numerics::*;

use super::{action::Action, server_query::ServerQuery};

pub struct ActionParser;

impl ActionParser {
    pub fn parse(message: Message, query: &mut ServerQuery) -> Option<Action> {
        match message.command() {
            "PING" => Some(Action::Pong),
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
            command => {
                println!("Unimplemented: {}", command);
                None
            }
        }
    }
}