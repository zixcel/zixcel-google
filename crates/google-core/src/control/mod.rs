//! Google-specific request construction; execution and grants belong to Crowsi.
mod builders;
mod catalog;
mod input;
mod plan;
pub use catalog::{Operation, catalog};
pub use input::{Account, Request};
pub use plan::{HttpRequest, Prepared, prepare};
pub const SCHEMA: &str = "zix-google-control/v1";
