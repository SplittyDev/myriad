use super::{Server, User};

pub struct ServerQuery<'a> {
    server: &'a Server,
    user: &'a User,
}

impl<'a> ServerQuery<'a> {
    pub fn new(server: &'a Server, user: &'a User) -> Self {
        Self {
            server,
            user,
        }
    }

    pub fn server_host(&self) -> &str {
        &self.server.config.host
    }
}