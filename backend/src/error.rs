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

    /// Missing, malformed, or expired/invalid authentication token (F02).
    #[error("Session expired or invalid. Please sign in again.")]
    Unauthorized,

    /// The authentication service (Clerk JWKS) is unreachable (F02).
    #[error("Authentication service unavailable. Please try again shortly.")]
    AuthServiceUnavailable,

    /// A record does not exist, or belongs to another user (F02 ownership
    /// guard never reveals the existence of other users' data).
    #[error("Not found")]
    NotFound,

    /// The LiteLLM gateway/proxy is unreachable (F03).
    #[error("Model gateway unavailable. Check the LiteLLM proxy and try again.")]
    GatewayUnavailable,

    /// A model is not available for the requested provider (F03).
    #[error("Selected model is not available for this provider.")]
    InvalidModelForProvider,

    /// A provider returned an error (rate limit, quota, exhausted fallback);
    /// the message carries the provider name and reason (F03).
    #[error("{0}")]
    ProviderError(String),

    /// A request field failed validation; `message` carries the field-specific
    /// text and `code` the stable machine code (e.g. `AGENT_VALIDATION`) (F04).
    #[error("{message}")]
    Validation {
        code: &'static str,
        message: String,
    },

    /// A uniqueness/conflict constraint was violated; `message` is the
    /// user-facing text, `code` the stable machine code (e.g.
    /// `AGENT_NAME_TAKEN`) (F04).
    #[error("{message}")]
    Conflict {
        code: &'static str,
        message: String,
    },

    /// A memory record exceeds the maximum length (F05).
    #[error("Memory record must be 8000 characters or fewer.")]
    MemoryRecordTooLarge,

    /// No global embedding model is configured; one must be set before memory
    /// records can be embedded and stored (F05).
    #[error("No embedding model is configured. Set a global embedding model first.")]
    NoEmbeddingModel,

    /// Any other internal failure.
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl AppError {
    /// Stable machine-readable error code.
    pub fn code(&self) -> &'static str {
        match self {
            AppError::DependencyUnavailable(_) => "HEALTH001",
            AppError::Unauthorized => "AUTH001",
            AppError::AuthServiceUnavailable => "AUTH002",
            AppError::NotFound => "NOT_FOUND",
            AppError::GatewayUnavailable => "GW001",
            AppError::InvalidModelForProvider => "GW002",
            AppError::ProviderError(_) => "GW003",
            AppError::Validation { code, .. } => code,
            AppError::Conflict { code, .. } => code,
            AppError::MemoryRecordTooLarge => "MEM001",
            AppError::NoEmbeddingModel => "MEM002",
            AppError::Internal(_) => "INTERNAL001",
        }
    }

    /// HTTP status mapped from the variant.
    pub fn status(&self) -> StatusCode {
        match self {
            AppError::DependencyUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::AuthServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::GatewayUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            AppError::InvalidModelForProvider => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::ProviderError(_) => StatusCode::BAD_GATEWAY,
            AppError::Validation { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::Conflict { .. } => StatusCode::CONFLICT,
            AppError::MemoryRecordTooLarge => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::NoEmbeddingModel => StatusCode::CONFLICT,
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
