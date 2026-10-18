use crate::{Error, Result};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct KeyQuery {
    pub project: String,
    #[serde(rename = "service-account")]
    pub service_account: String,
    #[serde(rename = "key-id")]
    pub key_id: String,
}
fn segment(value: &str, min: usize, max: usize) -> bool {
    (min..=max).contains(&value.len())
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        && !value.starts_with('-')
        && !value.ends_with('-')
}
impl KeyQuery {
    pub fn validate(&self) -> Result<()> {
        validate_account(&self.project, &self.service_account)?;
        if self.key_id.len() != 40
            || !self
                .key_id
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        {
            return Err(Error::InvalidInput);
        }
        Ok(())
    }
    pub fn account_path(&self) -> Result<String> {
        self.validate()?;
        Ok(format!(
            "/v1/projects/{}/serviceAccounts/{}",
            self.project, self.service_account
        ))
    }
}
pub(crate) fn validate_account(project: &str, email: &str) -> Result<()> {
    validate_project(project)?;
    let (local, domain) = email.split_once('@').ok_or(Error::InvalidInput)?;
    if !segment(local, 6, 30) || domain != format!("{project}.iam.gserviceaccount.com") {
        return Err(Error::InvalidInput);
    }
    Ok(())
}
pub(crate) fn validate_project(project: &str) -> Result<()> {
    if !segment(project, 6, 30) || !project.starts_with(|c: char| c.is_ascii_lowercase()) {
        return Err(Error::InvalidInput);
    }
    Ok(())
}
