//! Authentication & access control (F02).
//!
//! Clerk JWT verification, an in-memory JWKS cache, the `require_auth`
//! middleware, the `AuthUser` identity extractor + `ensure_owner` guard, and a
//! JIT-provisioned `users` table. `AuthState` bundles the JWKS cache with the
//! Clerk configuration and is carried in `AppState`.

use std::sync::Arc;

use crate::config::ClerkConfig;

pub mod jwks;

use jwks::JwksCache;

/// Shared authentication state: the JWKS cache plus the Clerk config used to
/// validate `iss`/`azp` claims. Cheap to share behind an `Arc`.
pub struct AuthState {
    pub jwks: JwksCache,
    pub clerk: ClerkConfig,
}

impl AuthState {
    /// Build auth state from the Clerk configuration.
    pub fn new(clerk: ClerkConfig) -> Arc<Self> {
        let jwks = JwksCache::new(clerk.jwks_url.clone());
        Arc::new(Self { jwks, clerk })
    }
}
