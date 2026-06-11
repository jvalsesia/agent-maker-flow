//! Flow business logic.
//!
//! The source of truth for the F08 validation rules: every save/re-save/rename
//! validates the name (trimmed, 1–80 chars) and — for save/re-save — the graph
//! structure (an object with `nodes`/`edges` arrays, every node carrying a
//! string `id`, and `rootNodeId` either null or a present node id) under a
//! serialized size cap. Per-user name uniqueness is enforced with a friendly
//! message, and the owner scope is applied before returning or mutating a row.
//! DAG/cycle enforcement stays in F07 — it is not re-checked here.

use serde_json::Value;
use std::collections::HashSet;

use sqlx::PgPool;
use uuid::Uuid;

use super::model::{Flow, FlowInput, FlowSummary, RenameInput, GRAPH_MAX_BYTES, NAME_MAX, NAME_MIN};
use super::repo;
use crate::error::AppError;

/// Machine code for a field-validation failure (422).
const VALIDATION_CODE: &str = "FLOW_VALIDATION";
/// Machine code for a per-user name collision (409).
const NAME_TAKEN_CODE: &str = "FLOW_NAME_TAKEN";

fn validation(message: impl Into<String>) -> AppError {
    AppError::Validation {
        code: VALIDATION_CODE,
        message: message.into(),
    }
}

fn name_taken(name: &str) -> AppError {
    AppError::Conflict {
        code: NAME_TAKEN_CODE,
        message: format!("A flow named '{name}' already exists."),
    }
}

/// Validate the flow name. Length is measured in characters to match the
/// `varchar(N)` column semantics; the value is validated trimmed.
fn validate_name(name: &str) -> Result<(), AppError> {
    let len = name.trim().chars().count();
    if len < NAME_MIN {
        return Err(validation("Name is required."));
    }
    if len > NAME_MAX {
        return Err(validation(format!(
            "Name must be {NAME_MAX} characters or fewer."
        )));
    }
    Ok(())
}

/// Light structural validation of the graph: an object with `nodes`/`edges`
/// arrays, each node an object with a string `id`, and `rootNodeId` null or a
/// present node id, all under the serialized size cap. Cycle/DAG rules remain
/// in F07.
fn validate_graph(graph: &Value) -> Result<(), AppError> {
    let obj = graph
        .as_object()
        .ok_or_else(|| validation("Graph must be an object."))?;

    let nodes = obj
        .get("nodes")
        .and_then(Value::as_array)
        .ok_or_else(|| validation("Graph must have a 'nodes' array."))?;

    if obj.get("edges").and_then(Value::as_array).is_none() {
        return Err(validation("Graph must have an 'edges' array."));
    }

    let mut ids = HashSet::new();
    for node in nodes {
        let id = node
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| validation("Every node must have a string 'id'."))?;
        ids.insert(id);
    }

    match obj.get("rootNodeId") {
        None | Some(Value::Null) => {}
        Some(Value::String(root)) => {
            if !ids.contains(root.as_str()) {
                return Err(validation("rootNodeId must reference an existing node."));
            }
        }
        Some(_) => return Err(validation("rootNodeId must be a string or null.")),
    }

    // Serialized size cap; rejected before the payload reaches the database.
    let bytes = serde_json::to_vec(graph)
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e).context("failed to size graph")))?;
    if bytes.len() > GRAPH_MAX_BYTES {
        return Err(validation(format!(
            "Graph exceeds the maximum size of {GRAPH_MAX_BYTES} bytes."
        )));
    }

    Ok(())
}

/// Validate the full save/re-save body (name + graph).
fn validate(input: &FlowInput) -> Result<(), AppError> {
    validate_name(&input.name)?;
    validate_graph(&input.graph)?;
    Ok(())
}

/// Save a new flow for `owner_id` after validation and a uniqueness check.
pub async fn create(db: &PgPool, owner_id: &str, mut input: FlowInput) -> Result<Flow, AppError> {
    validate(&input)?;
    input.name = input.name.trim().to_string();

    if repo::name_exists(db, owner_id, &input.name, None).await? {
        return Err(name_taken(&input.name));
    }

    repo::insert(db, owner_id, &input).await
}

/// List the caller's flows as summaries, most recently updated first.
pub async fn list(db: &PgPool, owner_id: &str) -> Result<Vec<FlowSummary>, AppError> {
    repo::list_summaries_by_owner(db, owner_id).await
}

/// Open one of the caller's flows; `NotFound` if absent or owned by another.
pub async fn get(db: &PgPool, owner_id: &str, id: Uuid) -> Result<Flow, AppError> {
    repo::get(db, owner_id, id).await?.ok_or(AppError::NotFound)
}

/// Re-save the caller's open flow after validation. The uniqueness check
/// excludes the flow's own row so re-saving under the same name is fine.
pub async fn update(
    db: &PgPool,
    owner_id: &str,
    id: Uuid,
    mut input: FlowInput,
) -> Result<Flow, AppError> {
    validate(&input)?;
    input.name = input.name.trim().to_string();

    if repo::name_exists(db, owner_id, &input.name, Some(id)).await? {
        return Err(name_taken(&input.name));
    }

    repo::update(db, owner_id, id, &input)
        .await?
        .ok_or(AppError::NotFound)
}

/// Rename the caller's flow (name only; graph untouched). The uniqueness check
/// excludes the flow's own row.
pub async fn rename(
    db: &PgPool,
    owner_id: &str,
    id: Uuid,
    input: RenameInput,
) -> Result<Flow, AppError> {
    validate_name(&input.name)?;
    let name = input.name.trim();

    if repo::name_exists(db, owner_id, name, Some(id)).await? {
        return Err(name_taken(name));
    }

    repo::rename(db, owner_id, id, name)
        .await?
        .ok_or(AppError::NotFound)
}

/// Delete one of the caller's flows; `NotFound` if absent or owned by another.
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
    use serde_json::json;

    fn valid_graph() -> Value {
        json!({
            "nodes": [
                { "id": "node-1", "type": "agent", "position": { "x": 120, "y": 80 },
                  "data": { "agentId": "11111111-1111-1111-1111-111111111111" } }
            ],
            "edges": [],
            "rootNodeId": "node-1"
        })
    }

    fn valid_input() -> FlowInput {
        FlowInput {
            name: "Research pipeline".to_string(),
            graph: valid_graph(),
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
    fn rejects_malformed_graph() {
        // Not an object.
        let mut input = valid_input();
        input.graph = json!(42);
        assert_eq!(validate(&input).unwrap_err().code(), VALIDATION_CODE);

        // Missing nodes.
        let mut input = valid_input();
        input.graph = json!({ "edges": [], "rootNodeId": null });
        assert_eq!(validate(&input).unwrap_err().code(), VALIDATION_CODE);

        // Missing edges.
        let mut input = valid_input();
        input.graph = json!({ "nodes": [], "rootNodeId": null });
        assert_eq!(validate(&input).unwrap_err().code(), VALIDATION_CODE);

        // rootNodeId references a node that does not exist.
        let mut input = valid_input();
        input.graph = json!({ "nodes": [], "edges": [], "rootNodeId": "ghost" });
        assert_eq!(validate(&input).unwrap_err().code(), VALIDATION_CODE);
    }

    #[test]
    fn rejects_oversized_graph() {
        // A node array large enough to blow past the byte cap.
        let nodes: Vec<Value> = (0..200_000)
            .map(|i| json!({ "id": format!("node-{i}") }))
            .collect();
        let mut input = valid_input();
        input.graph = json!({ "nodes": nodes, "edges": [], "rootNodeId": null });
        let err = validate(&input).unwrap_err();
        assert_eq!(err.code(), VALIDATION_CODE);
        assert!(err.to_string().contains("size"));
    }
}
