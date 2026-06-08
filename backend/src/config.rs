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
    pub clerk: ClerkConfig,
}

/// Clerk authentication settings (F02). `CLERK_ISSUER` and
/// `CLERK_AUTHORIZED_PARTIES` are required so token validation can pin the
/// issuer and authorized parties; `CLERK_JWKS_URL` defaults to the issuer's
/// well-known JWKS endpoint when unset.
#[derive(Clone, Debug)]
pub struct ClerkConfig {
    pub issuer: String,
    pub jwks_url: String,
    pub authorized_parties: Vec<String>,
}

impl AppConfig {
    /// Build configuration from environment variables.
    pub fn from_env() -> anyhow::Result<Self> {
        fn required(key: &str) -> anyhow::Result<String> {
            std::env::var(key).with_context(|| format!("missing required env var {key}"))
        }

        let issuer = required("CLERK_ISSUER")?;
        let jwks_url = std::env::var("CLERK_JWKS_URL")
            .unwrap_or_else(|_| format!("{}/.well-known/jwks.json", issuer.trim_end_matches('/')));
        let authorized_parties = required("CLERK_AUTHORIZED_PARTIES")?
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(Self {
            database_url: required("DATABASE_URL")?,
            redis_url: required("REDIS_URL")?,
            bind_addr: std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            frontend_origin: std::env::var("FRONTEND_ORIGIN")
                .unwrap_or_else(|_| "http://localhost:5173".to_string()),
            clerk: ClerkConfig {
                issuer,
                jwks_url,
                authorized_parties,
            },
        })
    }
}
