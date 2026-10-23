//! Real provider byte decoder -> provider normalizer -> independent shared monitor.
//! No live HTTP, credentials, account data or billing jobs.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use zixcel_cost::billing::{BillingError, Money, MonitorLimits, UsageMonitor};
use zixcel_google_core::{
    Error, GoogleRead, ReadRequest,
    exports::{Query, UsageError, UsageSelection, read_usage, usage_scope},
};
mod export_fixtures;
use export_fixtures::{args, metadata, row};
struct Bytes {
    replies: Vec<Vec<u8>>,
    paths: Vec<String>,
}
impl GoogleRead for Bytes {
    fn read(&mut self, request: &ReadRequest) -> zixcel_google_core::Result<Vec<u8>> {
        self.paths.push(request.path_and_query());
        Ok(self.replies.remove(0))
    }
}
fn query() -> Query {
    Query::parse(
        "billing.exports.read",
        &args()
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect::<BTreeMap<_, _>>(),
    )
    .unwrap()
}
fn selection() -> UsageSelection {
    UsageSelection {
        invoice_month: "202609".into(),
        currency: "USD".into(),
        observed_at_ms: 1_790_812_801_000,
    }
}
fn valid_row() -> Value {
    let mut r = row();
    r["f"][7]["v"][0]["v"]["f"][0]["v"] = json!("-0.000000001");
    r
}
fn page() -> Value {
    json!({"totalRows":"1", "rows":[valid_row()]})
}
fn wire(data: Value, etag: &str) -> Bytes {
    let mut m = metadata();
    m["etag"] = json!(etag);
    Bytes {
        replies: vec![m.clone(), data, m]
            .into_iter()
            .map(|v| serde_json::to_vec(&v).unwrap())
            .collect(),
        paths: vec![],
    }
}
fn monitor() -> UsageMonitor {
    let s = selection();
    UsageMonitor::new(
        usage_scope(&query(), &s).unwrap(),
        Money::new(100, 0, "USD".into()).unwrap(),
        MonitorLimits {
            expires_at_ms: s.observed_at_ms + 1000,
            maximum_observations: 20,
            maximum_revision_entries: 20,
            maximum_age_ms: 1000,
        },
        s.observed_at_ms,
    )
    .unwrap()
}
#[test]
fn vendor_bytes_keep_exact_credits_and_source_digest_through_normalizer_and_monitor() {
    let data = page();
    let expected = Sha256::digest(serde_json::to_vec(&data).unwrap())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let mut transport = wire(data, "r1");
    let snapshot = read_usage(&mut transport, &query(), &selection()).unwrap();
    assert_eq!(snapshot.observations[0].source_digest, expected);
    assert_eq!(snapshot.period_start_ms, 1_788_220_800_000);
    assert_eq!(snapshot.period_end_ms, 1_790_812_800_000);
    let mut m = monitor();
    let first = m.observe(&snapshot, selection().observed_at_ms).unwrap();
    assert_eq!(first.total.coefficient(), 9_007_199_254_740_993_000_000_000);
    assert_eq!(first.total.scale(), 9);
    let again = read_usage(&mut wire(page(), "r1"), &query(), &selection()).unwrap();
    assert!(
        !m.observe(&again, selection().observed_at_ms)
            .unwrap()
            .changed
    );
    assert_eq!(first.total, again.observations[0].total);
    assert_eq!(transport.paths.len(), 3);
    assert!(transport.paths.iter().all(|p| !p.contains("/jobs")));
    assert!(
        !serde_json::to_string(&snapshot)
            .unwrap()
            .contains("SYNTHETIC_PRIVATE_LABEL")
    );
}
#[test]
fn changed_same_revision_and_superseded_revision_never_replace_last_complete_result() {
    let mut m = monitor();
    let s = selection();
    let original = read_usage(&mut wire(page(), "r1"), &query(), &s).unwrap();
    m.observe(&original, s.observed_at_ms).unwrap();
    let mut altered = page();
    altered["rows"][0]["f"][4]["v"] = json!("4.25");
    let changed = read_usage(&mut wire(altered.clone(), "r1"), &query(), &s).unwrap();
    assert_eq!(
        m.observe(&changed, s.observed_at_ms),
        Err(BillingError::InconsistentRevision)
    );
    assert_eq!(m.observations(), 1);
    assert!(!m.observe(&original, s.observed_at_ms).unwrap().changed);
    let corrected = read_usage(&mut wire(altered, "r2"), &query(), &s).unwrap();
    assert!(m.observe(&corrected, s.observed_at_ms).unwrap().changed);
    assert_eq!(
        m.observe(&original, s.observed_at_ms),
        Err(BillingError::RevisionRollback)
    );
    let mut older = corrected.clone();
    older.observed_at_ms -= 1;
    assert_eq!(
        m.observe(&older, s.observed_at_ms),
        Err(BillingError::StaleObservation)
    );
    let mut digest_substitution = page();
    digest_substitution["unprojected"] = json!("synthetic");
    let changed = read_usage(&mut wire(digest_substitution, "r2"), &query(), &s).unwrap();
    assert_eq!(
        m.observe(&changed, s.observed_at_ms),
        Err(BillingError::InconsistentRevision)
    );
}
#[test]
fn incomplete_duplicate_continuation_and_unstable_pages_are_not_assembled_into_a_snapshot() {
    for data in [
        json!({"totalRows":"2","rows":[valid_row()]}),
        json!({"totalRows":"1","pageToken":"same-page-again","rows":[valid_row()]}),
    ] {
        assert_eq!(
            read_usage(&mut wire(data, "r1"), &query(), &selection()),
            Err(UsageError::IncompleteExport)
        );
    }
    let mut unstable = wire(page(), "r1");
    let mut after = metadata();
    after["etag"] = json!("r2");
    unstable.replies[2] = serde_json::to_vec(&after).unwrap();
    assert_eq!(
        read_usage(&mut unstable, &query(), &selection()),
        Err(UsageError::IncompleteExport)
    );
    let mut args = args()
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect::<BTreeMap<_, _>>();
    args.insert("page-token".into(), "duplicate".into());
    let q = Query::parse("billing.exports.read", &args).unwrap();
    let mut http = wire(page(), "r1");
    assert_eq!(
        read_usage(&mut http, &q, &selection()),
        Err(UsageError::InvalidSelection)
    );
    assert!(http.paths.is_empty());
}
#[test]
fn period_currency_precision_and_source_scope_are_explicit_not_inferred() {
    let mut data = page();
    data["rows"][0]["f"][6]["v"]["f"][0]["v"] = json!("202608");
    let snapshot = read_usage(&mut wire(data, "r1"), &query(), &selection()).unwrap();
    assert_eq!(snapshot.observations[0].total.coefficient(), 0);
    let mut s = selection();
    s.invoice_month = "202608".into();
    let prior_period = read_usage(&mut wire(page(), "r1"), &query(), &s).unwrap();
    assert_eq!(
        monitor().observe(&prior_period, selection().observed_at_ms),
        Err(BillingError::InvalidObservation)
    );
    let mut mixed = page();
    let mut yen = valid_row();
    yen["f"][5]["v"] = json!("JPY");
    mixed["rows"].as_array_mut().unwrap().push(yen);
    mixed["totalRows"] = json!("2");
    assert_eq!(
        read_usage(&mut wire(mixed, "r1"), &query(), &selection()),
        Err(UsageError::Arithmetic(BillingError::CurrencyMismatch))
    );
    assert_eq!(
        read_usage(
            &mut wire(json!({"totalRows":"1","rows":[row()]}), "r1"),
            &query(),
            &selection()
        ),
        Err(UsageError::Arithmetic(BillingError::InvalidAmount))
    );
    let mut wrong = wire(page(), "r1");
    let mut m = metadata();
    m["tableReference"]["projectId"] = json!("other-project");
    wrong.replies[0] = serde_json::to_vec(&m).unwrap();
    assert_eq!(
        read_usage(&mut wrong, &query(), &selection()),
        Err(UsageError::Provider(Error::ScopeMismatch))
    );
    let mut s = selection();
    s.invoice_month = "202613".into();
    let mut http = wire(page(), "r1");
    assert!(read_usage(&mut http, &query(), &s).is_err());
    assert!(http.paths.is_empty());
}
#[test]
fn real_decoder_retains_authorization_errors_without_calling_them_zero_cost() {
    struct Denied;
    impl GoogleRead for Denied {
        fn read(&mut self, _: &ReadRequest) -> zixcel_google_core::Result<Vec<u8>> {
            Err(Error::Forbidden)
        }
    }
    assert_eq!(
        read_usage(&mut Denied, &query(), &selection()),
        Err(UsageError::Provider(Error::Forbidden))
    );
}
