#![forbid(unsafe_code)]
//! Read-only, scope-bound IAM observations. No key creation, revocation or secret-key login.
mod account;
mod error;
mod http;
mod keys;
pub mod protocol;
mod query;
pub use error::{Error, Result};
pub use http::GoogleHttp;
pub use keys::{KeyObservation, KeyState, inspect_key};
pub use query::KeyQuery;

/// The adapter controls a fixed IAM origin. Tests can supply credential-free responses.
pub trait IamRead {
    fn get(&mut self, path: &str) -> Result<Vec<u8>>;
}

mod http_path;
