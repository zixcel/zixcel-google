use super::{
    collect,
    model::{Coverage, Record},
    query::InventoryQuery,
    scope::ProjectScope,
};
use crate::{Error, GoogleRead, Result};
use serde::Serialize;
use std::collections::BTreeMap;
#[derive(Serialize)]
pub struct Section {
    operation: String,
    error: Option<&'static str>,
    count: Option<usize>,
    coverage: Option<Coverage>,
    states: BTreeMap<String, usize>,
    sample: Vec<Record>,
}
#[derive(Serialize)]
pub struct Overview {
    schema: &'static str,
    scope: Option<String>,
    observed_at: String,
    selected_scans_complete: bool,
    sections: Vec<Section>,
    cost_conclusion: &'static str,
    visibility: &'static str,
}
pub fn inspect(transport: &mut impl GoogleRead, query: &InventoryQuery) -> Result<Overview> {
    if !(1..=8).contains(&query.max_pages) || query.page_token.is_some() {
        return Err(Error::InvalidInput);
    }
    let project = query.operation == "overview.project";
    if !project && query.operation != "overview.accessible" {
        return Err(Error::InvalidInput);
    }
    let scope = if project {
        Some(ProjectScope::resolve(
            transport,
            query.project.as_deref().ok_or(Error::InvalidInput)?,
        )?)
    } else {
        None
    };
    let ops: &[&str] = if project {
        &[
            "billing.project.inspect",
            "services.list",
            "assets.search",
            "compute.instances.list",
            "compute.disks.list",
            "compute.addresses.list",
            "compute.snapshots.list",
            "compute.commitments.list",
            "sql.instances.list",
            "storage.buckets.list",
            "run.services.list",
            "iam.accounts.list",
        ]
    } else {
        &["projects.list", "billing.accounts.list"]
    };
    let mut sections = Vec::new();
    let mut complete = true;
    for op in ops {
        let mut q = query.clone();
        q.operation = (*op).into();
        let section = match collect::collect(transport, &q, scope.as_ref()) {
            Ok(collection) => {
                complete &= collection.coverage.complete;
                let count = collection.records.len();
                let mut states = BTreeMap::new();
                for record in &collection.records {
                    if let Some(state) = &record.state {
                        *states.entry(state.clone()).or_default() += 1;
                    }
                }
                Section {
                    operation: (*op).into(),
                    error: None,
                    count: Some(count),
                    coverage: Some(collection.coverage),
                    states,
                    sample: collection.records.into_iter().take(3).collect(),
                }
            }
            Err(error) => {
                complete = false;
                Section {
                    operation: (*op).into(),
                    error: Some(error.code()),
                    count: None,
                    coverage: None,
                    states: BTreeMap::new(),
                    sample: vec![],
                }
            }
        };
        sections.push(section);
    }
    Ok(Overview {
        schema: "zix-google-overview/v1",
        scope: scope.map(|s| s.id),
        observed_at: chrono::Utc::now().to_rfc3339(),
        selected_scans_complete: complete,
        sections,
        cost_conclusion: "unknown; account_deletion_api_enablement_and_resource_presence_do_not_prove_zero_cost_or_invoice_amount",
        visibility: "selected_APIs_and_caller_visible_resources_only; samples_not_exhaustive",
    })
}
