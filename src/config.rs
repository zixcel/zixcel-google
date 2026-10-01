use serde::{Deserialize, Serialize};

use crate::boundary::{identifier, reference, reject_secrets, secret_ref};
use crate::{CONFIG_SCHEMA, ConnectorError};

const SERVICES: &[&str] = &["adsense", "analytics", "calendar", "drive", "gmail"];

/// Closed Google Workspace planner configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConnectorConfig {
    pub schema: String,
    pub config_id: String,
    pub workspace_ref: String,
    pub secret_ref: String,
    pub services: Vec<String>,
}

impl ConnectorConfig {
    /// Validates schema identity and the bounded service allowlist.
    pub fn validate(&self) -> Result<(), ConnectorError> {
        if self.schema != CONFIG_SCHEMA {
            return Err(ConnectorError::new(
                "schema",
                "expected zixcel://google/config/v1",
            ));
        }
        identifier("config_id", &self.config_id)?;
        reference("workspace_ref", &self.workspace_ref)?;
        secret_ref(&self.secret_ref)?;
        if self.services.is_empty() || self.services.len() > SERVICES.len() {
            return Err(ConnectorError::new(
                "services",
                "must contain 1..=5 services",
            ));
        }
        if self
            .services
            .iter()
            .any(|item| !SERVICES.contains(&item.as_str()))
        {
            return Err(ConnectorError::new(
                "services",
                "contains an unsupported service",
            ));
        }
        Ok(())
    }

    pub(crate) fn normalized(&self) -> Self {
        let mut value = self.clone();
        value.services.sort();
        value.services.dedup();
        value
    }
}

/// Parses a bounded, closed TOML configuration.
pub fn parse_config(source: &str) -> Result<ConnectorConfig, ConnectorError> {
    reject_secrets(source)?;
    let config: ConnectorConfig = toml::from_str(source).map_err(|_| {
        ConnectorError::new(
            "config",
            "invalid TOML or fields do not match the Google v1 schema",
        )
    })?;
    config.validate()?;
    Ok(config)
}
