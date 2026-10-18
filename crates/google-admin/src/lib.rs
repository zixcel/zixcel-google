#![forbid(unsafe_code)]
//! Read-only administrator observations. No cloud mutation or secret-key login.
mod account;
mod error;
mod http;
pub mod inventory;
mod keys;
pub mod protocol;
mod query;
mod request;
pub use error::{Error, Result};
pub use http::GoogleHttp;
pub use keys::{KeyObservation, KeyState, inspect_key};
pub use query::KeyQuery;
pub use request::{GoogleApi, ReadRequest};

/// Requests originate from reviewed fixed-origin builders; test implementations need no credentials.
pub trait GoogleRead {
    fn read(&mut self, request: &ReadRequest) -> Result<Vec<u8>>;
}

mod http_path;
