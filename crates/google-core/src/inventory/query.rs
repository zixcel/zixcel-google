use crate::{Error, Result};
use std::collections::BTreeMap;
#[derive(Clone, Debug)]
pub struct InventoryQuery {
    pub(crate) operation: String,
    pub(crate) project: Option<String>,
    pub(crate) billing_account: Option<String>,
    pub(crate) page_token: Option<String>,
    pub(crate) max_pages: u8,
    pub(crate) quota_project: Option<String>,
}
pub fn project_identifier(value: &str) -> Result<()> {
    if (1..=20).contains(&value.len()) && value.bytes().all(|b| b.is_ascii_digit()) {
        return Ok(());
    }
    crate::query::validate_project(value)
}
pub fn billing_identifier(value: &str) -> Result<()> {
    let parts = value.split('-').collect::<Vec<_>>();
    if parts.len() != 3
        || parts.iter().any(|s| {
            s.len() != 6
                || !s
                    .bytes()
                    .all(|b| b.is_ascii_digit() || b.is_ascii_uppercase())
        })
    {
        return Err(Error::InvalidInput);
    }
    Ok(())
}
impl InventoryQuery {
    pub fn operation(&self) -> &str {
        &self.operation
    }
    pub fn quota_project(&self) -> Option<&str> {
        self.quota_project.as_deref()
    }
    pub fn parse(operation: &str, args: &BTreeMap<String, String>) -> Result<Self> {
        let spec = super::catalog::specification(operation);
        let overview = operation == "overview.project" || operation == "overview.accessible";
        if spec.is_none() && !overview {
            return Err(Error::Protocol);
        }
        let project = spec.is_some_and(|s| s.project) || operation == "overview.project";
        let billing = spec.is_some_and(|s| s.billing);
        let paged = spec.is_some_and(|s| s.paged) || overview;
        let allowed = super::manifest::argument_names(project, billing, paged);
        if args.keys().any(|k| !allowed.contains(&k.as_str())) {
            return Err(Error::InvalidInput);
        }
        if project && !args.contains_key("project")
            || billing && !args.contains_key("billing-account")
        {
            return Err(Error::InvalidInput);
        }
        let query = Self {
            operation: operation.into(),
            project: args.get("project").cloned(),
            billing_account: args.get("billing-account").cloned(),
            page_token: args.get("page-token").cloned(),
            max_pages: args
                .get("max-pages")
                .map(|s| s.parse::<u8>())
                .transpose()
                .map_err(|_| Error::InvalidInput)?
                .unwrap_or(2),
            quota_project: args.get("quota-project").cloned(),
        };
        if !(1..=8).contains(&query.max_pages) {
            return Err(Error::InvalidInput);
        }
        for project in [query.project.as_deref(), query.quota_project.as_deref()]
            .into_iter()
            .flatten()
        {
            project_identifier(project)?;
        }
        if let Some(account) = &query.billing_account {
            billing_identifier(account)?;
        }
        if query
            .page_token
            .as_ref()
            .is_some_and(|s| s.is_empty() || s.len() > 2048 || s.chars().any(char::is_control))
        {
            return Err(Error::InvalidInput);
        }
        if overview && query.page_token.is_some() {
            return Err(Error::InvalidInput);
        }
        Ok(query)
    }
}
