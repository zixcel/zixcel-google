use super::*;
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
