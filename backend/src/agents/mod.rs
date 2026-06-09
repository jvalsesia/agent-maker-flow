//! Agents (F04).
//!
//! Reusable LLM behavior profiles owned by a user: name, preamble, system
//! prompt, provider/model, and the recent-N / top-K history and retrieval caps.
//! The system of record consumed by F07 (Flow Canvas) and F09 (Flow Execution).
//!
//! This stage provides the data model and the owner-scoped sqlx repository; the
//! validation/uniqueness/gateway service and HTTP routes are added in later
//! F04 stages.

pub mod model;
pub mod repo;

pub use model::{Agent, AgentInput};
