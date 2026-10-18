use super::*;
impl Projection {
    pub fn cell<'a>(&self, row: &'a Value, path: &[&str]) -> Result<Option<&'a Value>> {
        let mut fields = &self.fields[..];
        let mut cells = list(row, "f")?;
        let mut value = None;
        for (i, name) in path.iter().enumerate() {
            if cells.len() != fields.len() {
                return Err(Error::InvalidResponse);
            }
            let index = fields
                .iter()
                .position(|f| f["name"] == *name)
                .ok_or(Error::InvalidResponse)?;
            let v = cells[index].get("v").ok_or(Error::InvalidResponse)?;
            if v.is_null() {
                return Ok(None);
            }
            value = Some(v);
            if i + 1 < path.len() {
                fields = list(&fields[index], "fields")?;
                cells = list(v, "f")?;
            }
        }
        Ok(value)
    }
    pub fn credit_amounts(&self, row: &Value) -> Result<Vec<String>> {
        let values = self
            .cell(row, &["credits"])?
            .and_then(Value::as_array)
            .ok_or(Error::InvalidResponse)?;
        if values.len() > 1000 {
            return Err(Error::InvalidResponse);
        }
        let fields = list(self.field(&["credits"])?, "fields")?;
        let index = fields
            .iter()
            .position(|f| f["name"] == "amount")
            .ok_or(Error::InvalidResponse)?;
        values
            .iter()
            .map(|entry| {
                let cells = list(entry.get("v").ok_or(Error::InvalidResponse)?, "f")?;
                if cells.len() != fields.len() {
                    return Err(Error::InvalidResponse);
                }
                decimal(cells[index].get("v").ok_or(Error::InvalidResponse)?)
            })
            .collect()
    }
}
