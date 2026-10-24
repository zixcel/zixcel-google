#![forbid(unsafe_code)]
//! Google observations and typed operation plans; credentials stay with the trusted caller.
mod account;
pub mod control;
mod error;
pub mod exports;
mod http;
pub mod inventory;
mod keys;
pub mod lifecycle;
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
