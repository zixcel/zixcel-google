use crate::{Error, Result};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GoogleApi {
    Iam,
    Resource,
    Billing,
    Usage,
    Asset,
    Compute,
    Sql,
    Storage,
    Run,
    BigQuery,
}
impl GoogleApi {
    pub fn origin(self) -> &'static str {
        match self {
            Self::Iam => "https://iam.googleapis.com",
            Self::Resource => "https://cloudresourcemanager.googleapis.com",
            Self::Billing => "https://cloudbilling.googleapis.com",
            Self::Usage => "https://serviceusage.googleapis.com",
            Self::Asset => "https://cloudasset.googleapis.com",
            Self::Compute => "https://compute.googleapis.com",
            Self::Sql => "https://sqladmin.googleapis.com",
            Self::Storage => "https://storage.googleapis.com",
            Self::Run => "https://run.googleapis.com",
            Self::BigQuery => "https://bigquery.googleapis.com",
        }
    }
}
#[derive(Clone, Debug)]
pub struct ReadRequest {
    api: GoogleApi,
    path: String,
    query: Vec<(String, String)>,
}
impl ReadRequest {
    pub fn iam(path: &str) -> Result<Self> {
        crate::http_path::validate(path)?;
        let (path, query) = path.split_once('?').unwrap_or((path, ""));
        let query = if query.is_empty() {
            vec![]
        } else {
            vec![("keyTypes".into(), "USER_MANAGED".into())]
        };
        Self::new(GoogleApi::Iam, path.to_owned(), query)
    }
    pub(crate) fn new(api: GoogleApi, path: String, query: Vec<(String, String)>) -> Result<Self> {
        if !path.starts_with('/')
            || path.contains(['?', '#', '\\'])
            || path.split('/').any(|s| s == ".." || s == ".")
            || path.len() > 1024
            || !path.is_ascii()
            || path.bytes().any(|b| b.is_ascii_control())
            || query.len() > 8
            || query
                .iter()
                .any(|(k, v)| k.len() > 64 || v.len() > 2048 || v.chars().any(char::is_control))
        {
            return Err(Error::InvalidInput);
        }
        Ok(Self { api, path, query })
    }
    pub fn url(&self) -> String {
        let mut url = reqwest::Url::parse(self.api.origin()).expect("static origin");
        url.set_path(&self.path);
        if !self.query.is_empty() {
            url.query_pairs_mut()
                .extend_pairs(self.query.iter().map(|(k, v)| (k, v)));
        }
        url.to_string()
    }
    pub fn path_and_query(&self) -> String {
        self.url()[self.api.origin().len()..].to_owned()
    }
}
