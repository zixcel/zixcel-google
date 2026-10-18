use super::{
    Request,
    input::{fields, text},
};
use crate::{Error, Result};
use serde_json::json;
pub(super) fn permissions(input: &Request) -> Result<String> {
    fields(&input.input, &["permissions"])?;
    let permissions = input.input["permissions"]
        .as_array()
        .ok_or(Error::InvalidInput)?;
    if permissions.is_empty() || permissions.len() > 32 {
        return Err(Error::InvalidInput);
    }
    let mut names = std::collections::BTreeSet::new();
    for permission in permissions {
        let p = text(permission, 128)?;
        if p.len() < 2
            || !p.as_bytes()[0].is_ascii_alphabetic()
            || !p.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'.')
            || !names.insert(p)
        {
            return Err(Error::InvalidInput);
        }
    }
    Ok(json!({"permissions":permissions}).to_string())
}
pub(super) fn labels(input: &Request, resource: &str) -> Result<String> {
    fields(&input.input, &["labels", "etag"])?;
    let labels = input.input["labels"]
        .as_object()
        .ok_or(Error::InvalidInput)?;
    if labels.len() > 32 || !input.account.account.bytes().all(|b| b.is_ascii_digit()) {
        return Err(Error::InvalidInput);
    }
    for (key, value) in labels {
        if key.is_empty()
            || key.len() > 63
            || !key.as_bytes()[0].is_ascii_lowercase()
            || !label(key)
            || !value.as_str().is_some_and(|v| v.len() <= 63 && label(v))
        {
            return Err(Error::InvalidInput);
        }
    }
    let etag = text(&input.input["etag"], 256)?;
    Ok(json!({"name":resource,"labels":labels,"etag":etag}).to_string())
}
fn label(value: &str) -> bool {
    value
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"_-".contains(&b))
}
