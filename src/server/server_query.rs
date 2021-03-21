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

    pub fn server_host(&self) -> &str {
        &self.server.config.host
    }

    pub fn user(&self) -> &User {
        self.server.users
            .iter()
            .find(|user| user.client_id == self.client_id)
            .unwrap()
    }

    pub fn user_mut(&mut self) -> &mut User {
        let client_id = self.client_id;
        self.server.users
            .iter_mut()
            .find(|user| user.client_id == client_id)
            .unwrap()
    }

    pub fn user_host(&self) -> &str {
        &self.user().host
    }
}