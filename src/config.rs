use std::sync::OnceLock;

use config::{Config as ConfigLoader, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
  pub server: ServerConfig,
  pub database: DatabaseConfig,
  pub jwt: JwtConfig,
  pub logger: LoggerConfig,
  pub cors: CorsConfig,
  pub app: AppConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
  pub host: String,
  pub port: u16,
  #[serde(default = "default_binding")]
  pub binding: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
  pub url: String,
  #[serde(default = "default_db_logging")]
  pub enable_logging: bool,
}

fn default_db_logging() -> bool {
  false
}

#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
  pub secret: String,
  #[serde(default = "default_jwt_expiration")]
  pub expiration: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggerConfig {
  #[serde(default = "default_log_level")]
  pub level: String,
  #[serde(default = "default_log_format")]
  pub format: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CorsConfig {
  pub allow_origins: Vec<String>,
  #[serde(default = "default_cors_allow_headers")]
  pub allow_headers: Vec<String>,
  #[serde(default = "default_cors_allow_methods")]
  pub allow_methods: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
  pub base_url: String,
}

// Default value functions
fn default_binding() -> String {
  "localhost".to_string()
}

fn default_jwt_expiration() -> u64 {
  604800 // 7 days
}

fn default_log_level() -> String {
  "info".to_string()
}

fn default_log_format() -> String {
  "compact".to_string()
}

fn default_cors_allow_headers() -> Vec<String> {
  vec![
    "Authorization".to_string(),
    "Content-Type".to_string(),
    "Accept".to_string(),
  ]
}

fn default_cors_allow_methods() -> Vec<String> {
  vec![
    "GET".to_string(),
    "POST".to_string(),
    "PUT".to_string(),
    "DELETE".to_string(),
    "OPTIONS".to_string(),
  ]
}

pub static LOCK: OnceLock<Config> = OnceLock::new();

impl Config {
  pub fn get() -> &'static Self {
    LOCK.get().expect("Config not initialized")
  }
  /// Load configuration from a YAML file based on the environment
  /// Environment can be: "development", "production", or "test"
  pub fn load(environment: &str) -> Result<Self, ConfigError> {
    let mut builder = ConfigLoader::builder()
      // Load the environment-specific config file
      .add_source(File::with_name(&format!("config/{}", environment)).required(true))
      // Allow environment variables with APP__ prefix to override config
      // Example: APP__DATABASE__URL=postgres://... will override database.url
      .add_source(Environment::with_prefix("APP").separator("__"));

    // Handle PORT environment variable (standard for cloud platforms like Google Cloud Run)
    // Falls back to the port defined in the YAML file (default: 5150)
    if let Ok(port) = std::env::var("PORT") {
      builder = builder.set_override("server.port", port)?;
    }

    builder.build()?.try_deserialize()
  }
}
