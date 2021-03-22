use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct ServerConfig {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub motd: String,
}

impl ServerConfig {
    
    /// Get the (host, port) pair.
    pub fn addr(&self) -> (&str, u16) {
        (&self.host, self.port)
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            name: String::from("Myriad Devnet"),
            host: String::from("127.0.0.1"),
            port: 6667,
            motd: String::from("Don't worry, it only seems kinky the first time."),
        }
    }
}