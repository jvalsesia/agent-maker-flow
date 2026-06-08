//! Users repository: JIT provisioning of the local user identity.
//!
//! On the first authenticated request a user row is upserted, keyed by the
//! Clerk user id (`sub`). The row is the FK target later features use to scope
//! their records to an owner.

use serde::Serialize;
use sqlx::PgPool;

use crate::error::AppError;

/// A provisioned local user.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub email: Option<String>,
}

/// Insert-or-update the user keyed by the Clerk `sub`, refreshing `updated_at`
/// and filling in the email when newly present. Returns the stored row.
pub async fn upsert_user(pool: &PgPool, sub: &str, email: Option<&str>) -> Result<User, AppError> {
    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (id, email)
         VALUES ($1, $2)
         ON CONFLICT (id) DO UPDATE
         SET email = COALESCE(EXCLUDED.email, users.email),
             updated_at = now()
         RETURNING id, email",
    )
    .bind(sub)
    .bind(email)
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::Internal(anyhow::anyhow!(e).context("failed to upsert user")))?;

    Ok(user)
}

/// Fetch a user by id, if present.
#[allow(dead_code)]
pub async fn get_user(pool: &PgPool, id: &str) -> Result<Option<User>, AppError> {
    let user = sqlx::query_as::<_, User>("SELECT id, email FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e).context("failed to fetch user")))?;

    Ok(user)
}
