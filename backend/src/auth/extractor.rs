//! Authenticated identity extractor and ownership guard.
//!
//! `AuthUser` is inserted into request extensions by `require_auth` and pulled
//! back out by handlers via this `FromRequestParts` impl. `ensure_owner` is the
//! shared primitive later features use to reject cross-user access with a 404
//! that never reveals the existence of another user's record.

use axum::extract::FromRequestParts;
use axum::http::request::Parts;

use crate::error::AppError;

/// The authenticated caller, available to any handler behind `require_auth`.
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: String,
    pub email: Option<String>,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthUser>()
            .cloned()
            .ok_or(AppError::Unauthorized)
    }
}

/// Allow the request only when the caller owns the record; otherwise 404.
/// Never distinguishes "not found" from "owned by someone else".
pub fn ensure_owner(owner_id: &str, caller: &AuthUser) -> Result<(), AppError> {
    if owner_id == caller.id {
        Ok(())
    } else {
        Err(AppError::NotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    fn caller() -> AuthUser {
        AuthUser {
            id: "user_1".to_string(),
            email: None,
        }
    }

    #[test]
    fn ensure_owner_allows_owner() {
        assert!(ensure_owner("user_1", &caller()).is_ok());
    }

    #[test]
    fn ensure_owner_rejects_other_user() {
        let err = ensure_owner("user_2", &caller()).unwrap_err();
        assert_eq!(err.code(), "NOT_FOUND");
        assert_eq!(err.status(), StatusCode::NOT_FOUND);
    }
}
