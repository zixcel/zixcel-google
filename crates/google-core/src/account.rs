use crate::{Error, KeyQuery, Result};
use serde::Deserialize;
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Account {
    pub name: String,
    pub project_id: String,
    pub unique_id: String,
    pub email: String,
    #[serde(default)]
    pub disabled: bool,
}
impl Account {
    pub fn bind(&self, query: &KeyQuery) -> Result<()> {
        if self.project_id != query.project
            || self.email != query.service_account
            || !(6..=30).contains(&self.unique_id.len())
            || !self.unique_id.bytes().all(|b| b.is_ascii_digit())
        {
            return Err(Error::ScopeMismatch);
        }
        self.check_account_name(&self.name)
    }
    fn check_account_name(&self, name: &str) -> Result<()> {
        let parts: Vec<_> = name.split('/').collect();
        if parts.len() != 4
            || parts[0] != "projects"
            || parts[2] != "serviceAccounts"
            || (parts[1] != self.project_id && parts[1] != "-")
            || (parts[3] != self.email && parts[3] != self.unique_id)
        {
            return Err(Error::ScopeMismatch);
        }
        Ok(())
    }
    pub fn key_id<'a>(&self, name: &'a str) -> Result<&'a str> {
        let (prefix, id) = name.rsplit_once("/keys/").ok_or(Error::InvalidResponse)?;
        self.check_account_name(prefix)?;
        if id.len() != 40
            || !id
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        {
            return Err(Error::InvalidResponse);
        }
        Ok(id)
    }
}
