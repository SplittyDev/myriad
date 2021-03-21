use crate::config::ServerConfig;
use anyhow::{anyhow, Result};
use guard::guard;
use irc_rust::Message;
use std::{
    io::{BufRead, BufReader, BufWriter},
    net::{TcpListener, TcpStream},
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, RwLock,
    },
    thread::{self, JoinHandle},
};

mod action;
mod action_parser;
mod server_query;

use action_parser::ActionParser;

use self::server_query::ServerQuery;
use crate::models::User;

pub struct Server {
    config: ServerConfig,
    users: Vec<User>,
}

#[derive(Debug)]
enum ServerEvent {
    ClientConnected { stream: TcpStream, client_id: u64 },
    ClientDisconnected { client_id: u64 },
    IrcCommand { client_id: u64, message: String },
}

impl Server {
    pub fn new(config: ServerConfig) -> Self {
        Self {
            config,
            users: vec![],
        }
    }

    pub fn listen(&mut self) -> Result<()> {
        let mut threads: Vec<JoinHandle<()>> = Vec::new();

        let (sender, recv) = channel::<ServerEvent>();
        let listener = TcpListener::bind(self.config.addr())?;

        thread::spawn(move || {
            let client_count = Arc::new(RwLock::new(0u64));
            let assign_client_id = || -> Result<u64> {
                let client_count = client_count.clone();
                let mut writer = client_count
                    .write()
                    .map_err(|_| anyhow!("Unable to acquire mutable client count."))?;
                *writer += 1;
                Ok(*writer)
            };

            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        let sender = sender.clone();
                        let client_id = match assign_client_id() {
                            Ok(client_id) => client_id,
                            _ => return,
                        };
                        let handle = thread::spawn(move || {
                            if let Err(err) = Self::handle_stream(stream, client_id, sender) {
                                println!("[Client(id: {})] Error: {}", client_id, err);
                            }
                        });
                        threads.push(handle);
                    }
                    Err(err) => {
                        println!("Unable to accept client stream: {}", err)
                    }
                }
            }
        });

        self.handle_commands(recv);

        Ok(())
    }

    fn handle_stream(stream: TcpStream, client_id: u64, sender: Sender<ServerEvent>) -> Result<()> {
        // Register client with server
        sender.send(ServerEvent::ClientConnected {
            stream: stream.try_clone()?,
            client_id,
        })?;

        // Get a buffered reader for the incoming data
        let _addr = stream.peer_addr()?;
        let mut reader = BufReader::new(stream.try_clone()?);

        loop {
            // Read the next line
            let mut line = String::new();
            reader.read_line(&mut line)?;

            // Test for disconnect
            if line.is_empty() {
                sender.send(ServerEvent::ClientDisconnected { client_id })?;
                stream.shutdown(std::net::Shutdown::Both)?;
                break;
            }

            // Send message to server
            sender.send(ServerEvent::IrcCommand {
                client_id,
                message: line,
            })?;
        }

        Ok(())
    }

    fn handle_commands(&mut self, receiver: Receiver<ServerEvent>) {
        for command in receiver {
            self.handle_command(command)
        }
    }

    fn handle_command(&mut self, command: ServerEvent) {
        // Helpers
        #[allow(unused_variables)]
        let find_user_index_by_client_id = |client_id| {
            self.users
                .iter()
                .position(|user| user.client_id == client_id)
        };

        match command {
            ServerEvent::ClientConnected { stream, client_id } => {
                guard!(let Ok(peer_addr) = stream.peer_addr() else { return });
                let user = User::new(stream, client_id, peer_addr.to_string());
                self.users.push(user);
                dbg!(&self.users);
            }

            ServerEvent::ClientDisconnected { client_id } => {
                if let Some(index) = find_user_index_by_client_id(client_id) {
                    self.users.swap_remove(index);
                }
                dbg!(&self.users);
            }

            ServerEvent::IrcCommand { client_id, message } => {
                println!("[{} ->] {}", client_id, message);

                // Get a mutable writer for the user's stream
                let mut writer = {
                    guard!(let Some(user) = self.users.iter().find(|user| user.client_id == client_id) else { return });
                    let writer = user.stream
                        .try_clone()
                        .ok()
                        .map(|stream| BufWriter::new(stream));
                    guard!(let Some(mut writer) = writer else { return });
                    writer
                };
                
                // Initialize server query with mutable self and client id
                let mut query = ServerQuery::new(self, client_id);

                // Parse IRC message
                let message = Message::from(message.trim_end());
                guard!(let Some(action) = ActionParser::parse(message, &mut query) else { return });

                // Dispatch the action
                action.dispatch(&mut query, &mut writer);
            }
        }
    }
}
