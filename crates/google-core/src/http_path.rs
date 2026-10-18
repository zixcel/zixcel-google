use crate::{Error, Result};
pub(crate) fn validate(path: &str) -> Result<()> {
    let (resource, query) = path.split_once('?').unwrap_or((path, ""));
    let parts: Vec<_> = resource.split('/').collect();
    if !(parts.len() == 6 || parts.len() == 7)
        || !parts[0].is_empty()
        || parts[1] != "v1"
        || parts[2] != "projects"
        || parts[4] != "serviceAccounts"
        || (parts.len() == 6 && !query.is_empty())
        || (parts.len() == 7 && (parts[6] != "keys" || query != "keyTypes=USER_MANAGED"))
    {
        return Err(Error::InvalidInput);
    }
    if (6..=30).contains(&parts[5].len()) && parts[5].bytes().all(|b| b.is_ascii_digit()) {
        crate::query::validate_project(parts[3])?;
    } else {
        crate::query::validate_account(parts[3], parts[5])?;
    }
    Ok(())
}
