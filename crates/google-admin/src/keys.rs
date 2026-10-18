use crate::{Error, IamRead, KeyQuery, Result, account::Account};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyState {
    Enabled,
    Disabled,
    Expired,
    NotYetValid,
    NotPresent,
}
#[derive(Debug, Serialize)]
pub struct KeyObservation {
    pub project: String,
    pub service_account: String,
    pub key_id: String,
    pub state: KeyState,
    pub service_account_disabled: bool,
    pub observed_at: String,
    pub authority: &'static str,
    pub coverage: &'static str,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Key {
    name: String,
    key_type: String,
    #[serde(default)]
    disabled: bool,
    valid_after_time: String,
    valid_before_time: String,
    #[serde(default)]
    private_key_data: Option<serde_json::Value>,
}
#[derive(Deserialize)]
struct KeyList {
    #[serde(default)]
    keys: Vec<Key>,
}
/// Exactly one service account and its user-managed keys; no key is authenticated or modified.
pub fn inspect_key(
    transport: &mut impl IamRead,
    query: &KeyQuery,
    now: DateTime<Utc>,
) -> Result<KeyObservation> {
    let path = query.account_path()?;
    let account: Account = decode(&transport.get(&path).map_err(|error| {
        if error == Error::Unavailable {
            Error::AccountUnavailable
        } else {
            error
        }
    })?)?;
    account.bind(query)?;
    // The immutable account ID prevents an email being deleted/recreated between the two reads.
    let keys_path = format!(
        "/v1/projects/{}/serviceAccounts/{}/keys?keyTypes=USER_MANAGED",
        query.project, account.unique_id
    );
    let list: KeyList = decode(&transport.get(&keys_path).map_err(|error| {
        if error == Error::Unavailable {
            Error::KeyListUnavailable
        } else {
            error
        }
    })?)?;
    if list.keys.len() > 100 {
        return Err(Error::InvalidResponse);
    }
    let mut seen = HashSet::new();
    let mut state = KeyState::NotPresent;
    for key in list.keys {
        let id = account.key_id(&key.name)?;
        if !seen.insert(id.to_owned())
            || key.key_type != "USER_MANAGED"
            || key.private_key_data.is_some()
        {
            return Err(Error::InvalidResponse);
        }
        let start = DateTime::parse_from_rfc3339(&key.valid_after_time)
            .map_err(|_| Error::InvalidResponse)?;
        let end = DateTime::parse_from_rfc3339(&key.valid_before_time)
            .map_err(|_| Error::InvalidResponse)?;
        if start >= end {
            return Err(Error::InvalidResponse);
        }
        if id == query.key_id {
            state = if key.disabled {
                KeyState::Disabled
            } else if now >= end {
                KeyState::Expired
            } else if now < start {
                KeyState::NotYetValid
            } else {
                KeyState::Enabled
            };
        }
    }
    Ok(KeyObservation {
        project: query.project.clone(),
        service_account: query.service_account.clone(),
        key_id: query.key_id.clone(),
        state,
        service_account_disabled: account.disabled,
        observed_at: now.to_rfc3339(),
        authority: "none",
        coverage: "requested_account_user_managed_keys_only",
    })
}
fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    if bytes.len() > 1024 * 1024 {
        return Err(Error::ResponseTooLarge);
    }
    serde_json::from_slice(bytes).map_err(|_| Error::InvalidResponse)
}
