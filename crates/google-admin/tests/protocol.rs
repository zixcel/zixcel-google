use serde_json::{Value, json};
use std::{
    io::Write,
    process::{Command, Stdio},
};
fn request() -> Value {
    json!({"schema":"zix-provider/v1","id":"a".repeat(32),"operation":"iam.keys.inspect",
        "arguments":{"project":"sample-project","service-account":"reader-account@sample-project.iam.gserviceaccount.com","key-id":"b".repeat(40)},
        "limits":{"timeout_ms":35000,"maximum_response_bytes":131072}})
}
fn invoke(bytes: &[u8]) -> std::process::Output {
    let mut c = Command::new(env!("CARGO_BIN_EXE_zix-google"))
        .env_clear()
        .arg("--zix-provider-v1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let _ = c.stdin.take().unwrap().write_all(bytes);
    c.wait_with_output().unwrap()
}
#[test]
fn missing_credential_is_a_bound_error_without_using_gcloud_or_real_key() {
    let input = request();
    let out = invoke(&serde_json::to_vec(&input).unwrap());
    assert!(out.status.success());
    let response: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(response["id"], input["id"]);
    assert_eq!(response["operation"], input["operation"]);
    assert_eq!(response["error"], "credential_missing");
    assert!(response.get("result").is_none());
}
#[test]
fn mismatched_version_operation_unknown_fields_and_resource_injection_are_refused() {
    for (field, value) in [
        ("schema", json!("other/v1")),
        ("operation", json!("iam.keys.delete")),
        ("id", json!("bad")),
        ("unexpected", json!(true)),
    ] {
        let mut input = request();
        input[field] = value;
        let out = invoke(&serde_json::to_vec(&input).unwrap());
        assert!(!out.status.success());
        assert!(out.stdout.is_empty());
    }
    let mut input = request();
    input["arguments"]["project"] = json!("../../SYNTHETIC_PRIVATE_INPUT");
    let out = invoke(&serde_json::to_vec(&input).unwrap());
    assert!(!out.status.success());
    assert!(!String::from_utf8_lossy(&out.stderr).contains("SYNTHETIC_PRIVATE_INPUT"));
}
#[test]
fn bounded_input_and_manifest_declare_only_the_implemented_read_operation() {
    let out = invoke(&vec![b'x'; 65537]);
    assert!(!out.status.success());
    assert!(out.stdout.is_empty());
    let out = Command::new(env!("CARGO_BIN_EXE_zix-google"))
        .env_clear()
        .arg("--manifest")
        .output()
        .unwrap();
    assert!(out.status.success());
    let m: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(m["commands"].as_array().unwrap().len(), 19);
    assert!(
        m["commands"]
            .as_array()
            .unwrap()
            .iter()
            .all(|c| c["effect"] == "read")
    );
    assert!(serde_json::to_vec(&m).unwrap().len() <= 16384);
    assert_eq!(m["commands"][0]["effect"], "read");
    assert_eq!(
        m["credential_environment"],
        json!(["ZIX_GOOGLE_ACCESS_TOKEN"])
    );
}
