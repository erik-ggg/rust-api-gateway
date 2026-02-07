use config::{Config, File};
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub proxy: ProxyConfig,
    pub security: SecurityConfig,
    pub reliability: ReliabilityConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub log_level: String,
    pub log_format: String, // "text" or "json"
}

#[derive(Debug, Deserialize, Clone)]
pub struct ProxyConfig {
    pub target_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SecurityConfig {
    pub https: bool,
    pub secure_cookies: bool,
    pub same_site: String, // "Lax", "Strict", "None"
    pub enable_hsts: bool,
    pub enable_csp: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ReliabilityConfig {
    pub rate_limit_per_sec: u64,
    pub timeout_ms: u64,
}

impl AppConfig {
    pub fn new() -> Result<Self, config::ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        // Start with default config
        let builder = Config::builder().add_source(File::with_name("config/default"));

        // Add environment specific config if it exists
        let builder =
            builder.add_source(File::with_name(&format!("config/{}", run_mode)).required(false));

        // Add env overrides (e.g. APP__SERVER__PORT=8080)
        let builder = builder.add_source(config::Environment::with_prefix("APP").separator("__"));

        builder.build()?.try_deserialize()
    }
}
