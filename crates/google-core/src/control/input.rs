use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Account {
    pub provider: String,
    pub subject: String,
    pub account: String,
}
impl Account {
    pub(super) fn validate(&self) -> Result<()> {
        if self.provider != "google-cloud"
            || self.subject.is_empty()
            || self.subject.len() > 256
            || self.subject.chars().any(char::is_control)
        {
            return Err(Error::InvalidInput);
        }
        crate::inventory::query::project_identifier(&self.account)
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub schema: String,
    pub operation: String,
    pub account: Account,
    pub input: Map<String, Value>,
}
pub(super) fn fields(value: &Map<String, Value>, names: &[&str]) -> Result<()> {
    if value.len() != names.len() || names.iter().any(|name| !value.contains_key(*name)) {
        return Err(Error::InvalidInput);
    }
    Ok(())
}
pub(super) fn text(value: &Value, maximum: usize) -> Result<&str> {
    value
        .as_str()
        .filter(|s| !s.trim().is_empty() && s.len() <= maximum && !s.chars().any(char::is_control))
        .ok_or(Error::InvalidInput)
}
