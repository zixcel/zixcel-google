use crate::{Error, KeyQuery, Result, inventory::query::InventoryQuery};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub schema: String,
    pub id: String,
    pub operation: String,
    pub arguments: BTreeMap<String, String>,
    pub limits: Limits,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    pub timeout_ms: u64,
    pub maximum_response_bytes: usize,
}
#[derive(Serialize)]
pub struct Response {
    pub schema: &'static str,
    pub id: String,
    pub operation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<&'static str>,
}
pub(super) enum Input {
    Key(KeyQuery, Option<String>),
    Inventory(InventoryQuery),
}
impl Request {
    pub fn validate(&self) -> Result<()> {
        if self.schema != super::SCHEMA
            || self.id.len() != 32
            || !self
                .id
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
            || !(50..=40000).contains(&self.limits.timeout_ms)
            || !(1024..=131072).contains(&self.limits.maximum_response_bytes)
        {
            return Err(Error::Protocol);
        }
        self.input().map(|_| ())
    }
    pub(super) fn input(&self) -> Result<Input> {
        if self.operation != super::OPERATION {
            return Ok(Input::Inventory(InventoryQuery::parse(
                &self.operation,
                &self.arguments,
            )?));
        }
        let mut arguments = self.arguments.clone();
        let quota = arguments.remove("quota-project");
        if let Some(project) = &quota {
            crate::inventory::query::project_identifier(project)?;
        }
        let query: KeyQuery = serde_json::from_value(
            serde_json::to_value(arguments).map_err(|_| Error::InvalidInput)?,
        )
        .map_err(|_| Error::InvalidInput)?;
        query.validate()?;
        Ok(Input::Key(query, quota))
    }
}
