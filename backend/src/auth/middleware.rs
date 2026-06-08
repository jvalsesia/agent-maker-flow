//! `require_auth` middleware.
//!
//! Sources the session token from the `Authorization: Bearer` header (REST) or
//! a `?token=` query parameter (SSE / native `EventSource`, which cannot set
//! headers), verifies it, JIT-provisions the user, and injects `AuthUser` into
//! request extensions for downstream handlers.

use axum::extract::{Request, State};
use axum::http::header::AUTHORIZATION;
use axum::middleware::Next;
use axum::response::Response;

use crate::auth::extractor::AuthUser;
use crate::auth::user;
use crate::auth::verify::verify_token;
use crate::error::AppError;
use crate::state::AppState;

/// Validate the request's token and attach the authenticated identity, or
/// reject with the appropriate auth error envelope.
pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = extract_token(&req).ok_or(AppError::Unauthorized)?;
    let claims = verify_token(&state.auth, &token).await?;

    let user = user::upsert_user(&state.db, &claims.sub, claims.email.as_deref()).await?;

    req.extensions_mut().insert(AuthUser {
        id: user.id,
        email: user.email,
    });

    Ok(next.run(req).await)
}

/// Extract the bearer token from the `Authorization` header, falling back to a
/// `token` query parameter for SSE connections.
fn extract_token(req: &Request) -> Option<String> {
    if let Some(value) = req.headers().get(AUTHORIZATION) {
        if let Some(token) = value.to_str().ok().and_then(|s| s.strip_prefix("Bearer ")) {
            let token = token.trim();
            if !token.is_empty() {
                return Some(token.to_string());
            }
        }
    }

    let query = req.uri().query()?;
    for pair in query.split('&') {
        let mut kv = pair.splitn(2, '=');
        if kv.next() == Some("token") {
            if let Some(value) = kv.next() {
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
    }

    None
}
