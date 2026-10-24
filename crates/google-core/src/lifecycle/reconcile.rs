use super::{Plan, Request, prepare};
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Evidence {
    /// None denotes missing/timeout evidence, never an invented HTTP success.
    pub status: Option<u16>,
    pub body: Value,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Reconciliation {
    pub request: Request,
    pub plan_sha256: String,
    pub before: Evidence,
    pub mutation: Evidence,
    pub operation: Option<Evidence>,
    pub after: Evidence,
}
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum State {
    Complete,
    Pending,
    Unknown,
    Rejected,
    OperationFailed,
    TargetChanged,
}
fn body(e: &Evidence) -> Result<&Value> {
    if e.status != Some(200) || !e.body.is_object() {
        return Err(Error::InvalidResponse);
    }
    Ok(&e.body)
}
fn same_instance(v: &Value, plan: &Plan) -> bool {
    let segments: Vec<_> = plan.target_resource.split('/').collect();
    v["id"] == plan.expected_id
        && v["name"] == segments[5]
        && url_matches(
            &v["zone"],
            &format!("projects/{}/zones/{}", segments[1], segments[3]),
        )
        && url_matches(&v["selfLink"], &plan.target_resource)
}
fn url_matches(value: &Value, resource: &str) -> bool {
    value.as_str().is_some_and(|s| {
        s == format!("https://www.googleapis.com/compute/v1/{resource}")
            || s == format!("https://compute.googleapis.com/compute/v1/{resource}")
    })
}
fn same_iam(v: &Value, plan: &Plan) -> bool {
    if plan.operation.starts_with("key.") {
        v["name"] == plan.target_resource && v["keyType"] == "USER_MANAGED"
    } else {
        v["uniqueId"] == plan.expected_id && v["projectId"] == plan.account.account
    }
}
fn same(v: &Value, plan: &Plan) -> bool {
    if plan.operation.starts_with("instance.") {
        same_instance(v, plan)
    } else {
        same_iam(v, plan)
    }
}
fn operation(e: &Evidence, plan: &Plan, request_id: &str) -> Result<State> {
    let v = body(e)?;
    let segments: Vec<_> = plan.target_resource.split('/').collect();
    let name = v["name"]
        .as_str()
        .filter(|s| {
            !s.is_empty()
                && s.len() <= 63
                && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        })
        .ok_or(Error::InvalidResponse)?;
    if v["targetId"] != plan.expected_id
        || !url_matches(&v["targetLink"], &plan.target_resource)
        || !url_matches(
            &v["zone"],
            &format!("projects/{}/zones/{}", segments[1], segments[3]),
        )
        || !url_matches(
            &v["selfLink"],
            &format!(
                "projects/{}/zones/{}/operations/{name}",
                segments[1], segments[3]
            ),
        )
        || v["clientOperationId"] != request_id
        || v["operationType"]
            != plan
                .operation
                .rsplit('.')
                .next()
                .ok_or(Error::InvalidInput)?
    {
        return Err(Error::ScopeMismatch);
    }
    if !v["error"].is_null() || !v["httpErrorStatusCode"].is_null() {
        return Ok(State::OperationFailed);
    }
    match v["status"].as_str() {
        Some("DONE") => Ok(State::Complete),
        Some("PENDING" | "RUNNING") => Ok(State::Pending),
        _ => Err(Error::InvalidResponse),
    }
}
/// Evidence is supplied by a trusted executor. This pure check authenticates no receipt
/// and issues no grants; callers must bind evidence to the exact requests and times.
pub fn reconcile(input: Reconciliation) -> Result<State> {
    let plan = prepare(input.request.clone())?;
    if plan.digest != input.plan_sha256 {
        return Err(Error::ScopeMismatch);
    }
    if !["disable", "delete", "stop"].contains(
        &plan
            .operation
            .rsplit('.')
            .next()
            .ok_or(Error::InvalidInput)?,
    ) {
        return Err(Error::InvalidInput);
    }
    if !same(body(&input.before)?, &plan) {
        return Ok(State::TargetChanged);
    }
    let Some(status) = input.mutation.status else {
        return Ok(State::Unknown);
    };
    if !(200..300).contains(&status) {
        return Ok(
            if [400, 401, 403, 404, 409, 412, 422, 429].contains(&status) {
                State::Rejected
            } else {
                State::Unknown
            },
        );
    }
    if plan.operation.starts_with("instance.") {
        let id = input
            .request
            .request_id
            .as_deref()
            .ok_or(Error::InvalidInput)?;
        // Mutation acknowledgement itself must be scoped, even if a later read is supplied.
        let initial = operation(&input.mutation, &plan, id)?;
        if initial == State::OperationFailed {
            return Ok(initial);
        }
        let state = if let Some(read) = input.operation {
            if read.status.is_none() {
                return Ok(State::Unknown);
            }
            if read.body["name"] != input.mutation.body["name"] {
                return Err(Error::ScopeMismatch);
            }
            operation(&read, &plan, id)?
        } else {
            initial
        };
        if state != State::Complete {
            return Ok(state);
        }
    } else if input
        .mutation
        .body
        .as_object()
        .is_none_or(|b| !b.is_empty())
    {
        return Err(Error::InvalidResponse);
    }
    if input.after.status == Some(404) && plan.operation.ends_with(".delete") {
        return Ok(State::Complete);
    }
    if input.after.status != Some(200) {
        return Ok(State::Unknown);
    }
    let after = body(&input.after)?;
    if !same(after, &plan) {
        return Ok(State::TargetChanged);
    }
    if plan.operation.ends_with(".delete") {
        return Ok(State::Pending);
    }
    if plan.operation.ends_with(".stop") {
        return match after["status"].as_str() {
            Some("TERMINATED") => Ok(State::Complete),
            Some(
                "RUNNING" | "STOPPING" | "STAGING" | "PROVISIONING" | "SUSPENDED" | "SUSPENDING"
                | "REPAIRING",
            ) => Ok(State::Pending),
            _ => Err(Error::InvalidResponse),
        };
    }
    match after["disabled"].as_bool() {
        Some(true) => Ok(State::Complete),
        Some(false) => Ok(State::Pending),
        None => Err(Error::InvalidResponse),
    }
}
