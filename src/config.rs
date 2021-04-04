use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum CaseMap {
    Ascii,
    Rfc1459,
    Rfc1459Strict,
    Rfc7613,
}

impl ToString for CaseMap {
    fn to_string(&self) -> String {
        match self {
            Self::Ascii => "ascii",
            Self::Rfc1459 => "rfc1459",
            Self::Rfc1459Strict => "rfc1459-strict",
            Self::Rfc7613 => "rfc7613",
        }
        .to_string()
    }
}

#[derive(Serialize, Deserialize)]
pub struct ServerConfig {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub motd: String,
    #[serde(default = "ServerConfig::default_feat_awaylen")]
    pub feat_awaylen: u32,
    #[serde(default = "ServerConfig::default_feat_casemap")]
    pub feat_casemap: CaseMap,
}

// Default values for deserialization
impl ServerConfig {
    fn default_feat_awaylen() -> u32 {
        255
    }
    fn default_feat_casemap() -> CaseMap {
        CaseMap::Ascii
    }
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
            feat_awaylen: ServerConfig::default_feat_awaylen(),
            feat_casemap: ServerConfig::default_feat_casemap(),
        }
    }
}
