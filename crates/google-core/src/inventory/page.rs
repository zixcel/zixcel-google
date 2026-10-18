use super::{catalog::Spec, decode};
use crate::{Error, Result};
use serde_json::Value;
pub struct Page<'a> {
    pub records: Vec<&'a Value>,
    pub next: Option<String>,
    pub incomplete: bool,
}
pub fn parse(value: &Value, spec: Spec) -> Result<Page<'_>> {
    let next = decode::text(value, "nextPageToken")?.filter(|s| !s.is_empty());
    let mut incomplete = !decode::array(value, "unreachable")?.is_empty()
        || !decode::array(value, "warnings")?.is_empty();
    if value.get("warning").is_some() {
        incomplete = true;
    }
    let records = if !spec.paged {
        vec![value]
    } else if spec.aggregate {
        let mut records = Vec::new();
        if let Some(items) = value.get("items") {
            let items = items.as_object().ok_or(Error::InvalidResponse)?;
            if items.len() > 1000 {
                return Err(Error::InvalidResponse);
            }
            for item in items.values() {
                if !item.is_object() {
                    return Err(Error::InvalidResponse);
                }
                if let Some(warning) = item.get("warning")
                    && decode::text(warning, "code")?.as_deref() != Some("NO_RESULTS_ON_PAGE")
                {
                    incomplete = true;
                }
                records.extend(decode::array(item, spec.array)?);
            }
        }
        records
    } else {
        decode::array(value, spec.array)?.iter().collect()
    };
    if records.len() > 1000 {
        return Err(Error::ResponseTooLarge);
    }
    Ok(Page {
        records,
        next,
        incomplete,
    })
}
