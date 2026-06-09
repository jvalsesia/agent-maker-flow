//! Agent business logic.
//!
//! The source of truth for the PRD field rules: every create/update validates
//! each field, enforces per-user name uniqueness with a friendly message,
//! validates the `(provider, model)` pair against the live F03 gateway catalog,
//! and applies the owner scope before returning or mutating a record. The
//! client mirrors these checks for inline UX, but the server is authoritative.

use sqlx::PgPool;
use uuid::Uuid;

use super::model::{
    Agent, AgentInput, NAME_MAX, PREAMBLE_MAX, RECENT_N_MAX, RECENT_N_MIN, SYSTEM_PROMPT_MAX,
    TOP_K_MAX, TOP_K_MIN,
};
use super::repo;
use crate::error::AppError;
use crate::gateway::GatewayClient;

/// Machine code for a field-validation failure (422).
const VALIDATION_CODE: &str = "AGENT_VALIDATION";
/// Machine code for a per-user name collision (409).
const NAME_TAKEN_CODE: &str = "AGENT_NAME_TAKEN";

fn validation(message: impl Into<String>) -> AppError {
    AppError::Validation {
        code: VALIDATION_CODE,
        message: message.into(),
    }
}

fn name_taken(name: &str) -> AppError {
    AppError::Conflict {
        code: NAME_TAKEN_CODE,
        message: format!("An agent named '{name}' already exists."),
    }
}

/// Validate every field per the PRD rules. Lengths are measured in characters
/// to match the `varchar(N)` column semantics.
fn validate(input: &AgentInput) -> Result<(), AppError> {
    let name = input.name.trim();
    if name.is_empty() {
        return Err(validation("Name is required."));
    }
    if name.chars().count() > NAME_MAX {
        return Err(validation(format!(
            "Name must be {NAME_MAX} characters or fewer."
        )));
    }
    if let Some(preamble) = &input.preamble {
        if preamble.chars().count() > PREAMBLE_MAX {
            return Err(validation(format!(
                "Preamble must be {PREAMBLE_MAX} characters or fewer."
            )));
        }
    }
    if input.system_prompt.trim().is_empty() {
        return Err(validation("System prompt is required."));
    }
    if input.system_prompt.chars().count() > SYSTEM_PROMPT_MAX {
        return Err(validation(format!(
            "System prompt must be {SYSTEM_PROMPT_MAX} characters or fewer."
        )));
    }
    if !(RECENT_N_MIN..=RECENT_N_MAX).contains(&input.recent_n) {
        return Err(validation(format!(
            "Value must be between {RECENT_N_MIN} and {RECENT_N_MAX}."
        )));
    }
    if !(TOP_K_MIN..=TOP_K_MAX).contains(&input.top_k) {
        return Err(validation(format!(
            "Value must be between {TOP_K_MIN} and {TOP_K_MAX}."
        )));
    }
    Ok(())
}

/// Reject a `(provider, model)` pair that the F03 catalog does not offer.
/// Propagates `GW001` when the gateway is unreachable; an unknown provider or
/// an absent model both yield `GW002`.
async fn validate_provider_model(
    gateway: &GatewayClient,
    provider: &str,
    model: &str,
) -> Result<(), AppError> {
    match gateway.list_models(provider).await? {
        Some(models) if models.iter().any(|m| m.id == model) => Ok(()),
        _ => Err(AppError::InvalidModelForProvider),
    }
}

/// Create an agent for `owner_id` after full validation.
pub async fn create(
    db: &PgPool,
    gateway: &GatewayClient,
    owner_id: &str,
    mut input: AgentInput,
) -> Result<Agent, AppError> {
    validate(&input)?;
    input.name = input.name.trim().to_string();

    if repo::name_exists(db, owner_id, &input.name, None).await? {
        return Err(name_taken(&input.name));
    }
    validate_provider_model(gateway, &input.provider, &input.model).await?;

    repo::insert(db, owner_id, &input).await
}

/// List the caller's agents, ordered by name.
pub async fn list(db: &PgPool, owner_id: &str) -> Result<Vec<Agent>, AppError> {
    repo::list_by_owner(db, owner_id).await
}

/// Fetch one of the caller's agents; `NotFound` if absent or owned by another.
pub async fn get(db: &PgPool, owner_id: &str, id: Uuid) -> Result<Agent, AppError> {
    repo::get(db, owner_id, id).await?.ok_or(AppError::NotFound)
}

/// Full-replacement update of the caller's agent after full validation. The
/// uniqueness check excludes the agent's own row so an unchanged name is fine.
pub async fn update(
    db: &PgPool,
    gateway: &GatewayClient,
    owner_id: &str,
    id: Uuid,
    mut input: AgentInput,
) -> Result<Agent, AppError> {
    validate(&input)?;
    input.name = input.name.trim().to_string();

    if repo::name_exists(db, owner_id, &input.name, Some(id)).await? {
        return Err(name_taken(&input.name));
    }
    validate_provider_model(gateway, &input.provider, &input.model).await?;

    repo::update(db, owner_id, id, &input)
        .await?
        .ok_or(AppError::NotFound)
}

/// Delete one of the caller's agents; `NotFound` if absent or owned by another.
pub async fn delete(db: &PgPool, owner_id: &str, id: Uuid) -> Result<(), AppError> {
    if repo::delete(db, owner_id, id).await? {
        Ok(())
    } else {
        Err(AppError::NotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_input() -> AgentInput {
        AgentInput {
            name: "Summarizer".to_string(),
            preamble: Some("You are concise.".to_string()),
            system_prompt: "Summarize the input.".to_string(),
            provider: "openai".to_string(),
            model: "gpt-4o".to_string(),
            recent_n: 10,
            top_k: 5,
        }
    }

    #[test]
    fn accepts_a_valid_input() {
        assert!(validate(&valid_input()).is_ok());
    }

    #[test]
    fn rejects_empty_and_whitespace_name() {
        for blank in ["", "   "] {
            let mut input = valid_input();
            input.name = blank.to_string();
            let err = validate(&input).unwrap_err();
            assert_eq!(err.code(), VALIDATION_CODE);
            assert!(err.to_string().contains("Name"));
        }
    }

    #[test]
    fn rejects_name_over_limit() {
        let mut input = valid_input();
        input.name = "a".repeat(NAME_MAX + 1);
        assert_eq!(validate(&input).unwrap_err().code(), VALIDATION_CODE);
    }

    #[test]
    fn accepts_name_at_limit() {
        let mut input = valid_input();
        input.name = "a".repeat(NAME_MAX);
        assert!(validate(&input).is_ok());
    }

    #[test]
    fn rejects_preamble_over_limit() {
        let mut input = valid_input();
        input.preamble = Some("p".repeat(PREAMBLE_MAX + 1));
        assert_eq!(validate(&input).unwrap_err().code(), VALIDATION_CODE);
    }

    #[test]
    fn rejects_empty_system_prompt() {
        let mut input = valid_input();
        input.system_prompt = "   ".to_string();
        assert_eq!(validate(&input).unwrap_err().code(), VALIDATION_CODE);
    }

    #[test]
    fn rejects_out_of_range_recent_n() {
        for bad in [-1, RECENT_N_MAX + 1] {
            let mut input = valid_input();
            input.recent_n = bad;
            let err = validate(&input).unwrap_err();
            assert_eq!(err.to_string(), "Value must be between 0 and 100.");
        }
    }

    #[test]
    fn rejects_out_of_range_top_k() {
        for bad in [-1, TOP_K_MAX + 1] {
            let mut input = valid_input();
            input.top_k = bad;
            let err = validate(&input).unwrap_err();
            assert_eq!(err.to_string(), "Value must be between 0 and 50.");
        }
    }
}
