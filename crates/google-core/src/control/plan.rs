use super::{
    Account, Request, SCHEMA,
    catalog::OPERATIONS,
    input::{fields, text},
};
use crate::{Error, GoogleApi, ReadRequest, Result};
use serde::Serialize;
#[derive(Serialize)]
pub struct Prepared {
    pub schema: &'static str,
    pub operation: String,
    pub account: Account,
    pub request: HttpRequest,
}
#[derive(Serialize)]
pub struct HttpRequest {
    pub method: &'static str,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
}
pub fn prepare(input: Request) -> Result<Prepared> {
    if input.schema != SCHEMA || !OPERATIONS.iter().any(|op| op.id == input.operation) {
        return Err(Error::Protocol);
    }
    input.account.validate()?;
    let resource = format!("projects/{}", input.account.account);
    let (api, path, query, method, body) = match input.operation.as_str() {
        "accounts.search" | "services.list" => {
            fields(&input.input, &["pageToken"])?;
            let mut query = vec![("pageSize".into(), "50".into())];
            if !input.input["pageToken"].is_null() {
                query.push((
                    "pageToken".into(),
                    text(&input.input["pageToken"], 2048)?.into(),
                ));
            }
            let (api, path) = if input.operation == "services.list" {
                if !input.account.account.bytes().all(|b| b.is_ascii_digit()) {
                    return Err(Error::InvalidInput);
                }
                query.push(("filter".into(), "state:ENABLED".into()));
                (GoogleApi::Usage, format!("/v1/{resource}/services"))
            } else {
                (GoogleApi::Resource, "/v3/projects:search".into())
            };
            (api, path, query, "GET", None)
        }
        "account.read" | "billing.project-read" => {
            fields(&input.input, &[])?;
            let (api, path) = if input.operation == "account.read" {
                (GoogleApi::Resource, format!("/v3/{resource}"))
            } else {
                (GoogleApi::Billing, format!("/v1/{resource}/billingInfo"))
            };
            (api, path, vec![], "GET", None)
        }
        "permissions.test" => (
            GoogleApi::Resource,
            format!("/v3/{resource}:testIamPermissions"),
            vec![],
            "POST",
            Some(super::builders::permissions(&input)?),
        ),
        "project.set-labels" => (
            GoogleApi::Resource,
            format!("/v3/{resource}"),
            vec![("updateMask".into(), "labels".into())],
            "PATCH",
            Some(super::builders::labels(&input, &resource)?),
        ),
        _ => return Err(Error::Protocol),
    };
    let request = HttpRequest {
        method,
        url: ReadRequest::new(api, path, query)?.url(),
        body,
    };
    Ok(Prepared {
        schema: SCHEMA,
        operation: input.operation,
        account: input.account,
        request,
    })
}
