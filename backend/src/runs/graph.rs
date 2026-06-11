//! Pure translation of the `FlowGraph` (F07) into the executor's DAG view.
//!
//! Parses the inline graph body from the run request, extracts each node's
//! `data.agentId`, and derives:
//! - `agent_of` — `nodeId → agentId`
//! - `adjacency` — downstream successors per node, in declared edge order
//! - `in_degree` — upstream count per node
//! - `root` — the single declared root (validated to exist)
//! - `terminals` — nodes with no outgoing edge
//!
//! Validates: non-empty, exactly one `rootNodeId` and it is a real node, every
//! edge endpoint resolves, no self-loops, no cycles. Anything else rejects
//! with `AppError::RunInvalidGraph`. Pre-resolution of agent ownership is the
//! service's job; this layer never touches the database.

use std::collections::{HashMap, HashSet, VecDeque};

use serde_json::Value;
use uuid::Uuid;

use crate::error::AppError;

/// Executor-side view of a flow graph, derived once before scheduling.
#[derive(Debug, Clone)]
pub struct Dag {
    /// Node ids in declared order.
    pub nodes: Vec<String>,
    /// `nodeId → agentId`.
    pub agent_of: HashMap<String, Uuid>,
    /// Downstream successors per node, in declared edge order (deterministic
    /// for multi-upstream concatenation).
    pub adjacency: HashMap<String, Vec<String>>,
    /// Upstream count per node — scheduler readiness gate.
    pub in_degree: HashMap<String, usize>,
    /// Upstream predecessors per node, in declared edge order. Used to
    /// concatenate forwarded inputs deterministically.
    pub predecessors: HashMap<String, Vec<String>>,
    /// The single declared root.
    pub root: String,
    /// Nodes with no outgoing edge; their outputs feed the aggregated result.
    pub terminals: Vec<String>,
}

/// Translate a raw graph body into the executor's DAG view. Reject with
/// `RunInvalidGraph` for any structural violation that would prevent
/// dependency-ordered execution.
pub fn translate(graph: &Value) -> Result<Dag, AppError> {
    let obj = graph.as_object().ok_or(AppError::RunInvalidGraph)?;

    let nodes_raw = obj
        .get("nodes")
        .and_then(Value::as_array)
        .ok_or(AppError::RunInvalidGraph)?;
    let edges_raw = obj
        .get("edges")
        .and_then(Value::as_array)
        .ok_or(AppError::RunInvalidGraph)?;

    if nodes_raw.is_empty() {
        return Err(AppError::RunInvalidGraph);
    }

    // Build the node list and `agentId` map.
    let mut nodes = Vec::with_capacity(nodes_raw.len());
    let mut node_ids = HashSet::with_capacity(nodes_raw.len());
    let mut agent_of = HashMap::with_capacity(nodes_raw.len());
    for node in nodes_raw {
        let node_obj = node.as_object().ok_or(AppError::RunInvalidGraph)?;
        let id = node_obj
            .get("id")
            .and_then(Value::as_str)
            .ok_or(AppError::RunInvalidGraph)?
            .to_string();
        if !node_ids.insert(id.clone()) {
            return Err(AppError::RunInvalidGraph);
        }
        let agent_id_str = node_obj
            .get("data")
            .and_then(Value::as_object)
            .and_then(|d| d.get("agentId"))
            .and_then(Value::as_str)
            .ok_or(AppError::RunInvalidGraph)?;
        let agent_id = Uuid::parse_str(agent_id_str).map_err(|_| AppError::RunInvalidGraph)?;
        nodes.push(id.clone());
        agent_of.insert(id, agent_id);
    }

    // Validate `rootNodeId`: required, must reference an actual node.
    let root = obj
        .get("rootNodeId")
        .and_then(Value::as_str)
        .ok_or(AppError::RunInvalidGraph)?
        .to_string();
    if !node_ids.contains(&root) {
        return Err(AppError::RunInvalidGraph);
    }

    // Seed adjacency/in-degree/predecessors for every node so missing entries
    // don't represent "node absent" later.
    let mut adjacency: HashMap<String, Vec<String>> = HashMap::with_capacity(nodes.len());
    let mut in_degree: HashMap<String, usize> = HashMap::with_capacity(nodes.len());
    let mut predecessors: HashMap<String, Vec<String>> = HashMap::with_capacity(nodes.len());
    for id in &nodes {
        adjacency.insert(id.clone(), Vec::new());
        in_degree.insert(id.clone(), 0);
        predecessors.insert(id.clone(), Vec::new());
    }

    for edge in edges_raw {
        let edge_obj = edge.as_object().ok_or(AppError::RunInvalidGraph)?;
        let source = edge_obj
            .get("source")
            .and_then(Value::as_str)
            .ok_or(AppError::RunInvalidGraph)?
            .to_string();
        let target = edge_obj
            .get("target")
            .and_then(Value::as_str)
            .ok_or(AppError::RunInvalidGraph)?
            .to_string();

        if source == target {
            return Err(AppError::RunInvalidGraph);
        }
        if !node_ids.contains(&source) || !node_ids.contains(&target) {
            return Err(AppError::RunInvalidGraph);
        }

        adjacency.get_mut(&source).unwrap().push(target.clone());
        predecessors.get_mut(&target).unwrap().push(source);
        *in_degree.get_mut(&target).unwrap() += 1;
    }

    // The graph must be a single-rooted DAG: only the declared root may have
    // in-degree zero, and Kahn's algorithm must visit every node.
    for id in &nodes {
        if *in_degree.get(id).unwrap() == 0 && id != &root {
            return Err(AppError::RunInvalidGraph);
        }
    }

    // Kahn's algorithm — cycle detection. Seed with every in-degree-zero
    // node; the single-root rule above guarantees that, in an acyclic graph,
    // this is exactly `root`. A graph with a cycle reaches here with no
    // in-degree-zero node and the visit count falls short.
    let mut remaining: HashMap<String, usize> = in_degree.clone();
    let mut queue: VecDeque<String> = nodes
        .iter()
        .filter(|id| remaining[*id] == 0)
        .cloned()
        .collect();
    let mut visited = 0usize;
    while let Some(id) = queue.pop_front() {
        visited += 1;
        for next in adjacency.get(&id).unwrap() {
            let entry = remaining.get_mut(next).unwrap();
            *entry -= 1;
            if *entry == 0 {
                queue.push_back(next.clone());
            }
        }
    }
    if visited != nodes.len() {
        return Err(AppError::RunInvalidGraph);
    }

    let terminals: Vec<String> = nodes
        .iter()
        .filter(|id| adjacency.get(*id).unwrap().is_empty())
        .cloned()
        .collect();

    Ok(Dag {
        nodes,
        agent_of,
        adjacency,
        in_degree,
        predecessors,
        root,
        terminals,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn agent(id: u8) -> String {
        Uuid::from_bytes([id; 16]).to_string()
    }

    fn node(id: &str, agent_id_byte: u8) -> Value {
        json!({ "id": id, "data": { "agentId": agent(agent_id_byte) } })
    }

    fn edge(source: &str, target: &str) -> Value {
        json!({ "id": format!("{source}-{target}"), "source": source, "target": target })
    }

    #[test]
    fn translates_linear_graph_to_dag() {
        let graph = json!({
            "nodes": [node("n1", 1), node("n2", 2), node("n3", 3)],
            "edges": [edge("n1", "n2"), edge("n2", "n3")],
            "rootNodeId": "n1"
        });
        let dag = translate(&graph).unwrap();
        assert_eq!(dag.root, "n1");
        assert_eq!(dag.terminals, vec!["n3".to_string()]);
        assert_eq!(dag.in_degree["n1"], 0);
        assert_eq!(dag.in_degree["n2"], 1);
        assert_eq!(dag.in_degree["n3"], 1);
        assert_eq!(dag.adjacency["n1"], vec!["n2".to_string()]);
        assert_eq!(dag.adjacency["n3"], Vec::<String>::new());
        assert_eq!(dag.agent_of["n1"], Uuid::from_bytes([1; 16]));
    }

    #[test]
    fn rejects_cycle() {
        let graph = json!({
            "nodes": [node("n1", 1), node("n2", 2)],
            "edges": [edge("n1", "n2"), edge("n2", "n1")],
            "rootNodeId": "n1"
        });
        let err = translate(&graph).unwrap_err();
        assert_eq!(err.code(), "RUN001");
    }

    #[test]
    fn rejects_self_loop() {
        let graph = json!({
            "nodes": [node("n1", 1)],
            "edges": [edge("n1", "n1")],
            "rootNodeId": "n1"
        });
        assert_eq!(translate(&graph).unwrap_err().code(), "RUN001");
    }

    #[test]
    fn rejects_missing_root() {
        let graph = json!({
            "nodes": [node("n1", 1), node("n2", 2)],
            "edges": [edge("n1", "n2")],
            "rootNodeId": null
        });
        assert_eq!(translate(&graph).unwrap_err().code(), "RUN001");
    }

    #[test]
    fn rejects_root_not_in_nodes() {
        let graph = json!({
            "nodes": [node("n1", 1)],
            "edges": [],
            "rootNodeId": "ghost"
        });
        assert_eq!(translate(&graph).unwrap_err().code(), "RUN001");
    }

    #[test]
    fn rejects_multiple_roots() {
        // Two zero-in-degree nodes, one declared root → the other is an extra root.
        let graph = json!({
            "nodes": [node("n1", 1), node("n2", 2), node("n3", 3)],
            "edges": [edge("n1", "n3")],
            "rootNodeId": "n1"
        });
        assert_eq!(translate(&graph).unwrap_err().code(), "RUN001");
    }

    #[test]
    fn rejects_empty_graph() {
        let graph = json!({ "nodes": [], "edges": [], "rootNodeId": null });
        assert_eq!(translate(&graph).unwrap_err().code(), "RUN001");
    }

    #[test]
    fn rejects_duplicate_node_ids() {
        let graph = json!({
            "nodes": [node("n1", 1), node("n1", 2)],
            "edges": [],
            "rootNodeId": "n1"
        });
        assert_eq!(translate(&graph).unwrap_err().code(), "RUN001");
    }

    #[test]
    fn rejects_edge_to_unknown_node() {
        let graph = json!({
            "nodes": [node("n1", 1)],
            "edges": [edge("n1", "ghost")],
            "rootNodeId": "n1"
        });
        assert_eq!(translate(&graph).unwrap_err().code(), "RUN001");
    }

    #[test]
    fn derives_terminals_for_diamond() {
        let graph = json!({
            "nodes": [node("n1", 1), node("n2", 2), node("n3", 3), node("n4", 4)],
            "edges": [
                edge("n1", "n2"),
                edge("n1", "n3"),
                edge("n2", "n4"),
                edge("n3", "n4"),
            ],
            "rootNodeId": "n1"
        });
        let dag = translate(&graph).unwrap();
        assert_eq!(dag.terminals, vec!["n4".to_string()]);
        assert_eq!(dag.in_degree["n2"], 1);
        assert_eq!(dag.in_degree["n3"], 1);
        assert_eq!(dag.in_degree["n4"], 2);
        // Predecessors are recorded in declared edge order (n2 first, then n3).
        assert_eq!(dag.predecessors["n4"], vec!["n2".to_string(), "n3".to_string()]);
    }

    #[test]
    fn predecessors_preserve_edge_order() {
        let graph = json!({
            "nodes": [node("n1", 1), node("n2", 2), node("n3", 3)],
            "edges": [edge("n2", "n3"), edge("n1", "n3"), edge("n1", "n2")],
            "rootNodeId": "n1"
        });
        let dag = translate(&graph).unwrap();
        assert_eq!(dag.predecessors["n3"], vec!["n2".to_string(), "n1".to_string()]);
    }
}
