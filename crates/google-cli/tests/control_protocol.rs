use serde_json::{Value, json};
use std::{
    io::Write,
    process::{Command, Stdio},
};
fn invoke(argument: &str, input: &[u8]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_zix-google"))
        .env_clear()
        .env("ZIX_GOOGLE_ACCESS_TOKEN", "SYNTHETIC_MUST_NOT_BE_RESOLVED")
        .arg(argument)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let _ = child.stdin.take().unwrap().write_all(input);
    child.wait_with_output().unwrap()
}
fn request() -> Value {
    json!({"schema":"zix-google-control/v1","operation":"project.set-labels",
        "account":{"provider":"google-cloud","subject":"synthetic-owner","account":"1234567890"},
        "input":{"labels":{"env":"test"},"etag":"observed-etag"}})
}
#[test]
fn local_control_protocol_ignores_credentials_and_only_returns_a_bound_plan() {
    let output = invoke(
        "--prepare-control-v1",
        &serde_json::to_vec(&request()).unwrap(),
    );
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let plan: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(plan["operation"], "project.set-labels");
    assert_eq!(plan["request"]["method"], "PATCH");
    assert_eq!(plan["account"], request()["account"]);
    assert!(!String::from_utf8_lossy(&output.stdout).contains("SYNTHETIC_MUST_NOT_BE_RESOLVED"));
    assert!(plan.get("grant").is_none());
    assert!(plan.get("receipt").is_none());
    let output = invoke("--control-manifest", b"");
    assert!(output.status.success());
    let catalog: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(catalog["plan_only"], true);
    assert_eq!(catalog["approval_required"], true);
}
#[test]
fn local_protocol_rejects_oversized_unknown_secret_and_invalid_scope_inputs() {
    for (field, value) in [
        ("schema", json!("other/v1")),
        ("credential", json!("SYNTHETIC_PRIVATE")),
        ("operation", json!("arbitrary.delete")),
        ("approval", json!(true)),
    ] {
        let mut input = request();
        input[field] = value;
        let output = invoke("--prepare-control-v1", &serde_json::to_vec(&input).unwrap());
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(!String::from_utf8_lossy(&output.stderr).contains("SYNTHETIC_PRIVATE"));
    }
    let output = invoke("--prepare-control-v1", &vec![b'x'; 65537]);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let mut input = request();
    input["account"]["account"] = json!("../other");
    assert!(
        !invoke("--prepare-control-v1", &serde_json::to_vec(&input).unwrap())
            .status
            .success()
    );
}
