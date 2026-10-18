use super::*;
#[test]
fn label_update_requires_observed_etag_and_numeric_project_name() {
    let value = plan(
        "project.set-labels",
        json!({"labels":{},"etag":"observed-etag"}),
    );
    let body: Value = serde_json::from_str(value["request"]["body"].as_str().unwrap()).unwrap();
    assert_eq!(
        body,
        json!({"name":"projects/1234567890","etag":"observed-etag","labels":{}})
    );
    for input in [
        json!({"labels":{}}),
        json!({"labels":{},"etag":""}),
        json!({"labels":{"Bad":"value"},"etag":"x"}),
        json!({"labels":{},"etag":"x","token":"private"}),
    ] {
        assert!(control::prepare(request("project.set-labels", input)).is_err());
    }
    let mut request = request("project.set-labels", json!({"labels":{},"etag":"x"}));
    request.account.account = "sample-project".into();
    assert!(control::prepare(request).is_err());
}
#[test]
fn account_scope_and_pagination_cannot_change_the_destination() {
    let value = plan(
        "accounts.search",
        json!({"pageToken":"next&filter=private"}),
    );
    assert!(
        value["request"]["url"]
            .as_str()
            .unwrap()
            .contains("pageToken=next%26filter%3Dprivate")
    );
    for account in [
        "../other",
        "x?key=private",
        "projects/123",
        "sample-project/other",
    ] {
        let mut r = request("account.read", json!({}));
        r.account.account = account.into();
        assert!(control::prepare(r).is_err());
    }
    for input in [
        json!({}),
        json!({"pageToken":""}),
        json!({"pageToken":"x".repeat(2049)}),
    ] {
        assert!(control::prepare(request("accounts.search", input)).is_err());
    }
    let mut r = request("services.list", json!({"pageToken":null}));
    r.account.account = "sample-project".into();
    assert!(control::prepare(r).is_err());
    let mut r = request("account.read", json!({}));
    r.account.provider = "other".into();
    assert!(control::prepare(r).is_err());
    assert!(matches!(
        control::prepare(request("arbitrary.delete", json!({}))),
        Err(Error::Protocol)
    ));
}
