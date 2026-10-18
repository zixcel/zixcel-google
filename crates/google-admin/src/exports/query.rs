use super::*;
#[derive(Clone, Debug)]
pub struct Query {
    pub(crate) operation: String,
    pub(crate) project: String,
    pub(crate) dataset: Option<String>,
    pub(crate) table: Option<String>,
    pub(crate) billing_account: Option<String>,
    pub(crate) page_token: Option<String>,
    pub(crate) quota_project: Option<String>,
}
pub fn identifier(value: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > 1024
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
    {
        return Err(Error::InvalidInput);
    }
    Ok(())
}
pub fn export_account(table: &str) -> Option<String> {
    let suffix = table
        .strip_prefix("gcp_billing_export_resource_v1_")
        .or_else(|| table.strip_prefix("gcp_billing_export_v1_"))?;
    let account = suffix.replace('_', "-");
    crate::inventory::query::billing_identifier(&account).ok()?;
    Some(account)
}
impl Query {
    pub fn parse(operation: &str, args: &BTreeMap<String, String>) -> Result<Self> {
        if !["billing.exports.discover", "billing.exports.read"].contains(&operation) {
            return Err(Error::Protocol);
        }
        let manifest = commands()
            .into_iter()
            .find(|v| v["id"] == operation)
            .ok_or(Error::Protocol)?;
        let allowed = manifest["arguments"].as_array().ok_or(Error::Protocol)?;
        if args
            .keys()
            .any(|k| !allowed.iter().any(|v| v["name"] == k.as_str()))
        {
            return Err(Error::InvalidInput);
        }
        let q = Self {
            operation: operation.into(),
            project: args.get("project").ok_or(Error::InvalidInput)?.clone(),
            dataset: args.get("dataset").cloned(),
            table: args.get("table").cloned(),
            billing_account: args.get("billing-account").cloned(),
            page_token: args.get("page-token").cloned(),
            quota_project: args.get("quota-project").cloned(),
        };
        q.validate()?;
        Ok(q)
    }
    pub(super) fn validate(&self) -> Result<()> {
        crate::inventory::query::project_identifier(&self.project)?;
        if let Some(p) = &self.quota_project {
            crate::inventory::query::project_identifier(p)?;
        }
        if let Some(d) = &self.dataset {
            identifier(d)?;
        }
        if let Some(t) = &self.page_token
            && (t.is_empty() || t.len() > 2048 || t.chars().any(char::is_control))
        {
            return Err(Error::InvalidInput);
        }
        if self.operation == "billing.exports.read" {
            let table = self.table.as_deref().ok_or(Error::InvalidInput)?;
            let account = self.billing_account.as_deref().ok_or(Error::InvalidInput)?;
            if self.dataset.is_none() || export_account(table).as_deref() != Some(account) {
                return Err(Error::InvalidInput);
            }
        }
        Ok(())
    }
}
