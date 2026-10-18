use crate::{Error, Result};
use serde_json::Value;
pub fn json(bytes: &[u8]) -> Result<Value> {
    if bytes.len() > 1024 * 1024 {
        return Err(Error::ResponseTooLarge);
    }
    let value: Value = serde_json::from_slice(bytes).map_err(|_| Error::InvalidResponse)?;
    if !value.is_object() || value.get("error").is_some() {
        return Err(Error::InvalidResponse);
    }
    Ok(value)
}
pub fn text(value: &Value, key: &str) -> Result<Option<String>> {
    match value.get(key) {
        None => Ok(None),
        Some(Value::String(s))
            if s.len() <= 2048 && s.is_ascii() && !s.chars().any(char::is_control) =>
        {
            Ok(Some(s.clone()))
        }
        _ => Err(Error::InvalidResponse),
    }
}
pub fn required(value: &Value, key: &str) -> Result<String> {
    text(value, key)?
        .filter(|s| !s.is_empty())
        .ok_or(Error::InvalidResponse)
}
pub fn flag(value: &Value, key: &str) -> Result<Option<bool>> {
    match value.get(key) {
        None => Ok(None),
        Some(Value::Bool(b)) => Ok(Some(*b)),
        _ => Err(Error::InvalidResponse),
    }
}
pub fn quantity(value: &Value, key: &str) -> Result<Option<String>> {
    let result = match value.get(key) {
        None => return Ok(None),
        Some(Value::String(s)) => s.clone(),
        Some(Value::Number(n)) => n
            .as_u64()
            .map(|n| n.to_string())
            .ok_or(Error::InvalidResponse)?,
        _ => return Err(Error::InvalidResponse),
    };
    if result.is_empty() || result.len() > 40 || !result.bytes().all(|b| b.is_ascii_digit()) {
        return Err(Error::InvalidResponse);
    }
    Ok(Some(result))
}
pub fn array<'a>(value: &'a Value, key: &str) -> Result<&'a [Value]> {
    match value.get(key) {
        None => Ok(&[]),
        Some(Value::Array(items)) if items.len() <= 1000 => Ok(items),
        _ => Err(Error::InvalidResponse),
    }
}
