use std::net::TcpStream;

#[derive(Debug)]
pub struct User {
    pub stream: TcpStream,
    pub client_id: u64,
    pub host: String,
    pub nickname: Option<String>,
}

impl User {
    pub fn new(stream: TcpStream, client_id: u64, host: String) -> Self {
        Self {
            stream,
            client_id,
            host,
            nickname: None,
        }
    }
}