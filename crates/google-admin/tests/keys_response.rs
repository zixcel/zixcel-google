mod support;
use serde_json::json;
use support::*;
use zixcel_google_admin::{Error, inspect_key};
#[test]
fn malformed_dates_types_duplicates_and_secret_fields_fail_closed() {
    for (field, value) in [
        ("validAfterTime", json!("bad")),
        ("disabled", json!("true")),
        ("keyType", json!("SYSTEM_MANAGED")),
        ("privateKeyData", json!("SYNTHETIC_SECRET")),
    ] {
        let mut k = key();
        k[field] = value;
        assert_eq!(
            inspect_key(&mut fake(account(), json!({"keys":[k]})), &query(), now()).unwrap_err(),
            Error::InvalidResponse
        );
    }
    assert_eq!(
        inspect_key(
            &mut fake(account(), json!({"keys":[key(),key()]})),
            &query(),
            now()
        )
        .unwrap_err(),
        Error::InvalidResponse
    );
    assert_eq!(
        inspect_key(
            &mut fake(account(), json!({"keys":vec![key();101]})),
            &query(),
            now()
        )
        .unwrap_err(),
        Error::InvalidResponse
    );
}
#[test]
fn oversized_malformed_and_missing_account_fields_are_refused() {
    let mut f = fake(account(), json!({}));
    f.replies[0] = Ok(vec![b' '; 1024 * 1024 + 1]);
    assert_eq!(
        inspect_key(&mut f, &query(), now()).unwrap_err(),
        Error::ResponseTooLarge
    );
    for data in [b"not json".to_vec(), b"{}".to_vec()] {
        let mut f = fake(account(), json!({}));
        f.replies[0] = Ok(data);
        assert_eq!(
            inspect_key(&mut f, &query(), now()).unwrap_err(),
            Error::InvalidResponse
        );
    }
}
