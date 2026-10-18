//! Read existing billing exports. Never create datasets, exports or SQL jobs.
use crate::{Error, GoogleApi, GoogleRead, ReadRequest, Result};
use serde_json::{Value, json};
use std::collections::BTreeMap;
mod discovery;
mod query;
mod rows;
mod schema;
mod service;
pub use query::Query;
pub fn execute(http: &mut impl GoogleRead, query: &Query) -> Result<Value> {
    query.validate()?;
    let path = format!("/bigquery/v2/projects/{}", query.project);
    if query.operation == "billing.exports.discover" {
        discovery::discover(http, query, &path)
    } else {
        rows::read(http, query, &path)
    }
}
fn get(http: &mut impl GoogleRead, path: &str, args: Vec<(String, String)>) -> Result<Value> {
    Ok(get_with_digest(http, path, args)?.0)
}
fn get_with_digest(
    http: &mut impl GoogleRead,
    path: &str,
    args: Vec<(String, String)>,
) -> Result<(Value, String)> {
    use sha2::Digest;
    let bytes = http.read(&ReadRequest::new(GoogleApi::BigQuery, path.into(), args)?)?;
    if bytes.len() > 1024 * 1024 {
        return Err(Error::ResponseTooLarge);
    }
    let value: Value = serde_json::from_slice(&bytes).map_err(|_| Error::InvalidResponse)?;
    if !value.is_object() || value.get("error").is_some() {
        return Err(Error::InvalidResponse);
    }
    let digest = sha2::Sha256::digest(&bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    Ok((value, digest))
}
fn text(value: &Value, key: &str) -> Result<String> {
    let text = value
        .get(key)
        .and_then(Value::as_str)
        .ok_or(Error::InvalidResponse)?;
    if text.len() > 2048 || text.chars().any(char::is_control) {
        return Err(Error::InvalidResponse);
    }
    Ok(text.into())
}
fn list<'a>(value: &'a Value, key: &str) -> Result<&'a [Value]> {
    match value.get(key) {
        None => Ok(&[]),
        Some(Value::Array(v)) if v.len() <= 1000 => Ok(v),
        _ => Err(Error::InvalidResponse),
    }
}
pub fn commands() -> Vec<Value> {
    ["discover","read"].into_iter().map(|name| {
        let mut args=vec![json!({"name":"project","required":true,"maximum":30}),
            json!({"name":"quota-project","required":false,"maximum":30}),
            json!({"name":"page-token","required":false,"maximum":2048})];
        if name=="read" { for (arg,size) in [("dataset",1024),("table",128),("billing-account",20)] {
            args.push(json!({"name":arg,"required":true,"maximum":size}));
        }} else { args.push(json!({"name":"dataset","required":false,"maximum":1024})); }
        json!({"id":format!("billing.exports.{name}"),"words":["billing","exports",name],"effect":"read",
            "description":"Read one metadata or existing billing-export data page; no SQL jobs or export setup.",
            "required_permissions":if name=="read"{vec!["bigquery.tables.get","bigquery.tables.getData"]}else{vec!["bigquery.datasets.get","bigquery.tables.list"]},"arguments":args})
    }).collect()
}
