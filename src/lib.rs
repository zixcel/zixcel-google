#![forbid(unsafe_code)]
#![doc = "Planning-only Google connector with closed, credential-free input."]

mod boundary;
mod config;
mod plan;
mod reports;
#[cfg(test)]
mod unit_tests;

pub use boundary::ConnectorError;
pub use config::{ConnectorConfig, parse_config};
pub use plan::{ConnectorPlan, PlanStep, build_plan};
pub use reports::{
    CapabilityReport, DoctorReport, ValidationReport, capabilities, doctor, validation_report,
};

pub const CONNECTOR: &str = "zixcel-google";
pub const PROVIDER: &str = "google";
pub const CONFIG_SCHEMA: &str = "zixcel://google/config/v1";
pub const PLAN_SCHEMA: &str = "zixcel://contracts/connector-plan/v1";
