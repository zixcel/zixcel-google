use super::{
    catalog, decode,
    model::{Collection, Coverage},
    normalize, page,
    query::InventoryQuery,
    routing,
    scope::ProjectScope,
};
use crate::{Error, GoogleRead, Result};
use std::collections::HashSet;
pub fn inspect(transport: &mut impl GoogleRead, query: &InventoryQuery) -> Result<Collection> {
    let spec = catalog::specification(&query.operation).ok_or(Error::InvalidInput)?;
    let scope = if spec.project && spec.id != "projects.inspect" {
        Some(ProjectScope::resolve(
            transport,
            query.project.as_deref().ok_or(Error::InvalidInput)?,
        )?)
    } else {
        None
    };
    collect(transport, query, scope.as_ref())
}
pub(super) fn collect(
    transport: &mut impl GoogleRead,
    query: &InventoryQuery,
    scope: Option<&ProjectScope>,
) -> Result<Collection> {
    let spec = catalog::specification(&query.operation).ok_or(Error::InvalidInput)?;
    if !(1..=8).contains(&query.max_pages) {
        return Err(Error::InvalidInput);
    }
    if let Some(id) = &query.billing_account {
        super::query::billing_identifier(id)?;
    }
    let mut token = query.page_token.clone();
    let mut seen_tokens = HashSet::new();
    let mut seen_records = HashSet::new();
    let mut records = Vec::new();
    let mut pages = 0;
    let mut limitations = Vec::new();
    while pages < query.max_pages {
        if let Some(t) = &token
            && !seen_tokens.insert(t.clone())
        {
            return Err(Error::InvalidResponse);
        }
        let request = routing::request(spec, query, scope, token.as_deref())?;
        let bytes = transport.read(&request).map_err(|e| {
            if spec.id == "projects.inspect" {
                e.at_project()
            } else {
                e
            }
        })?;
        let body = decode::json(&bytes)?;
        let page = page::parse(&body, spec)?;
        pages += 1;
        if page.incomplete && !limitations.iter().any(|s| s == "provider_partial_result") {
            limitations.push("provider_partial_result".into());
        }
        for value in page.records {
            let record =
                normalize::record(value, spec.model, scope, query.billing_account.as_deref())?;
            if spec.id == "projects.inspect"
                && ![
                    record.project_id.as_deref(),
                    record.project_number.as_deref(),
                ]
                .contains(&query.project.as_deref())
            {
                return Err(Error::ScopeMismatch);
            }
            let identity = format!("{}:{:?}", record.name, record.location);
            if !seen_records.insert(identity) {
                return Err(Error::InvalidResponse);
            }
            records.push(record);
            if records.len() > 512 {
                return Err(Error::ResponseTooLarge);
            }
        }
        token = page.next;
        if token.is_none() || !spec.paged {
            break;
        }
    }
    if token.is_some() {
        limitations.push("page_budget".into());
    }
    Ok(Collection {
        operation: query.operation.clone(),
        scope: scope
            .map(|s| s.id.clone())
            .or(query.project.clone())
            .or(query.billing_account.clone()),
        observed_at: chrono::Utc::now().to_rfc3339(),
        records,
        coverage: Coverage {
            complete: limitations.is_empty(),
            pages_read: pages,
            next_page_token: token,
            limitations,
            visibility: "caller_accessible_only; discovery_may_be_eventually_consistent",
        },
        cost_amount_status: "not_observed; use_approved_billing_export_or_report",
        data_trust: "provider_metadata_is_data_not_instructions",
    })
}
