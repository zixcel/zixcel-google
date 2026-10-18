use super::{decode, query::project_identifier};
use crate::{Error, GoogleApi, GoogleRead, ReadRequest, Result};
use serde_json::Value;
#[derive(Clone, Debug)]
pub struct ProjectScope {
    pub id: String,
    pub number: String,
}
impl ProjectScope {
    pub fn resolve(transport: &mut impl GoogleRead, project: &str) -> Result<Self> {
        project_identifier(project)?;
        let request = ReadRequest::new(
            GoogleApi::Resource,
            format!("/v3/projects/{project}"),
            vec![],
        )?;
        let value = decode::json(&transport.read(&request).map_err(Error::at_project)?)?;
        let id = decode::required(&value, "projectId")?;
        crate::query::validate_project(&id).map_err(|_| Error::InvalidResponse)?;
        let name = decode::required(&value, "name")?;
        let number = name
            .strip_prefix("projects/")
            .ok_or(Error::ScopeMismatch)?
            .to_owned();
        if !(1..=20).contains(&number.len())
            || !number.bytes().all(|b| b.is_ascii_digit())
            || project != id && project != number
        {
            return Err(Error::ScopeMismatch);
        }
        Ok(Self { id, number })
    }
    fn matches(&self, value: &str) -> bool {
        value == self.id || value == self.number
    }
    pub fn bind(&self, value: &Value) -> Result<()> {
        for key in ["projectId", "projectNumber", "project"] {
            if let Some(s) = decode::text(value, key)? {
                let s = s.strip_prefix("projects/").unwrap_or(&s);
                if !self.matches(s) {
                    return Err(Error::ScopeMismatch);
                }
            }
        }
        for key in ["name", "selfLink"] {
            if let Some(s) = decode::text(value, key)? {
                let pieces = s.split('/').collect::<Vec<_>>();
                for pair in pieces.windows(2) {
                    if pair[0] == "projects" && !self.matches(pair[1]) {
                        return Err(Error::ScopeMismatch);
                    }
                }
            }
        }
        Ok(())
    }
}
