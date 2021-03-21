use irc_rust::Message;

use super::action::Action;

pub struct ActionParser;

impl ActionParser {
    pub fn parse(message: Message) -> Option<Action> {
        match message.command() {
            "PING" => Some(Action::Pong),
            command => {
                println!("Unimplemented: {}", command);
                None
            }
        }
    }
}