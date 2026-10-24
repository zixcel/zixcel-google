//! Normalize one complete, stable export page through the existing byte decoder.
//! No SQL job, invoice issuance or payment assertion is involved.
use super::*;
use chrono::{NaiveDate, TimeZone, Utc};
use zixcel_cost::billing::{BillingError, BillingScope, Money, UsageObservation, UsageSnapshot};

#[derive(Clone, Debug)]
pub struct UsageSelection {
    /// Explicit invoice reporting month (YYYYMM), not the usage timestamp month.
    pub invoice_month: String,
    pub currency: String,
    pub observed_at_ms: u64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UsageError {
    Provider(Error),
    Arithmetic(BillingError),
    IncompleteExport,
    InvalidSelection,
}
impl std::fmt::Display for UsageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Provider(e) => write!(f, "{e}"),
            Self::Arithmetic(e) => write!(f, "{e}"),
            Self::IncompleteExport => f.write_str("incomplete_or_unstable_export"),
            Self::InvalidSelection => f.write_str("invalid_usage_selection"),
        }
    }
}
impl std::error::Error for UsageError {}
impl From<Error> for UsageError {
    fn from(e: Error) -> Self {
        Self::Provider(e)
    }
}
impl From<BillingError> for UsageError {
    fn from(e: BillingError) -> Self {
        Self::Arithmetic(e)
    }
}

/// Bind account, source project/dataset/table and reporting month before collection.
pub fn usage_scope(
    q: &Query,
    selection: &UsageSelection,
) -> std::result::Result<BillingScope, UsageError> {
    q.validate()?;
    if q.operation != "billing.exports.read" || q.page_token.is_some() {
        return Err(UsageError::InvalidSelection);
    }
    Money::new(0, 0, selection.currency.clone())?;
    let month = &selection.invoice_month;
    if month.len() != 6 || !month.bytes().all(|b| b.is_ascii_digit()) {
        return Err(UsageError::InvalidSelection);
    }
    let year: i32 = month[..4]
        .parse()
        .map_err(|_| UsageError::InvalidSelection)?;
    let number: u32 = month[4..]
        .parse()
        .map_err(|_| UsageError::InvalidSelection)?;
    if !(1970..=9998).contains(&year) {
        return Err(UsageError::InvalidSelection);
    }
    let start = NaiveDate::from_ymd_opt(year, number, 1).ok_or(UsageError::InvalidSelection)?;
    let end = if number == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, number + 1, 1)
    }
    .ok_or(UsageError::InvalidSelection)?;
    let millis = |date: NaiveDate| -> std::result::Result<u64, UsageError> {
        u64::try_from(
            Utc.from_utc_datetime(
                &date
                    .and_hms_opt(0, 0, 0)
                    .ok_or(UsageError::InvalidSelection)?,
            )
            .timestamp_millis(),
        )
        .map_err(|_| UsageError::InvalidSelection)
    };
    use sha2::Digest;
    let identity = serde_json::to_vec(&(&q.project, &q.dataset, &q.table))
        .map_err(|_| UsageError::InvalidSelection)?;
    let digest = sha2::Sha256::digest(identity)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    Ok(BillingScope {
        provider: "google-billing-export".into(),
        account: format!(
            "{}:{digest}",
            q.billing_account
                .as_deref()
                .ok_or(UsageError::InvalidSelection)?
        ),
        period_start_ms: millis(start)?,
        period_end_ms: millis(end)?,
    })
}

/// Read and normalize synthetic or authenticated provider bytes. Caller owns acquisition authorization.
/// Multi-page/unstable reports are refused; there is no implicit paging or paid query.
pub fn read_usage(
    http: &mut impl GoogleRead,
    q: &Query,
    selection: &UsageSelection,
) -> std::result::Result<UsageSnapshot, UsageError> {
    let scope = usage_scope(q, selection)?;
    let page = execute(http, q)?;
    if page["coverage"]["complete"] != true {
        return Err(UsageError::IncompleteExport);
    }
    let mut total = Money::new(0, 0, selection.currency.clone())?;
    for row in list(&page, "records")? {
        if text(row, "invoice_month")? != selection.invoice_month {
            continue;
        }
        let currency = text(row, "currency")?;
        total = total.checked_add(&Money::from_decimal(&text(row, "cost")?, currency.clone())?)?;
        for credit in list(row, "credits")? {
            let amount = credit.as_str().ok_or(Error::InvalidResponse)?;
            total = total.checked_add(&Money::from_decimal(amount, currency.clone())?)?;
        }
    }
    let source = page.get("source").ok_or(Error::InvalidResponse)?;
    // Etag + last-modified identify a table revision; changed bytes under it are
    // rejected by UsageMonitor rather than minted into an invented new revision.
    use sha2::Digest;
    let revision = serde_json::to_vec(&(
        text(source, "table_etag")?,
        text(source, "last_modified_time")?,
    ))
    .map_err(|_| Error::InvalidResponse)?;
    let revision = sha2::Sha256::digest(revision)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let snapshot = UsageSnapshot {
        provider: scope.provider,
        account: scope.account,
        period_start_ms: scope.period_start_ms,
        period_end_ms: scope.period_end_ms,
        observed_at_ms: selection.observed_at_ms,
        complete: true,
        observations: vec![UsageObservation {
            observation_id: "selected-month-total".into(),
            revision,
            total,
            source_digest: text(source, "raw_response_sha256")?,
        }],
    };
    snapshot.validate()?;
    Ok(snapshot)
}
