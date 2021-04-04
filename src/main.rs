use anyhow::{anyhow, Result};
use std::fs::File;
use std::io::{Read, Write};

mod config;
mod models;
mod numerics;
mod server;

use config::ServerConfig;
use server::Server;

fn main() -> Result<()> {
    let config = read_or_create_config()?;
    Server::new(config).listen()
}

fn read_or_create_config() -> Result<ServerConfig> {
    if std::path::Path::new("config.toml").exists() {
        read_config()
    } else {
        create_config()
    }
}

fn create_config() -> Result<ServerConfig> {
    // Create default configuration
    let config = ServerConfig::default();

    // Serialize config to string
    match toml::to_string(&config) {
        Ok(config_text) => {
            // Write to file
            let mut file = File::create("config.toml")?;
            file.write_all(config_text.as_ref())?;

            // Return the configuration
            Ok(config)
        }
        Err(_) => Err(anyhow!("Unable to serialize default config.")),
    }
}

fn read_config() -> Result<ServerConfig> {
    // Open config file
    let mut config_file = File::open("config.toml")?;

    // Read file into buffer
    let mut config_text = String::new();
    config_file.read_to_string(&mut config_text)?;

    // Deserialize configuration
    match toml::from_str(&config_text) {
        Ok(config) => Ok(config),
        Err(_) => Err(anyhow!("Unable to deserialize config.")),
    }
}
