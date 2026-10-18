use super::*;
#[test]
fn disabled_bigquery_is_reported_without_querying_or_enabling_it() {
    let mut f=Fake{urls:vec![],replies:vec![
        json!({"name":"projects/123456789012","projectId":"sample-project"}),
        json!({"name":"projects/123456789012/services/bigquery.googleapis.com","state":"DISABLED"})
    ].into()};
    let args = BTreeMap::from([("project".into(), "sample-project".into())]);
    let q = Query::parse("billing.exports.discover", &args).unwrap();
    assert_eq!(
        exports::execute(&mut f, &q).unwrap_err(),
        Error::ServiceDisabled
    );
    assert_eq!(f.urls.len(), 2);
    assert!(f.urls[1].starts_with("https://serviceusage.googleapis.com/"));
}
#[test]
fn unavailable_service_metadata_does_not_block_an_authorized_bigquery_read() {
    let mut f = Fake {
        urls: vec![],
        replies: vec![json!({"unavailable":"synthetic"}), json!({"datasets":[]})].into(),
    };
    let args = BTreeMap::from([("project".into(), "sample-project".into())]);
    let q = Query::parse("billing.exports.discover", &args).unwrap();
    let r = exports::execute(&mut f, &q).unwrap();
    assert_eq!(r["coverage"]["complete"], true);
    assert_eq!(f.urls.len(), 2);
    assert!(f.urls[1].starts_with("https://bigquery.googleapis.com/"));
}
#[test]
fn arbitrary_tables_scope_mismatch_and_malformed_schema_are_rejected_before_data_reads() {
    let mut a = args();
    a[2] = ("table", "private_unrelated");
    let (v, f) = invoke("billing.exports.read", &a, vec![]);
    assert_eq!(v.unwrap_err(), Error::InvalidInput);
    assert!(f.urls.is_empty());
    let mut m = metadata();
    m["tableReference"]["projectId"] = json!("other-project");
    let (v, f) = invoke("billing.exports.read", &args(), vec![m]);
    assert_eq!(v.unwrap_err(), Error::ScopeMismatch);
    assert_eq!(f.urls.len(), 1);
    let mut m = metadata();
    m["schema"]["fields"][4]["type"] = json!("STRING");
    let (v, f) = invoke("billing.exports.read", &args(), vec![m]);
    assert_eq!(v.unwrap_err(), Error::InvalidResponse);
    assert_eq!(f.urls.len(), 1);
}
#[test]
fn wrong_account_and_non_decimal_amounts_are_never_reported_as_costs() {
    for (i, value, expected) in [
        (0, json!("FFFFFF-FFFFFF-FFFFFF"), Error::ScopeMismatch),
        (4, json!("NaN"), Error::InvalidResponse),
    ] {
        let m = metadata();
        let mut r = row();
        r["f"][i]["v"] = value;
        let (v, _) = invoke(
            "billing.exports.read",
            &args(),
            vec![m, json!({"totalRows":"1","rows":[r]})],
        );
        assert_eq!(v.unwrap_err(), expected);
    }
}
