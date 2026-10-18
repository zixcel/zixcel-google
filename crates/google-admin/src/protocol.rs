use crate::{Error, GoogleHttp, Result, inspect_key, inventory};
use serde_json::{Value, json};
mod model;
use model::Input;
pub use model::{Limits, Request, Response};
pub const SCHEMA: &str = "zix-provider/v1";
pub const OPERATION: &str = "iam.keys.inspect";
fn run(request: &Request) -> Result<Value> {
    request.validate()?;
    let input = request.input()?;
    let quota = match &input {
        Input::Key(_, q) => q.as_deref(),
        Input::Inventory(q) => q.quota_project.as_deref(),
        Input::Export(q) => q.quota_project.as_deref(),
    };
    let mut http = GoogleHttp::from_env()?;
    http.limit_time(std::time::Duration::from_millis(request.limits.timeout_ms))?;
    if let Some(project) = quota {
        http.set_quota_project(project)?;
    }
    let value = match input {
        Input::Key(query, _) => {
            serde_json::to_value(inspect_key(&mut http, &query, chrono::Utc::now())?)
        }
        Input::Inventory(query) if query.operation.starts_with("overview.") => {
            serde_json::to_value(inventory::overview(&mut http, &query)?)
        }
        Input::Inventory(query) => serde_json::to_value(inventory::inspect(&mut http, &query)?),
        Input::Export(query) => Ok(crate::exports::execute(&mut http, &query)?),
    }
    .map_err(|_| Error::InvalidResponse)?;
    bounded_value(value, request.limits.maximum_response_bytes)
}
fn bounded_value(value: Value, maximum: usize) -> Result<Value> {
    if serde_json::to_vec(&value)
        .map_err(|_| Error::InvalidResponse)?
        .len()
        + 512
        > maximum
    {
        return Err(Error::ResponseTooLarge);
    }
    Ok(value)
}
pub fn execute(request: &Request) -> Response {
    let (result, error, http_status) = match run(request) {
        Ok(value) => (Some(value), None, None),
        Err(e) => (None, Some(e.code()), e.http_status()),
    };
    Response {
        schema: SCHEMA,
        id: request.id.clone(),
        operation: request.operation.clone(),
        result,
        error,
        http_status,
    }
}
pub fn manifest() -> Value {
    let mut commands = inventory::manifest::commands();
    commands.extend(crate::exports::commands());
    commands.push(json!({"id":OPERATION,"words":["iam","keys","inspect"],"effect":"read",
        "description":"Observe one service-account key without using or changing it.",
        "required_permissions":["iam.serviceAccounts.get","iam.serviceAccountKeys.list"],
        "arguments":[{"name":"project","required":true,"maximum":30},{"name":"service-account","required":true,"maximum":128},
            {"name":"key-id","required":true,"maximum":40},{"name":"quota-project","required":false,"maximum":30}]}));
    json!({"schema":"zix-extension/v1","package":"zixcel-google-admin","version":env!("CARGO_PKG_VERSION"),"namespace":"google",
        "platform":"linux-x86_64","protocol":SCHEMA,"credential_environment":["ZIX_GOOGLE_ACCESS_TOKEN"],"timeout_ms":35000,"commands":commands})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn output_budget_reserves_the_bound_protocol_envelope() {
        assert!(bounded_value(json!({"records":[]}), 1024).is_ok());
        assert_eq!(
            bounded_value(json!({"name":"x".repeat(1024)}), 1024).unwrap_err(),
            Error::ResponseTooLarge
        );
    }
}
