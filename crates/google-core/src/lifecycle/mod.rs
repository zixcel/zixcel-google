//! Explicit Google lifecycle plans and evidence classification, never HTTP or authority.
mod plan;
mod reconcile;
pub use plan::{Plan, Request, Target, catalog, prepare};
pub use reconcile::{Evidence, Reconciliation, State, reconcile};
pub const SCHEMA: &str = "zix-google-lifecycle/v1";
