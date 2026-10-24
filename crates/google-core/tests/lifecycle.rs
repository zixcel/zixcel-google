use serde_json::{Value, json};
use zixcel_google_core::{
    Error,
    lifecycle::{self, State},
};
fn request(op: &str) -> Value {
    let target = if op.starts_with("instance.") {
        json!({"kind":"instance","zone":"us-central1-a","name":"sample-vm","expected_id":"7363510519170619800"})
    } else if op.starts_with("key.") {
        json!({"kind":"key","unique_id":"123456789012345678901","key_id":"a".repeat(40)})
    } else {
        json!({"kind":"service_account","unique_id":"123456789012345678901"})
    };
    json!({"schema":lifecycle::SCHEMA,"operation":op,"account":{"provider":"google-cloud","subject":"synthetic-owner","account":"sample-project"},
      "target":target,"request_id":if op.starts_with("instance.") && op != "instance.get" && op != "instance.operation-get" {Some("12345678-1234-1234-1234-123456789abc")} else {None}})
}
fn prepare(v: Value) -> Result<lifecycle::Plan, Error> {
    lifecycle::prepare(serde_json::from_value(v).unwrap())
}
fn instance() -> Value {
    json!({"id":"7363510519170619800","name":"sample-vm","status":"RUNNING", "zone":"https://www.googleapis.com/compute/v1/projects/sample-project/zones/us-central1-a","selfLink":"https://www.googleapis.com/compute/v1/projects/sample-project/zones/us-central1-a/instances/sample-vm"})
}
fn evidence(v: Value) -> Value {
    json!({"status":200,"body":v})
}
fn case(op: &str) -> Value {
    let plan = prepare(request(op)).unwrap();
    let before = if op.starts_with("instance.") {
        instance()
    } else if op.starts_with("key.") {
        json!({"name":plan.target_resource,"keyType":"USER_MANAGED","disabled":false})
    } else {
        json!({"uniqueId":plan.expected_id,"projectId":"sample-project","disabled":false})
    };
    let mut after = before.clone();
    if op.ends_with("stop") {
        after["status"] = json!("TERMINATED");
    } else {
        after["disabled"] = json!(true);
    }
    let mutation = if op.starts_with("instance.") {
        json!({"name":"operation-test","status":"DONE","targetId":plan.expected_id,"targetLink":before["selfLink"],"zone":before["zone"],"selfLink":"https://www.googleapis.com/compute/v1/projects/sample-project/zones/us-central1-a/operations/operation-test","clientOperationId":"12345678-1234-1234-1234-123456789abc","operationType":op.rsplit('.').next().unwrap()})
    } else {
        json!({})
    };
    json!({"request":request(op),"plan_sha256":plan.digest,"before":evidence(before),"mutation":evidence(mutation),"operation":null,"after":if op.ends_with("delete") {json!({"status":404,"body":null})} else {evidence(after)}})
}
fn reconcile(v: Value) -> Result<State, Error> {
    lifecycle::reconcile(serde_json::from_value(v).unwrap())
}
#[test]
fn all_reviewed_lifecycle_routes_are_explicit_and_import_bound() {
    for id in [
        "service-account.get",
        "service-account.disable",
        "service-account.delete",
        "key.get",
        "key.disable",
        "key.delete",
        "instance.get",
        "instance.stop",
        "instance.delete",
        "instance.operation-get",
    ] {
        let mut r = request(id);
        if id.ends_with("operation-get") {
            r["operation_name"] = json!("operation-test");
        }
        let p = prepare(r).unwrap();
        assert!(
            p.request.url.starts_with("https://iam.googleapis.com/")
                || p.request
                    .url
                    .starts_with("https://compute.googleapis.com/compute/v1/")
        );
        assert!(p.contract["catalog_sha256"].as_str().unwrap().len() == 64);
        assert_eq!(p.atomic_identity_precondition, !id.starts_with("instance."));
    }
}
#[test]
fn scoped_before_and_poststate_are_required_for_each_mutation() {
    for op in [
        "service-account.disable",
        "service-account.delete",
        "key.disable",
        "key.delete",
        "instance.stop",
        "instance.delete",
    ] {
        assert_eq!(reconcile(case(op)).unwrap(), State::Complete);
        let mut v = case(op);
        v["before"]["status"] = json!(403);
        assert_eq!(reconcile(v), Err(Error::InvalidResponse));
        let mut v = case(op);
        v["after"]["status"] = json!(403);
        assert_eq!(reconcile(v).unwrap(), State::Unknown);
        let mut v = case(op);
        v["mutation"]["status"] = Value::Null;
        assert_eq!(reconcile(v).unwrap(), State::Unknown);
        let mut v = case(op);
        v["mutation"]["status"] = json!(503);
        assert_eq!(reconcile(v).unwrap(), State::Unknown);
        let mut v = case(op);
        v["mutation"]["status"] = json!(403);
        assert_eq!(reconcile(v).unwrap(), State::Rejected);
    }
}
#[test]
fn compute_completion_requires_operation_target_zone_request_and_errors() {
    for (key, bad) in [
        ("targetId", json!("1")),
        ("targetLink", json!("https://example.invalid/vm")),
        (
            "zone",
            json!("https://compute.googleapis.com/compute/v1/projects/foreign/zones/us-central1-a"),
        ),
        ("clientOperationId", json!("other")),
        ("operationType", json!("delete")),
    ] {
        let mut v = case("instance.stop");
        v["mutation"]["body"][key] = bad;
        assert_eq!(reconcile(v), Err(Error::ScopeMismatch));
    }
    let mut v = case("instance.stop");
    v["mutation"]["body"]["error"] = json!({"errors":[{"code":"FAILURE"}]});
    assert_eq!(reconcile(v).unwrap(), State::OperationFailed);
    let mut v = case("instance.stop");
    v["mutation"]["body"]["status"] = json!("RUNNING");
    assert_eq!(reconcile(v).unwrap(), State::Pending);
    let mut v = case("instance.stop");
    v["mutation"]["body"]["status"] = json!("FUTURE_STATE");
    assert_eq!(reconcile(v), Err(Error::InvalidResponse));
    let mut v = case("instance.stop");
    v["after"]["body"]["id"] = json!("9999999999999999999");
    assert_eq!(reconcile(v).unwrap(), State::TargetChanged);
    let mut v = case("instance.stop");
    v["after"]["body"]["id"] = json!(7363510519170619800u64);
    assert_eq!(reconcile(v).unwrap(), State::TargetChanged);
}
#[test]
fn rejects_wildcard_cross_project_injected_uri_request_ids_and_body_fields() {
    for project in ["-", "foreign/../sample-project", "sample-project?key=x"] {
        let mut r = request("service-account.delete");
        r["account"]["account"] = json!(project);
        assert!(prepare(r).is_err());
    }
    for id in [
        "",
        "00000000-0000-0000-0000-000000000000",
        "12345678-1234-1234-1234-123456789abc&x=y",
    ] {
        let mut r = request("instance.delete");
        r["request_id"] = json!(id);
        assert!(prepare(r).is_err());
    }
    let mut r = request("key.delete");
    r["target"]["key_id"] = json!("../keys/private");
    assert!(prepare(r).is_err());
    let mut r = request("instance.delete");
    r["target"]["name"] = json!("https://example.invalid");
    assert!(prepare(r).is_err());
    let mut r = request("service-account.disable");
    r["request_id"] = json!("12345678-1234-1234-1234-123456789abc");
    assert!(prepare(r).is_err());
    let mut v = case("key.disable");
    v["after"]["body"]["keyType"] = json!("SYSTEM_MANAGED");
    assert_eq!(reconcile(v).unwrap(), State::TargetChanged);
    let mut v = case("service-account.disable");
    v["after"]["body"]["disabled"] = json!("true");
    assert_eq!(reconcile(v), Err(Error::InvalidResponse));
}
#[test]
fn operation_reads_cannot_substitute_another_operation_or_hide_absent_evidence() {
    let mut v = case("instance.stop");
    v["operation"] = v["mutation"].clone();
    v["operation"]["body"]["name"] = json!("operation-other");
    assert_eq!(reconcile(v), Err(Error::ScopeMismatch));
    let mut v = case("instance.stop");
    v["operation"] = json!({"status":null,"body":null});
    assert_eq!(reconcile(v).unwrap(), State::Unknown);
}

#[test]
fn old_plan_or_policy_digest_cannot_silently_rebind() {
    let mut v = case("instance.stop");
    v["plan_sha256"] = json!("0".repeat(64));
    assert_eq!(reconcile(v), Err(Error::ScopeMismatch));
}
