use crate::{Error, IamRead, Result};
use reqwest::{
    blocking::Client,
    header::{AUTHORIZATION, HeaderValue},
};
use std::time::{Duration, Instant};
const ORIGIN: &str = "https://iam.googleapis.com";
const MAXIMUM: usize = 1024 * 1024;
pub struct GoogleHttp {
    client: Client,
    authorization: HeaderValue,
    deadline: Instant,
    remaining: u8,
}
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
            deadline: Instant::now() + Duration::from_secs(30),
            remaining: 2,
        })
    }
    pub fn from_env() -> Result<Self> {
        let token =
            std::env::var("ZIX_GOOGLE_ACCESS_TOKEN").map_err(|_| Error::CredentialMissing)?;
        Self::new(&token)
    }
    pub(crate) fn limit_time(&mut self, limit: Duration) -> Result<()> {
        if limit.is_zero() || limit > Duration::from_secs(40) {
            return Err(Error::Protocol);
        }
        self.deadline = self.deadline.min(Instant::now() + limit);
        Ok(())
    }
}
impl IamRead for GoogleHttp {
    fn get(&mut self, path: &str) -> Result<Vec<u8>> {
        if path.len() > 1024 {
            return Err(Error::InvalidInput);
        }
        crate::http_path::validate(path)?;
        self.get_url(&format!("{ORIGIN}{path}"))
    }
}
impl GoogleHttp {
    // Only the validated fixed-origin public get() calls this in production.
    fn get_url(&mut self, url: &str) -> Result<Vec<u8>> {
        if self.remaining == 0 {
            return Err(Error::Deadline);
        }
        self.remaining -= 1;
        let timeout = self
            .deadline
            .checked_duration_since(Instant::now())
            .ok_or(Error::Deadline)?;
        let response = self
            .client
            .get(url)
            .header(AUTHORIZATION, self.authorization.clone())
            .timeout(timeout.min(Duration::from_secs(15)))
            .send()
            .map_err(|_| Error::Transport)?;
        match response.status().as_u16() {
            200 => {}
            401 => return Err(Error::Unauthorized),
            403 => return Err(Error::Forbidden),
            404 => return Err(Error::Unavailable),
            429 => return Err(Error::RateLimited),
            _ => return Err(Error::HttpStatus),
        }
        if response
            .content_length()
            .is_some_and(|n| n > MAXIMUM as u64)
        {
            return Err(Error::ResponseTooLarge);
        }
        zixcel_bounded_io::read_bounded(response, MAXIMUM as u64).map_err(|e| {
            if e.get_ref()
                .is_some_and(|v| v.is::<zixcel_bounded_io::InputLimit>())
            {
                Error::ResponseTooLarge
            } else {
                Error::Transport
            }
        })
    }
}
#[cfg(test)]
#[path = "http_tests.rs"]
mod tests;
