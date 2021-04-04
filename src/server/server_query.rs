use std::borrow::BorrowMut;

use itertools::Itertools;

use crate::{config::ServerConfig, models::Channel};

use super::{Server, User};

pub struct ServerQuery<'a> {
    server: &'a mut Server,
    client_id: u64,
}

impl<'a> ServerQuery<'a> {
    pub fn new(server: &'a mut Server, client_id: u64) -> Self {
        Self {
            server,
            client_id,
        }
    }

    //
    // Server
    //

    pub fn server(&self) -> &Server {
        self.server
    }

    pub fn server_mut(&mut self) -> &mut Server {
        self.server
    }

    pub fn server_host(&self) -> &str {
        &self.server.config.host
    }

    pub fn server_name(&self) -> &str {
        &self.server.config.name
    }

    pub fn server_startup_time(&self) -> String {
        self.server.startup_time.to_string()
    }

    pub fn server_config(&self) -> &ServerConfig {
        &self.server.config
    }

    //
    // User
    //

    pub fn user(&self) -> &User {
        self.server.users
            .iter()
            .find(|user| user.client_id == self.client_id)
            .unwrap()
    }

    pub fn user_find_by_client_id(&self, client_id: u64) -> Option<&User> {
        self.server.users.iter().find(|user| user.client_id == client_id)
    }

    pub fn user_count(&self) -> usize {
        self.server.users.len()
    }

    pub fn user_nick_unsafe(&self) -> String {
        self.user().nickname.clone().unwrap()
    }

    pub fn user_mut(&mut self) -> &mut User {
        let client_id = self.client_id;
        self.server.users
            .iter_mut()
            .find(|user| user.client_id == client_id)
            .unwrap()
    }

    pub fn user_host(&self) -> String {
        self.user().host.to_string()
    }

    //
    // Channel
    //

    pub fn channel_create(&mut self, name: String) {
        let channel = Channel::new(name);
        self.server.channels.push(channel);
    }

    pub fn channel_find(&self, name: &str) -> Option<&Channel> {
        self.server.channels.iter().find(|channel| channel.name() == name)
    }

    pub fn channel_exists(&self, name: &str) -> bool {
        self.server.channels.iter().find(|channel| channel.name() == name).is_some()
    }

    pub fn channel_get_or_create(&mut self, name: &str) -> &mut Channel {
        let server_mut = self.server_mut();
        if server_mut.channels.iter().find(|channel| channel.name() == name).is_none() {
            server_mut.channels.push(Channel::new(name.to_string()));
            return server_mut.channels.last_mut().unwrap();
        } else {
            return server_mut.channels.iter_mut().find(|channel| channel.name() == name).unwrap();
        }
    }

    pub fn channel_users(&self, name: &str) -> Option<Vec<&User>> {
        self.channel_find(name).map(|channel| {
            channel.clients().iter().flat_map(|client_id| {
                self.user_find_by_client_id(*client_id)
            }).collect_vec()
        })
    }
}