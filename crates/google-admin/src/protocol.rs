use crate::{Error, GoogleHttp, KeyQuery, Result, inspect_key};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
pub const SCHEMA: &str = "zix-provider/v1";
pub const OPERATION: &str = "iam.keys.inspect";
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub schema: String,
    pub id: String,
    pub operation: String,
    pub arguments: KeyQuery,
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
    pub operation: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<&'static str>,
}
impl Request {
    pub fn validate(&self) -> Result<()> {
        if self.schema != SCHEMA
            || self.operation != OPERATION
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
        self.arguments.validate()
    }
}
pub fn execute(request: &Request) -> Response {
    let result = request.validate().and_then(|()| {
        let mut http = GoogleHttp::from_env()?;
        http.limit_time(std::time::Duration::from_millis(request.limits.timeout_ms))?;
        inspect_key(&mut http, &request.arguments, chrono::Utc::now())
    });
    match result {
        Ok(value) => Response {
            schema: SCHEMA,
            id: request.id.clone(),
            operation: OPERATION,
            result: Some(json!(value)),
            error: None,
        },
        Err(error) => Response {
            schema: SCHEMA,
            id: request.id.clone(),
            operation: OPERATION,
            result: None,
            error: Some(error.code()),
        },
    }
}
pub fn manifest() -> Value {
    json!({"schema":"zix-extension/v1", "package":"zixcel-google-admin", "version":env!("CARGO_PKG_VERSION"),
        "namespace":"google", "platform":"linux-x86_64", "protocol":SCHEMA,
        "credential_environment":["ZIX_GOOGLE_ACCESS_TOKEN"],"timeout_ms":35000,
        "commands":[{"id":OPERATION,"words":["iam","keys","inspect"],"effect":"read",
            "description":"Observe one service-account key without using or changing it.",
            "required_permissions":["iam.serviceAccounts.get","iam.serviceAccountKeys.list"],
            "arguments":[{"name":"project","required":true,"maximum":30},
                {"name":"service-account","required":true,"maximum":128},
                {"name":"key-id","required":true,"maximum":40}]}]})
}
