//! Typed application configuration loaded from the environment.

use anyhow::Context;

/// Runtime configuration for the backend service.
///
/// Required variables fail fast at startup with a clear message; optional
/// variables fall back to development-friendly defaults.
#[derive(Clone, Debug)]
pub struct AppConfig {
    pub database_url: String,
    pub redis_url: String,
    pub bind_addr: String,
    pub frontend_origin: String,
}

impl AppConfig {
    /// Build configuration from environment variables.
    pub fn from_env() -> anyhow::Result<Self> {
        fn required(key: &str) -> anyhow::Result<String> {
            std::env::var(key).with_context(|| format!("missing required env var {key}"))
        }

        Ok(Self {
            database_url: required("DATABASE_URL")?,
            redis_url: required("REDIS_URL")?,
            bind_addr: std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            frontend_origin: std::env::var("FRONTEND_ORIGIN")
                .unwrap_or_else(|_| "http://localhost:5173".to_string()),
        })
    }
}
