mod control_cases;
use serde_json::{Value, json};
use zixcel_google_core::{Error, control};
fn request(operation: &str, input: Value) -> control::Request {
    serde_json::from_value(json!({"schema":control::SCHEMA,"operation":operation,
        "account":{"provider":"google-cloud","subject":"synthetic-owner","account":"1234567890"},"input":input})).unwrap()
}
fn plan(operation: &str, input: Value) -> Value {
    serde_json::to_value(control::prepare(request(operation, input)).unwrap()).unwrap()
}
#[test]
fn reviewed_operations_have_fixed_origins_and_separate_cost_effects() {
    let samples = [
        (
            "accounts.search",
            json!({"pageToken":null}),
            "GET",
            "cloudresourcemanager.googleapis.com",
        ),
        (
            "services.list",
            json!({"pageToken":null}),
            "GET",
            "serviceusage.googleapis.com",
        ),
        (
            "account.read",
            json!({}),
            "GET",
            "cloudresourcemanager.googleapis.com",
        ),
        (
            "billing.project-read",
            json!({}),
            "GET",
            "cloudbilling.googleapis.com",
        ),
        (
            "permissions.test",
            json!({"permissions":["resourcemanager.projects.get"]}),
            "POST",
            "cloudresourcemanager.googleapis.com",
        ),
        (
            "project.set-labels",
            json!({"labels":{"env":"test"},"etag":"observed-etag"}),
            "PATCH",
            "cloudresourcemanager.googleapis.com",
        ),
    ];
    for (id, input, method, host) in samples {
        let value = plan(id, input);
        assert_eq!(value["request"]["method"], method);
        assert!(
            value["request"]["url"]
                .as_str()
                .unwrap()
                .starts_with(&format!("https://{host}/"))
        );
        assert_eq!(value["account"]["account"], "1234567890");
    }
    let catalog = control::catalog();
    assert_eq!(catalog["plan_only"], true);
    assert_eq!(catalog["approval_required"], true);
    assert_eq!(catalog["operations"].as_array().unwrap().len(), 6);
    for op in catalog["operations"].as_array().unwrap() {
        assert_eq!(op["request_charge"], "not_assessed");
        assert_eq!(op["creates_billable_resources"], false);
    }
}
#[test]
fn permission_test_rejects_duplicates_unbounded_and_injected_fields() {
    for input in [
        json!({"permissions":[]}),
        json!({"permissions":["a.b","a.b"]}),
        json!({"permissions":vec!["a.b";33]}),
        json!({"permissions":["a/b"]}),
        json!({"permissions":["a.b"],"url":"https://example.invalid"}),
    ] {
        assert!(control::prepare(request("permissions.test", input)).is_err());
    }
}
