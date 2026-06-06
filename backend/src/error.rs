//! Application error type and JSON error envelope.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

/// Standard application error. Renders the platform-wide error envelope:
/// `{ "status": "error", "error": { "code", "message" } }`.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// A required dependency (database/cache) is unreachable at request time.
    #[error("{0}")]
    DependencyUnavailable(String),

    /// Any other internal failure.
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl AppError {
    /// Stable machine-readable error code.
    pub fn code(&self) -> &'static str {
        match self {
            AppError::DependencyUnavailable(_) => "HEALTH001",
            AppError::Internal(_) => "INTERNAL001",
        }
    }

    /// HTTP status mapped from the variant.
    pub fn status(&self) -> StatusCode {
        match self {
            AppError::DependencyUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = Json(json!({
            "status": "error",
            "error": {
                "code": self.code(),
                "message": self.to_string(),
            }
        }));
        (self.status(), body).into_response()
    }
}
