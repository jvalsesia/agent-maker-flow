//! Topological execution engine for a run.
//!
//! Drives a `Dag`: keeps every node with in-degree zero ready to run, kicks
//! off ready nodes concurrently through a [`tokio::task::JoinSet`] bounded by
//! [`MAX_CONCURRENCY`], and when a node finishes either marks its
//! successors ready (decrementing their in-degree) or, on failure, marks
//! their transitive descendants `Skipped` so unaffected branches keep going.
//!
//! Each node's execution:
//! 1. Resolve the pre-fetched agent config (failure → `RunAgentMissing`).
//! 2. Build the node's forwarded input — the run prompt at the root, the
//!    deterministic concatenation of upstream outputs everywhere else.
//! 3. Run F06 retrieval against that input using the agent's `top_k`.
//! 4. Assemble messages — `[system] + recent_n(history) + [user]`, where the
//!    user content is the retrieved context concatenated ahead of the
//!    forwarded input.
//! 5. Call F03 [`GatewayClient::complete`] with the agent's model.
//! 6. Emit `node.started`, one `node.partial` (full text, single chunk in
//!    this version), then `node.completed`.
//!
//! Terminal output is the deterministic concatenation of every terminal
//! node's output, separated by blank lines. The `run.finished` event carries
//! overall status (`failed` if any node failed) plus the lists of failed and
//! skipped node ids.

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};

use chrono::Utc;
use serde_json::{Map, Value};
use sqlx::PgPool;
use tokio::task::JoinSet;
use uuid::Uuid;

use crate::agents::Agent;
use crate::gateway::types::{ChatMessage, CompletionRequest};
use crate::gateway::GatewayClient;
use crate::memory::retrieval;
use crate::runs::graph::Dag;
use crate::runs::model::{
    ExecutionEvent, HistoryTurn, NodeError, NodeOutcome, RetrievalSummary, RunStatus, UsageSummary,
};
use crate::runs::registry::RunRegistry;

/// Cap on parallel node executions per run. Bounded so a wide diamond does
/// not flood the gateway.
pub const MAX_CONCURRENCY: usize = 8;

/// Per-node outputs collected during a run; consulted for downstream
/// forwarding and the terminal aggregation.
type NodeOutputs = HashMap<String, String>;

/// What a finished node task reports back to the scheduler.
struct NodeReport {
    node_id: String,
    outcome: NodeOutcome,
    output: Option<String>,
}

/// Pull text out of an upstream output, falling back to the empty string so
/// a skipped/failed upstream contributes nothing.
fn forwarded_input(predecessors: &[String], outputs: &NodeOutputs, prompt: &str) -> String {
    if predecessors.is_empty() {
        return prompt.to_string();
    }
    predecessors
        .iter()
        .filter_map(|id| outputs.get(id).cloned())
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Concatenate retrieved memory records into a single context blob — ahead
/// of the forwarded upstream output, per F06.
fn retrieval_context(records: &[retrieval::RetrievedRecord]) -> String {
    records
        .iter()
        .map(|r| r.text.clone())
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Build the `messages` array for one completion request.
pub(crate) fn assemble_messages(
    agent: &Agent,
    history: &[HistoryTurn],
    retrieval_text: &str,
    forwarded: &str,
) -> Vec<ChatMessage> {
    let mut system = String::new();
    if let Some(p) = &agent.preamble {
        if !p.is_empty() {
            system.push_str(p);
            system.push_str("\n\n");
        }
    }
    system.push_str(&agent.system_prompt);

    let mut messages = Vec::with_capacity(history.len() + 2);
    messages.push(ChatMessage {
        role: "system".to_string(),
        content: system,
    });
    let keep = (agent.recent_n.max(0) as usize).min(history.len());
    let start = history.len() - keep;
    for turn in &history[start..] {
        messages.push(ChatMessage {
            role: turn.role.clone(),
            content: turn.content.clone(),
        });
    }
    let mut user = String::new();
    if !retrieval_text.is_empty() {
        user.push_str(retrieval_text);
        user.push_str("\n\n");
    }
    user.push_str(forwarded);
    messages.push(ChatMessage {
        role: "user".to_string(),
        content: user,
    });
    messages
}

/// Build the deterministic terminal aggregation.
pub(crate) fn aggregate_terminal_output(terminals: &[String], outputs: &NodeOutputs) -> String {
    terminals
        .iter()
        .filter_map(|id| outputs.get(id).cloned())
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Walk `failed` and the adjacency map to find every transitive descendant.
/// Used so a downstream task can be marked `Skipped` (with the upstream that
/// caused it recorded) instead of started.
fn descendants_of(failed: &str, adjacency: &HashMap<String, Vec<String>>) -> Vec<String> {
    let mut out = Vec::new();
    let mut queue: VecDeque<String> = adjacency
        .get(failed)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .collect();
    let mut seen: HashSet<String> = HashSet::new();
    while let Some(id) = queue.pop_front() {
        if !seen.insert(id.clone()) {
            continue;
        }
        out.push(id.clone());
        if let Some(next) = adjacency.get(&id) {
            for n in next {
                queue.push_back(n.clone());
            }
        }
    }
    out
}

/// What the engine needs to execute one node — agents are pre-resolved by
/// the service so the engine never reads from the database for lookups.
pub struct EngineInputs {
    pub run_id: Uuid,
    pub owner_id: String,
    pub flow_id: Option<Uuid>,
    pub prompt: String,
    pub history: Arc<Vec<HistoryTurn>>,
    pub dag: Dag,
    pub agents: Arc<HashMap<String, Agent>>,
}

/// Drive a run to completion: emit events through the registry, return when
/// every node has reached a terminal state and `run.finished` has been
/// emitted. The function never errors — every failure is reported as a node
/// event and reflected in `run.finished`.
pub async fn execute(
    inputs: EngineInputs,
    db: PgPool,
    gateway: Arc<GatewayClient>,
    registry: Arc<RunRegistry>,
) {
    let EngineInputs {
        run_id,
        owner_id,
        flow_id,
        prompt,
        history,
        dag,
        agents,
    } = inputs;

    registry.append(
        run_id,
        ExecutionEvent::RunStarted {
            run_id,
            flow_id,
            node_count: dag.nodes.len(),
            root_node_id: dag.root.clone(),
            started_at: Utc::now(),
        },
    );

    let outputs: Arc<Mutex<NodeOutputs>> = Arc::new(Mutex::new(NodeOutputs::new()));
    let mut remaining_in_degree: HashMap<String, usize> = dag.in_degree.clone();
    let mut completed: HashMap<String, NodeOutcome> = HashMap::new();
    let mut failed_nodes: Vec<String> = Vec::new();
    let mut skipped_nodes: Vec<String> = Vec::new();

    // Initial ready set — every node with no upstream dependency.
    let mut ready: VecDeque<String> = dag
        .nodes
        .iter()
        .filter(|id| remaining_in_degree[*id] == 0)
        .cloned()
        .collect();

    let mut join: JoinSet<NodeReport> = JoinSet::new();

    loop {
        while join.len() < MAX_CONCURRENCY {
            let Some(node_id) = ready.pop_front() else {
                break;
            };
            let agent = match agents.get(&node_id) {
                Some(a) => a.clone(),
                None => {
                    // Defensive — should never happen because the service
                    // pre-resolves every node. Treat as a node failure.
                    let err = NodeError {
                        code: "RUN003".to_string(),
                        message: "agent missing".to_string(),
                    };
                    registry.append(
                        run_id,
                        ExecutionEvent::NodeFailed {
                            run_id,
                            node_id: node_id.clone(),
                            error: err,
                            failed_at: Utc::now(),
                        },
                    );
                    completed.insert(node_id.clone(), NodeOutcome::Failed);
                    failed_nodes.push(node_id.clone());
                    for d in descendants_of(&node_id, &dag.adjacency) {
                        if completed.contains_key(&d) {
                            continue;
                        }
                        registry.append(
                            run_id,
                            ExecutionEvent::NodeSkipped {
                                run_id,
                                node_id: d.clone(),
                                reason: "upstream failed".to_string(),
                                upstream_node_id: node_id.clone(),
                            },
                        );
                        completed.insert(d.clone(), NodeOutcome::Skipped);
                        skipped_nodes.push(d);
                    }
                    continue;
                }
            };
            let predecessors = dag.predecessors.get(&node_id).cloned().unwrap_or_default();
            let forwarded = {
                let guard = outputs.lock().expect("outputs lock poisoned");
                forwarded_input(&predecessors, &guard, &prompt)
            };

            let db = db.clone();
            let gateway = gateway.clone();
            let registry = registry.clone();
            let outputs = outputs.clone();
            let owner_id = owner_id.clone();
            let history = history.clone();
            let node_id_for_task = node_id.clone();
            let run_id_for_task = run_id;

            join.spawn(async move {
                execute_node(
                    run_id_for_task,
                    node_id_for_task,
                    owner_id,
                    agent,
                    forwarded,
                    history,
                    db,
                    gateway,
                    registry,
                    outputs,
                )
                .await
            });
        }

        if join.is_empty() {
            break;
        }

        let report = match join.join_next().await {
            Some(Ok(report)) => report,
            Some(Err(_join_err)) => {
                // Task panicked. We don't know which node — try to fall
                // through and finish; any node still pending will eventually
                // stall this loop, so break to avoid that.
                break;
            }
            None => break,
        };

        completed.insert(report.node_id.clone(), report.outcome);
        match report.outcome {
            NodeOutcome::Completed => {
                if let Some(output) = report.output {
                    let mut guard = outputs.lock().expect("outputs lock poisoned");
                    guard.insert(report.node_id.clone(), output);
                }
                for successor in dag.adjacency.get(&report.node_id).cloned().unwrap_or_default() {
                    let entry = remaining_in_degree.get_mut(&successor).unwrap();
                    *entry -= 1;
                    if *entry == 0 && !completed.contains_key(&successor) {
                        ready.push_back(successor);
                    }
                }
            }
            NodeOutcome::Failed => {
                failed_nodes.push(report.node_id.clone());
                for d in descendants_of(&report.node_id, &dag.adjacency) {
                    if completed.contains_key(&d) {
                        continue;
                    }
                    registry.append(
                        run_id,
                        ExecutionEvent::NodeSkipped {
                            run_id,
                            node_id: d.clone(),
                            reason: "upstream failed".to_string(),
                            upstream_node_id: report.node_id.clone(),
                        },
                    );
                    completed.insert(d.clone(), NodeOutcome::Skipped);
                    skipped_nodes.push(d);
                }
            }
            NodeOutcome::Skipped => {
                // Unreachable from inside the join loop: skips are emitted
                // synchronously when a parent fails.
            }
        }
    }

    let outputs_final = outputs.lock().expect("outputs lock poisoned").clone();
    let aggregated = aggregate_terminal_output(&dag.terminals, &outputs_final);
    let status = if failed_nodes.is_empty() {
        RunStatus::Succeeded
    } else {
        RunStatus::Failed
    };
    let aggregated_output = if aggregated.is_empty() {
        None
    } else {
        Some(aggregated.clone())
    };

    registry.append(
        run_id,
        ExecutionEvent::RunFinished {
            run_id,
            status,
            output: aggregated_output.clone(),
            failed_nodes,
            skipped_nodes,
            finished_at: Utc::now(),
        },
    );
    registry.finish(run_id, status, aggregated_output);
}

#[allow(clippy::too_many_arguments)]
async fn execute_node(
    run_id: Uuid,
    node_id: String,
    owner_id: String,
    agent: Agent,
    forwarded: String,
    history: Arc<Vec<HistoryTurn>>,
    db: PgPool,
    gateway: Arc<GatewayClient>,
    registry: Arc<RunRegistry>,
    outputs: Arc<Mutex<NodeOutputs>>,
) -> NodeReport {
    let _ = outputs; // outputs is shared but written by the scheduler loop only

    registry.append(
        run_id,
        ExecutionEvent::NodeStarted {
            run_id,
            node_id: node_id.clone(),
            agent_id: agent.id,
            agent_name: agent.name.clone(),
            model: agent.model.clone(),
            started_at: Utc::now(),
        },
    );

    let retrieval_outcome = retrieval::retrieve(
        &db,
        &gateway,
        &owner_id,
        agent.id,
        agent.top_k,
        &forwarded,
    )
    .await;
    let retrieval_text = retrieval_context(&retrieval_outcome.records);
    let messages = assemble_messages(&agent, &history, &retrieval_text, &forwarded);

    let params: Map<String, Value> = Map::new();
    let req = CompletionRequest {
        model: agent.model.clone(),
        messages,
        params,
    };

    match gateway.complete(req).await {
        Ok(result) => {
            registry.append(
                run_id,
                ExecutionEvent::NodePartial {
                    run_id,
                    node_id: node_id.clone(),
                    delta: result.content.clone(),
                },
            );
            registry.append(
                run_id,
                ExecutionEvent::NodeCompleted {
                    run_id,
                    node_id: node_id.clone(),
                    output: result.content.clone(),
                    retrieval: RetrievalSummary {
                        retrieved_count: retrieval_outcome.retrieved_count,
                        excluded_mismatched: retrieval_outcome.excluded_mismatched,
                        status: retrieval_outcome.status.clone(),
                    },
                    usage: UsageSummary {
                        prompt_tokens: result.prompt_tokens,
                        completion_tokens: result.completion_tokens,
                        cost_usd: result.cost_usd,
                        cached: result.cached,
                    },
                    completed_at: Utc::now(),
                },
            );
            NodeReport {
                node_id,
                outcome: NodeOutcome::Completed,
                output: Some(result.content),
            }
        }
        Err(err) => {
            let node_error = NodeError {
                code: err.code().to_string(),
                message: err.to_string(),
            };
            registry.append(
                run_id,
                ExecutionEvent::NodeFailed {
                    run_id,
                    node_id: node_id.clone(),
                    error: node_error,
                    failed_at: Utc::now(),
                },
            );
            NodeReport {
                node_id,
                outcome: NodeOutcome::Failed,
                output: None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::retrieval::{RetrievalStatus, RetrievedRecord};
    use chrono::Utc;
    use uuid::Uuid;

    fn agent(name: &str, recent_n: i32) -> Agent {
        Agent {
            id: Uuid::new_v4(),
            name: name.to_string(),
            preamble: Some("be helpful".into()),
            system_prompt: "You are a test agent.".to_string(),
            provider: "openai".to_string(),
            model: "gpt-4o".to_string(),
            recent_n,
            top_k: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn turn(role: &str, content: &str) -> HistoryTurn {
        HistoryTurn {
            role: role.to_string(),
            content: content.to_string(),
        }
    }

    #[test]
    fn forwarded_input_is_prompt_at_root() {
        let outputs = NodeOutputs::new();
        let f = forwarded_input(&[], &outputs, "hi");
        assert_eq!(f, "hi");
    }

    #[test]
    fn forwarded_input_uses_single_upstream_output() {
        let mut outputs = NodeOutputs::new();
        outputs.insert("n1".into(), "upstream text".into());
        let f = forwarded_input(&["n1".into()], &outputs, "ignored");
        assert_eq!(f, "upstream text");
    }

    #[test]
    fn concatenates_multiple_upstream_outputs_deterministically() {
        let mut outputs = NodeOutputs::new();
        outputs.insert("n2".into(), "B".into());
        outputs.insert("n3".into(), "C".into());
        // predecessors order (declared) drives concat order.
        let f = forwarded_input(&["n2".into(), "n3".into()], &outputs, "ignored");
        assert_eq!(f, "B\n\nC");
    }

    #[test]
    fn forwarded_input_skips_missing_upstream() {
        // A failed/skipped upstream contributes nothing; siblings still flow.
        let mut outputs = NodeOutputs::new();
        outputs.insert("n3".into(), "C".into());
        let f = forwarded_input(&["n2".into(), "n3".into()], &outputs, "ignored");
        assert_eq!(f, "C");
    }

    #[test]
    fn assembles_messages_with_retrieval_ahead_of_forwarded() {
        let a = agent("Researcher", 0);
        let msgs = assemble_messages(&a, &[], "MEM-CTX", "FWD");
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].role, "system");
        assert!(msgs[0].content.starts_with("be helpful"));
        assert!(msgs[0].content.contains("You are a test agent."));
        assert_eq!(msgs[1].role, "user");
        assert!(msgs[1].content.starts_with("MEM-CTX\n\n"));
        assert!(msgs[1].content.ends_with("FWD"));
    }

    #[test]
    fn assembles_messages_caps_history_at_recent_n() {
        let a = agent("R", 2);
        let history = vec![
            turn("user", "u1"),
            turn("assistant", "a1"),
            turn("user", "u2"),
            turn("assistant", "a2"),
            turn("user", "u3"),
        ];
        let msgs = assemble_messages(&a, &history, "", "FWD");
        // system + 2 history turns + user
        assert_eq!(msgs.len(), 4);
        // Tail of history kept.
        assert_eq!(msgs[1].content, "a2");
        assert_eq!(msgs[2].content, "u3");
    }

    #[test]
    fn assembles_messages_drops_empty_retrieval() {
        let a = agent("R", 0);
        let msgs = assemble_messages(&a, &[], "", "FWD");
        assert_eq!(msgs[1].content, "FWD");
    }

    #[test]
    fn aggregates_terminal_outputs_in_declared_order() {
        let mut outputs = NodeOutputs::new();
        outputs.insert("n4".into(), "X".into());
        outputs.insert("n5".into(), "Y".into());
        let agg = aggregate_terminal_output(&["n4".into(), "n5".into()], &outputs);
        assert_eq!(agg, "X\n\nY");
    }

    #[test]
    fn descendants_of_walks_transitively() {
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();
        adj.insert("n1".into(), vec!["n2".into(), "n3".into()]);
        adj.insert("n2".into(), vec!["n4".into()]);
        adj.insert("n3".into(), vec!["n4".into()]);
        adj.insert("n4".into(), vec![]);
        let mut descs = descendants_of("n1", &adj);
        descs.sort();
        assert_eq!(descs, vec!["n2", "n3", "n4"]);
    }

    #[test]
    fn retrieval_context_is_records_joined() {
        let records = vec![
            RetrievedRecord {
                id: Uuid::new_v4(),
                text: "A".into(),
                score: 0.9,
            },
            RetrievedRecord {
                id: Uuid::new_v4(),
                text: "B".into(),
                score: 0.8,
            },
        ];
        assert_eq!(retrieval_context(&records), "A\n\nB");
        assert_eq!(retrieval_context(&[]), "");
    }

    // Smoke-test that the wire payload still matches the F06 status shape.
    #[test]
    fn retrieval_summary_matches_f06_shape() {
        let summary = RetrievalSummary {
            retrieved_count: 0,
            excluded_mismatched: 0,
            status: RetrievalStatus::Ok,
        };
        let v = serde_json::to_value(&summary).unwrap();
        assert_eq!(v["retrieved_count"], 0);
        assert_eq!(v["status"]["kind"], "ok");
    }
}
