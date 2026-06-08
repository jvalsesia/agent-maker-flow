//! Clerk session-token verification.
//!
//! Reads the `kid` from the JWT header, resolves the matching decoding key via
//! the JWKS cache, validates the RS256 signature plus the `iss` and `exp`
//! claims, and then checks the `azp` (authorized party) against the configured
//! allow-list. Any failure collapses to `AppError::Unauthorized` so no detail
//! about why a token was rejected leaks to the caller.

use jsonwebtoken::{decode, decode_header, Algorithm, Validation};
use serde::Deserialize;

use crate::auth::AuthState;
use crate::error::AppError;

/// Claims extracted from a verified Clerk session token.
#[derive(Debug, Clone, Deserialize)]
pub struct Claims {
    /// Clerk user id (subject).
    pub sub: String,
    /// Issuer; validated against the configured Clerk issuer.
    #[allow(dead_code)]
    pub iss: String,
    /// Authorized party; validated against the configured allow-list.
    pub azp: Option<String>,
    /// Expiry (seconds since epoch); validated by `jsonwebtoken`.
    pub exp: usize,
    /// Optional email custom claim (best-effort; stored when present).
    #[serde(default)]
    pub email: Option<String>,
}

/// Verify a raw JWT string and return its claims, or `Unauthorized` /
/// `AuthServiceUnavailable` on failure.
pub async fn verify_token(auth: &AuthState, token: &str) -> Result<Claims, AppError> {
    let header = decode_header(token).map_err(|_| AppError::Unauthorized)?;
    let kid = header.kid.ok_or(AppError::Unauthorized)?;

    // May refetch JWKS; a network failure surfaces as 503 (AuthServiceUnavailable).
    let key = auth.jwks.decoding_key(&kid).await?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&[auth.clerk.issuer.as_str()]);
    validation.validate_exp = true;
    // Clerk session tokens do not carry an `aud` claim by default.
    validation.validate_aud = false;

    let data = decode::<Claims>(token, &key, &validation).map_err(|_| AppError::Unauthorized)?;
    let claims = data.claims;

    // Authorized-party check (jsonwebtoken has no built-in `azp` validation).
    if !auth.clerk.authorized_parties.is_empty() {
        let allowed = claims
            .azp
            .as_ref()
            .map(|azp| auth.clerk.authorized_parties.iter().any(|p| p == azp))
            .unwrap_or(false);
        if !allowed {
            return Err(AppError::Unauthorized);
        }
    }

    Ok(claims)
}
