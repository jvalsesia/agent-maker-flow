//! Flows (F08).
//!
//! Named, per-user saved flows: a name plus the complete F07 `FlowGraph`
//! (nodes with agent references + positions, edges, root) persisted verbatim in
//! a single `jsonb` column so it round-trips for F07 reload and F09 execution
//! with no transformation. The persistence seam between F07 (produces the
//! graph) and F09 (executes a saved graph).
//!
//! The data model, the owner-scoped sqlx repository, and the validation /
//! uniqueness service; the protected CRUD + rename endpoints live in
//! `routes/flows.rs`.

pub mod model;
pub mod repo;
pub mod service;

pub use model::{Flow, FlowGraph, FlowInput, FlowSummary, RenameInput};
