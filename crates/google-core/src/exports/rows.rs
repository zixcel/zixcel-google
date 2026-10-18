use super::*;
mod record;
pub(super) fn read(http: &mut impl GoogleRead, q: &Query, base: &str) -> Result<Value> {
    let dataset = q.dataset.as_deref().ok_or(Error::InvalidInput)?;
    let table = q.table.as_deref().ok_or(Error::InvalidInput)?;
    let account = q.billing_account.as_deref().ok_or(Error::InvalidInput)?;
    let path = format!("{base}/datasets/{dataset}/tables/{table}");
    let metadata = get(http, &path, vec![])?;
    binding(&metadata, q)?;
    if text(&metadata, "lastModifiedTime")?.parse::<u64>().is_err() {
        return Err(Error::InvalidResponse);
    }
    let projection = schema::Projection::new(&metadata)?;
    let mut args = vec![
        ("maxResults".into(), "50".into()),
        (
            "selectedFields".into(),
            projection
                .fields
                .iter()
                .map(|v| text(v, "name"))
                .collect::<Result<Vec<_>>>()?
                .join(","),
        ),
    ];
    if let Some(token) = &q.page_token {
        args.push(("pageToken".into(), token.clone()));
    }
    let (data, source_digest) = get_with_digest(http, &format!("{path}/data"), args)?;
    let mut records = Vec::new();
    for row in list(&data, "rows")? {
        records.push(record::decode(&projection, row, account)?);
    }
    let after = get(http, &path, vec![])?;
    binding(&after, q)?;
    let stable = text(&metadata, "etag")? == text(&after, "etag")?
        && metadata["lastModifiedTime"] == after["lastModifiedTime"];
    let next = data
        .get("pageToken")
        .map(|_| text(&data, "pageToken"))
        .transpose()?
        .filter(|s| !s.is_empty());
    let total = text(&data, "totalRows")?;
    if total.parse::<u64>().is_err() {
        return Err(Error::InvalidResponse);
    }
    Ok(
        json!({"schema":"zix-google-billing-export-page/v1","source":{"project":q.project,"dataset":dataset,"table":table,
        "billing_account":account,"table_etag":text(&after,"etag")?,"last_modified_time":after.get("lastModifiedTime"),
        "raw_response_sha256":source_digest},
        "records":records,"coverage":{"complete":q.page_token.is_none()&&next.is_none()&&stable&&total==records.len().to_string(),
        "next_page_token":next,"table_stable_during_page_read":stable,"total_rows":total},
        "amount_semantics":"exported_decimal_strings_not_invoice_finality; credits_separate; no_currency_conversion",
        "limitations":["existing_export_only","later_adjustments_possible","multi_page_snapshot_not_guaranteed","not_an_invoice_confirmation"],
        "query_job_created":false}),
    )
}
fn binding(metadata: &Value, q: &Query) -> Result<()> {
    let r = metadata
        .get("tableReference")
        .ok_or(Error::InvalidResponse)?;
    if text(r, "projectId")? != q.project
        || Some(text(r, "datasetId")?) != q.dataset
        || Some(text(r, "tableId")?) != q.table
    {
        return Err(Error::ScopeMismatch);
    }
    if metadata["type"] != "TABLE" || metadata.get("externalDataConfiguration").is_some() {
        return Err(Error::InvalidResponse);
    }
    Ok(())
}
