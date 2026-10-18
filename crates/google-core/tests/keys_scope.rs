mod support;
use serde_json::json;
use support::*;
use zixcel_google_core::{Error, KeyQuery, inspect_key};
#[test]
fn account_binding_rejects_other_project_email_identifier_and_name() {
    for (field, value) in [
        ("projectId", "other-project"),
        ("email", "other@sample-project.iam.gserviceaccount.com"),
        ("uniqueId", "not-numeric"),
        (
            "name",
            "projects/other-project/serviceAccounts/123456789012345678901",
        ),
    ] {
        let mut a = account();
        a[field] = json!(value);
        let mut f = fake(a, json!({}));
        assert_eq!(
            inspect_key(&mut f, &query(), now()).unwrap_err(),
            Error::ScopeMismatch
        );
        assert_eq!(f.paths.len(), 1);
    }
}
#[test]
fn key_names_are_bound_to_the_observed_account() {
    for name in [
        "projects/other-project/serviceAccounts/123456789012345678901/keys/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "projects/sample-project/serviceAccounts/999999999999999999999/keys/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    ] {
        let mut k = key();
        k["name"] = json!(name);
        assert_eq!(
            inspect_key(&mut fake(account(), json!({"keys":[k]})), &query(), now()).unwrap_err(),
            Error::ScopeMismatch
        );
    }
}
#[test]
fn invalid_query_never_reaches_transport() {
    for (field, value) in [
        ("project", "../sample"),
        (
            "service-account",
            "reader-account@other-project.iam.gserviceaccount.com",
        ),
        ("key-id", "A"),
        ("key-id", "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/"),
    ] {
        let mut v = serde_json::to_value(query()).unwrap();
        v[field] = json!(value);
        let q: KeyQuery = serde_json::from_value(v).unwrap();
        let mut f = fake(account(), json!({}));
        assert_eq!(
            inspect_key(&mut f, &q, now()).unwrap_err(),
            Error::InvalidInput
        );
        assert!(f.paths.is_empty());
    }
}
