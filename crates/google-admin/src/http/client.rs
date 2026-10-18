use super::*;
impl GoogleHttp {
    pub fn new(token: &str) -> Result<Self> {
        if token.is_empty()
            || token.len() > 16384
            || !token
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"._~+/-=".contains(&b))
        {
            return Err(Error::CredentialInvalid);
        }
        let mut authorization = HeaderValue::from_str(&format!("Bearer {token}"))
            .map_err(|_| Error::CredentialInvalid)?;
        authorization.set_sensitive(true);
        let client = Client::builder()
            .https_only(true)
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .retry(reqwest::retry::never())
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(15))
            .build()
            .map_err(|_| Error::Transport)?;
        Ok(Self {
            client,
            authorization,
            quota_project: None,
            deadline: Instant::now() + Duration::from_secs(30),
            remaining: 32,
        })
    }
    pub fn from_env() -> Result<Self> {
        let token =
            std::env::var("ZIX_GOOGLE_ACCESS_TOKEN").map_err(|_| Error::CredentialMissing)?;
        Self::new(&token)
    }
    pub fn set_quota_project(&mut self, project: &str) -> Result<()> {
        crate::inventory::query::project_identifier(project)?;
        self.quota_project = Some(HeaderValue::from_str(project).map_err(|_| Error::InvalidInput)?);
        Ok(())
    }
    pub(crate) fn limit_time(&mut self, limit: Duration) -> Result<()> {
        if limit.is_zero() || limit > Duration::from_secs(40) {
            return Err(Error::Protocol);
        }
        self.deadline = self.deadline.min(Instant::now() + limit);
        Ok(())
    }
}
