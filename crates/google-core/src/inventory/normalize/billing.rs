use super::super::{
    decode::{flag, text},
    model::Record,
    query,
};
use crate::{Error, Result};
use serde_json::Value;
pub(super) fn account(value: &Value, r: &mut Record) -> Result<()> {
    query::billing_identifier(
        r.name
            .strip_prefix("billingAccounts/")
            .ok_or(Error::InvalidResponse)?,
    )?;
    r.open = flag(value, "open")?;
    r.charge_hint = "account_open_is_not_spend_evidence";
    Ok(())
}
pub(super) fn project(value: &Value, r: &mut Record, billing: Option<&str>) -> Result<()> {
    let project = r.project_id.as_deref().ok_or(Error::InvalidResponse)?;
    crate::query::validate_project(project).map_err(|_| Error::InvalidResponse)?;
    if r.name != format!("projects/{project}/billingInfo") {
        return Err(Error::ScopeMismatch);
    }
    r.billing_account = text(value, "billingAccountName")?.filter(|s| !s.is_empty());
    r.billing_enabled = flag(value, "billingEnabled")?;
    if let Some(name) = &r.billing_account {
        let id = name
            .strip_prefix("billingAccounts/")
            .ok_or(Error::InvalidResponse)?;
        query::billing_identifier(id)?;
        if billing.is_some_and(|expected| expected != id) {
            return Err(Error::ScopeMismatch);
        }
    }
    if r.billing_enabled == Some(true) && r.billing_account.is_none() {
        return Err(Error::InvalidResponse);
    }
    r.charge_hint = "billing_linkage_is_not_a_cost_statement";
    Ok(())
}
