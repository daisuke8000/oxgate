use secrecy::SecretBox;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub database_url: SecretBox<String>,
    pub hydra_admin_url: String,
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

const DEFAULT_HOST: &str = "0.0.0.0";
const DEFAULT_PORT: u16 = 3000;

fn default_host() -> String {
    DEFAULT_HOST.to_string()
}

fn default_port() -> u16 {
    DEFAULT_PORT
}

impl Config {
    pub fn load() -> Result<Self, envy::Error> {
        envy::from_env()
    }
}
