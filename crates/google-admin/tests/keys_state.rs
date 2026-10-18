mod support;
use serde_json::json;
use support::*;
use zixcel_google_admin::{Error, KeyState, inspect_key};
#[test]
fn successful_two_step_binding_distinguishes_enabled_disabled_and_absent() {
    let mut f = fake(account(), json!({"keys":[key()]}));
    let result = inspect_key(&mut f, &query(), now()).unwrap();
    assert_eq!(result.state, KeyState::Enabled);
    assert_eq!(result.authority, "none");
    assert_eq!(f.paths.len(), 2);
    assert!(f.paths[1].contains("/serviceAccounts/123456789012345678901/keys?"));
    assert!(f.paths[1].ends_with("/keys?keyTypes=USER_MANAGED"));
    let mut k = key();
    k["disabled"] = json!(true);
    assert_eq!(
        inspect_key(&mut fake(account(), json!({"keys":[k]})), &query(), now())
            .unwrap()
            .state,
        KeyState::Disabled
    );
    assert_eq!(
        inspect_key(&mut fake(account(), json!({})), &query(), now())
            .unwrap()
            .state,
        KeyState::NotPresent
    );
}
#[test]
fn missing_and_inaccessible_accounts_never_become_absent_keys() {
    for error in [
        Error::Unauthorized,
        Error::Forbidden,
        Error::Unavailable,
        Error::RateLimited,
        Error::Transport,
    ] {
        let mut f = Fake {
            replies: vec![Err(error)],
            paths: vec![],
        };
        assert_eq!(inspect_key(&mut f, &query(), now()).unwrap_err(), error);
        assert_eq!(f.paths.len(), 1);
        let mut f = fake(account(), json!({}));
        f.replies[1] = Err(error);
        assert_eq!(inspect_key(&mut f, &query(), now()).unwrap_err(), error);
    }
}
#[test]
fn expired_and_future_keys_do_not_appear_enabled() {
    let mut expired = key();
    expired["validBeforeTime"] = json!("2025-01-01T00:00:00Z");
    assert_eq!(
        inspect_key(
            &mut fake(account(), json!({"keys":[expired]})),
            &query(),
            now()
        )
        .unwrap()
        .state,
        KeyState::Expired
    );
    let mut future = key();
    future["validAfterTime"] = json!("2027-01-01T00:00:00Z");
    assert_eq!(
        inspect_key(
            &mut fake(account(), json!({"keys":[future]})),
            &query(),
            now()
        )
        .unwrap()
        .state,
        KeyState::NotYetValid
    );
}
