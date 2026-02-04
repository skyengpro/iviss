use anyhow::{Context, Result};
use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub external_database_url: String, // Added for "Two pools" req
    pub jwt_secret: String,
    pub server_host: String,
    pub server_port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .context("DATABASE_URL must be set")?;
            
        let external_database_url = env::var("EXTERNAL_DATABASE_URL")
            .unwrap_or_else(|_| database_url.clone()); // Fallback for dev

        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "supersecr3t".to_string());

        let server_host = env::var("SERVER_HOST")
            .unwrap_or_else(|_| "0.0.0.0".to_string());

        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .context("SERVER_PORT must be a number")?;

        Ok(Self {
            database_url,
            external_database_url,
            jwt_secret,
            server_host,
            server_port,
        })
    }
}
