use super::*;
pub(super) fn discover(http: &mut impl GoogleRead, q: &Query, base: &str) -> Result<Value> {
    if matches!(service::enabled(http, &q.project), Ok(false)) {
        return Err(Error::ServiceDisabled);
    }
    let (path, array, reference) = match &q.dataset {
        Some(dataset) => (
            format!("{base}/datasets/{dataset}/tables"),
            "tables",
            "tableReference",
        ),
        None => (format!("{base}/datasets"), "datasets", "datasetReference"),
    };
    let mut args = vec![("maxResults".into(), "50".into())];
    if let Some(t) = &q.page_token {
        args.push(("pageToken".into(), t.clone()));
    }
    let value = get(http, &path, args)?;
    let mut records = Vec::new();
    for item in list(&value, array)? {
        let r = item.get(reference).ok_or(Error::InvalidResponse)?;
        let project = text(r, "projectId")?;
        let dataset = text(r, "datasetId")?;
        query::identifier(&dataset).map_err(|_| Error::InvalidResponse)?;
        if project != q.project || q.dataset.as_ref().is_some_and(|d| d != &dataset) {
            return Err(Error::ScopeMismatch);
        }
        let mut record = json!({"project":project,"dataset":dataset});
        if q.dataset.is_some() {
            let table = text(r, "tableId")?;
            if let Some(account) = query::export_account(&table) {
                record["table"] = json!(table);
                record["billing_account"] = json!(account);
                records.push(record);
            }
        } else {
            records.push(record);
        }
    }
    let next = value
        .get("nextPageToken")
        .map(|_| text(&value, "nextPageToken"))
        .transpose()?
        .filter(|s| !s.is_empty());
    Ok(
        json!({"schema":"zix-google-billing-discovery/v1","records":records,
        "coverage":{"complete":next.is_none(),"next_page_token":next,"visibility":"caller_accessible_metadata_only"},
        "cost_amount_status":"not_observed; empty_discovery_is_not_proof_of_no_export_or_charges"}),
    )
}
