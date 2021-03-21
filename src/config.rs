use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct ServerConfig {
    pub name: String,
    pub host: String,
    pub port: u16,
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
        }
    }
}