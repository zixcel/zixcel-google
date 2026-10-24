use crate::{
    Error, Result,
    control::{Account, HttpRequest},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const IAM: &str = include_str!("../../../../api/iam.catalog.json");
const COMPUTE: &str = include_str!("../../../../api/compute.catalog.json");
const POLICY: &str = include_str!("../../../../api/operations.json");
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub schema: String,
    pub operation: String,
    pub account: Account,
    pub target: Target,
    pub request_id: Option<String>,
    pub operation_name: Option<String>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Target {
    ServiceAccount {
        unique_id: String,
    },
    Key {
        unique_id: String,
        key_id: String,
    },
    Instance {
        zone: String,
        name: String,
        expected_id: String,
    },
}
#[derive(Serialize)]
pub struct Plan {
    pub schema: &'static str,
    pub digest: String,
    pub operation: String,
    pub account: Account,
    pub request: HttpRequest,
    pub target_resource: String,
    pub expected_id: String,
    pub contract: Value,
    pub atomic_identity_precondition: bool,
}
pub(super) fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
pub fn catalog() -> Value {
    json!({"schema":super::SCHEMA,"plan_only":true,"approval_required":true,
      "allowed_origins":["https://iam.googleapis.com","https://compute.googleapis.com"],
      "operations":serde_json::from_str::<Value>(POLICY).expect("embedded policy")["operations"],
      "policy_sha256":hash(POLICY.as_bytes())})
}
fn number(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 30
        && !value.starts_with('0')
        && value.bytes().all(|b| b.is_ascii_digit())
}
fn segment(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 63
        && value.as_bytes()[0].is_ascii_lowercase()
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        && !value.ends_with('-')
}
fn uuid(value: &str) -> bool {
    value.len() == 36
        && value != "00000000-0000-0000-0000-000000000000"
        && value.bytes().enumerate().all(|(i, b)| {
            if [8, 13, 18, 23].contains(&i) {
                b == b'-'
            } else {
                b.is_ascii_hexdigit() && !b.is_ascii_uppercase()
            }
        })
}
pub fn prepare(input: Request) -> Result<Plan> {
    if input.schema != super::SCHEMA {
        return Err(Error::Protocol);
    }
    input.account.validate()?;
    // Project IDs, never wildcard projects or caller-supplied resource URIs.
    crate::query::validate_project(&input.account.account)?;
    let policy = catalog();
    let operation = policy["operations"]
        .as_array()
        .expect("embedded policy")
        .iter()
        .find(|op| op["id"] == input.operation)
        .ok_or(Error::Protocol)?;
    let write = operation["effect"] == "write";
    let project = &input.account.account;
    let (source, resource, expected, atomic) = match &input.target {
        Target::ServiceAccount { unique_id }
            if number(unique_id) && input.operation.starts_with("service-account.") =>
        {
            (
                IAM,
                format!("projects/{project}/serviceAccounts/{unique_id}"),
                unique_id.clone(),
                true,
            )
        }
        Target::Key { unique_id, key_id }
            if number(unique_id)
                && input.operation.starts_with("key.")
                && key_id.len() == 40
                && key_id
                    .bytes()
                    .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()) =>
        {
            (
                IAM,
                format!("projects/{project}/serviceAccounts/{unique_id}/keys/{key_id}"),
                key_id.clone(),
                true,
            )
        }
        Target::Instance {
            zone,
            name,
            expected_id,
        } if segment(zone)
            && segment(name)
            && number(expected_id)
            && input.operation.starts_with("instance.") =>
        {
            (
                COMPUTE,
                format!("projects/{project}/zones/{zone}/instances/{name}"),
                expected_id.clone(),
                false,
            )
        }
        _ => return Err(Error::InvalidInput),
    };
    let spec: Value = serde_json::from_str(source).expect("embedded reviewed catalog");
    let method = spec["operations"]
        .as_array()
        .expect("embedded catalog")
        .iter()
        .find(|m| m["id"] == operation["api_method"])
        .ok_or(Error::Protocol)?;
    let expected_origin = if source == IAM {
        "https://iam.googleapis.com/"
    } else {
        "https://compute.googleapis.com/"
    };
    if method["context"]["rootUrl"] != expected_origin || !method["binding"].is_null() {
        return Err(Error::Protocol);
    }
    if input.operation != "instance.operation-get" && input.operation_name.is_some() {
        return Err(Error::InvalidInput);
    }
    let action = input.operation.rsplit('.').next().ok_or(Error::Protocol)?;
    let (path, verb, body) = if source == IAM {
        if input.request_id.is_some() {
            return Err(Error::InvalidInput);
        }
        let suffix = if action == "disable" { ":disable" } else { "" };
        if method["path"] != format!("v1/{{+name}}{suffix}") {
            return Err(Error::Protocol);
        }
        (
            format!("v1/{resource}{suffix}"),
            if action == "disable" {
                "POST"
            } else if action == "delete" {
                "DELETE"
            } else {
                "GET"
            },
            (action == "disable").then(|| "{}".into()),
        )
    } else if action == "operation-get" {
        let name = input
            .operation_name
            .as_deref()
            .filter(|s| segment(s))
            .ok_or(Error::InvalidInput)?;
        if input.request_id.is_some()
            || method["path"] != "projects/{project}/zones/{zone}/operations/{operation}"
        {
            return Err(Error::InvalidInput);
        }
        let prefix = resource
            .rsplit_once("/instances/")
            .ok_or(Error::InvalidInput)?
            .0;
        (
            format!("compute/v1/{prefix}/operations/{name}"),
            "GET",
            None,
        )
    } else {
        let suffix = if action == "stop" { "/stop" } else { "" };
        if method["path"]
            != format!("projects/{{project}}/zones/{{zone}}/instances/{{instance}}{suffix}")
        {
            return Err(Error::Protocol);
        }
        if write && !input.request_id.as_deref().is_some_and(uuid)
            || !write && input.request_id.is_some()
        {
            return Err(Error::InvalidInput);
        }
        (
            format!("compute/v1/{resource}{suffix}"),
            if action == "stop" {
                "POST"
            } else if action == "delete" {
                "DELETE"
            } else {
                "GET"
            },
            None,
        )
    };
    if method["method"] != verb {
        return Err(Error::Protocol);
    }
    let query = input
        .request_id
        .as_ref()
        .map(|id| format!("?requestId={id}"))
        .unwrap_or_default();
    let mut plan = Plan {
        digest: String::new(),
        schema: super::SCHEMA,
        operation: input.operation,
        account: input.account,
        request: HttpRequest {
            method: verb,
            url: format!("{expected_origin}{path}{query}"),
            body,
        },
        target_resource: resource,
        expected_id: expected,
        atomic_identity_precondition: atomic,
        contract: json!({"source":spec["source"],"catalog_sha256":hash(source.as_bytes()),
          "api_method":operation["api_method"],"policy_sha256":hash(POLICY.as_bytes())}),
    };
    let mut value = serde_json::to_value(&plan).map_err(|_| Error::Protocol)?;
    value
        .as_object_mut()
        .ok_or(Error::Protocol)?
        .remove("digest");
    plan.digest = hash(&serde_json::to_vec(&value).map_err(|_| Error::Protocol)?);
    Ok(plan)
}
