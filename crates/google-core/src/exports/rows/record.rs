use super::*;
pub(super) fn decode(p: &schema::Projection, row: &Value, account: &str) -> Result<Value> {
    let string = |path: &[&str]| -> Result<Option<String>> {
        p.cell(row, path)?
            .map(|v| {
                let s = v.as_str().ok_or(Error::InvalidResponse)?;
                if s.len() > 2048 || s.chars().any(char::is_control) {
                    return Err(Error::InvalidResponse);
                }
                Ok(s.to_owned())
            })
            .transpose()
    };
    if string(&["billing_account_id"])?.as_deref() != Some(account) {
        return Err(Error::ScopeMismatch);
    }
    let currency = string(&["currency"])?.ok_or(Error::InvalidResponse)?;
    if currency.len() != 3 || !currency.bytes().all(|b| b.is_ascii_uppercase()) {
        return Err(Error::InvalidResponse);
    }
    let month = string(&["invoice", "month"])?.ok_or(Error::InvalidResponse)?;
    if month.len() != 6
        || !month.bytes().all(|b| b.is_ascii_digit())
        || !("01"..="12").contains(&&month[4..])
    {
        return Err(Error::InvalidResponse);
    }
    Ok(
        json!({"billing_account":account,"project_id":string(&["project","id"])?,
        "service":string(&["service","description"])?,"sku":string(&["sku","description"])?,
        "invoice_month":month,"currency":currency,"cost":schema::decimal(p.cell(row,&["cost"])?.ok_or(Error::InvalidResponse)?)?,
        "credits":p.credit_amounts(row)?,"cost_type":string(&["cost_type"])?,
        "usage_start_time":string(&["usage_start_time"])?,"usage_end_time":string(&["usage_end_time"])?}),
    )
}
