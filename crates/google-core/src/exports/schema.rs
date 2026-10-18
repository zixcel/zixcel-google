use super::*;
mod cells;
pub(super) const COLUMNS: &[&str] = &[
    "billing_account_id",
    "project",
    "service",
    "sku",
    "cost",
    "currency",
    "invoice",
    "credits",
    "usage_start_time",
    "usage_end_time",
    "cost_type",
];
pub(super) struct Projection {
    pub fields: Vec<Value>,
}
impl Projection {
    pub fn new(metadata: &Value) -> Result<Self> {
        let fields = list(
            metadata.get("schema").ok_or(Error::InvalidResponse)?,
            "fields",
        )?;
        let selected = fields
            .iter()
            .filter(|f| f["name"].as_str().is_some_and(|n| COLUMNS.contains(&n)))
            .cloned()
            .collect::<Vec<_>>();
        if selected.len() != COLUMNS.len() {
            return Err(Error::InvalidResponse);
        }
        let p = Self { fields: selected };
        for (path, kind) in [
            (&["billing_account_id"][..], "STRING"),
            (&["project", "id"][..], "STRING"),
            (&["service", "description"][..], "STRING"),
            (&["sku", "description"][..], "STRING"),
            (&["invoice", "month"][..], "STRING"),
            (&["cost"][..], "FLOAT"),
            (&["currency"][..], "STRING"),
            (&["credits", "amount"][..], "FLOAT"),
            (&["usage_start_time"][..], "TIMESTAMP"),
            (&["usage_end_time"][..], "TIMESTAMP"),
            (&["cost_type"][..], "STRING"),
        ] {
            let field = p.field(path)?;
            if field["type"] != kind {
                return Err(Error::InvalidResponse);
            }
        }
        if p.field(&["credits"])?["mode"] != "REPEATED" {
            return Err(Error::InvalidResponse);
        }
        Ok(p)
    }
    pub(super) fn field(&self, path: &[&str]) -> Result<&Value> {
        let mut fields = &self.fields[..];
        let mut result = None;
        for (i, name) in path.iter().enumerate() {
            let f = fields
                .iter()
                .find(|f| f["name"] == *name)
                .ok_or(Error::InvalidResponse)?;
            if i + 1 < path.len() {
                fields = list(f, "fields")?;
            }
            result = Some(f);
        }
        result.ok_or(Error::InvalidResponse)
    }
}
pub(super) fn decimal(value: &Value) -> Result<String> {
    let s = value.as_str().ok_or(Error::InvalidResponse)?;
    if s.len() > 128 || s.parse::<serde_json::Number>().is_err() {
        return Err(Error::InvalidResponse);
    }
    Ok(s.into())
}
