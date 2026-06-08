//! `GET /api/v1/me` — return the authenticated user's identity.

use axum::Json;
use serde_json::{json, Value};

use crate::auth::AuthUser;

/// Returns the caller's identity from the `AuthUser` injected by `require_auth`.
/// Reaching this handler implies a valid token (the middleware rejects
/// otherwise), so it always renders the success envelope.
pub async fn me(user: AuthUser) -> Json<Value> {
    Json(json!({
        "status": "success",
        "data": {
            "user_id": user.id,
            "email": user.email,
        }
    }))
}
