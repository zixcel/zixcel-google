mod support;
use serde_json::json;
use support::*;
use zixcel_google_core::{Error, inspect_key};

#[test]
fn resource_failures_identify_the_phase_without_claiming_key_absence() {
    let mut account_failure = Fake {
        replies: vec![Err(Error::Unavailable)],
        paths: vec![],
    };
    let error = inspect_key(&mut account_failure, &query(), now()).unwrap_err();
    assert_eq!(error, Error::AccountUnavailable);
    assert_eq!(error.code(), "account_unavailable");
    assert_eq!(account_failure.paths.len(), 1);

    let mut list_failure = fake(account(), json!({"keys":[key()]}));
    list_failure.replies[1] = Err(Error::Unavailable);
    let error = inspect_key(&mut list_failure, &query(), now()).unwrap_err();
    assert_eq!(error, Error::KeyListUnavailable);
    assert_eq!(error.code(), "key_list_unavailable");
    assert_eq!(list_failure.paths.len(), 2);
}
