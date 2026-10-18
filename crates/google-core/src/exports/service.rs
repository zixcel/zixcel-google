use super::*;
/// Best-effort preflight: lack of these extra metadata permissions must not block BigQuery reads.
pub(super) fn enabled(http: &mut impl GoogleRead, project: &str) -> Result<bool> {
    let scope = crate::inventory::scope::ProjectScope::resolve(http, project)?;
    let name = format!("projects/{}/services/bigquery.googleapis.com", scope.number);
    let request = ReadRequest::new(GoogleApi::Usage, format!("/v1/{name}"), vec![])?;
    let bytes = http.read(&request)?;
    if bytes.len() > 1024 * 1024 {
        return Err(Error::ResponseTooLarge);
    }
    let value: Value = serde_json::from_slice(&bytes).map_err(|_| Error::InvalidResponse)?;
    if text(&value, "name")? != name {
        return Err(Error::ScopeMismatch);
    }
    match value["state"].as_str() {
        Some("ENABLED") => Ok(true),
        Some("DISABLED") => Ok(false),
        _ => Err(Error::InvalidResponse),
    }
}
