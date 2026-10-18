use crate::{Error, GoogleRead, ReadRequest, Result};
use reqwest::{
    blocking::Client,
    header::{AUTHORIZATION, HeaderValue},
};
use std::time::{Duration, Instant};
const MAXIMUM: usize = 1024 * 1024;
pub struct GoogleHttp {
    client: Client,
    authorization: HeaderValue,
    quota_project: Option<HeaderValue>,
    deadline: Instant,
    remaining: u8,
}
mod client;
mod status;
impl GoogleRead for GoogleHttp {
    fn read(&mut self, request: &ReadRequest) -> Result<Vec<u8>> {
        self.get_url(&request.url())
    }
}
impl GoogleHttp {
    // Only fixed-origin request builders call this in production.
    fn get_url(&mut self, url: &str) -> Result<Vec<u8>> {
        if self.remaining == 0 {
            return Err(Error::Deadline);
        }
        self.remaining -= 1;
        let timeout = self
            .deadline
            .checked_duration_since(Instant::now())
            .ok_or(Error::Deadline)?;
        let mut request = self
            .client
            .get(url)
            .header(AUTHORIZATION, self.authorization.clone())
            .timeout(timeout.min(Duration::from_secs(15)));
        if let Some(project) = &self.quota_project {
            request = request.header("x-goog-user-project", project.clone());
        }
        let response = request.send().map_err(|_| Error::Transport)?;
        let status = response.status().as_u16();
        if response
            .content_length()
            .is_some_and(|n| n > MAXIMUM as u64)
        {
            return Err(Error::ResponseTooLarge);
        }
        let bytes = zixcel_bounded_io::read_bounded(response, MAXIMUM as u64).map_err(|e| {
            if e.get_ref()
                .is_some_and(|v| v.is::<zixcel_bounded_io::InputLimit>())
            {
                Error::ResponseTooLarge
            } else {
                Error::Transport
            }
        })?;
        status::check(status, &bytes)?;
        Ok(bytes)
    }
}
#[cfg(test)]
#[path = "http_tests.rs"]
mod tests;
