use zixcel_google::{build_plan, doctor, parse_config};

const CONFIG_A: &str = r#"
schema = "zixcel://google/config/v1"
config_id = "publishing"
workspace_ref = "workspace:publishing"
secret_ref = "secret://google/workspace/publisher"
services = ["drive", "analytics", "adsense"]
"#;

#[test]
fn plan_is_deterministic_after_normalization() {
    let second = CONFIG_A.replace(
        "[\"drive\", \"analytics\", \"adsense\"]",
        "[\"adsense\", \"drive\", \"analytics\"]",
    );
    let first = build_plan(&parse_config(CONFIG_A).expect("config")).expect("plan");
    let second = build_plan(&parse_config(&second).expect("config")).expect("plan");
    assert_eq!(first, second);
}

#[test]
fn contract_is_closed_bounded_and_secret_safe() {
    assert!(parse_config(&format!("{CONFIG_A}\naccess_token = \"no\"\n")).is_err());
    assert!(parse_config(&format!("{CONFIG_A}\nfuture = true\n")).is_err());
    assert!(parse_config(&" ".repeat(1_048_577)).is_err());
}

#[test]
fn doctor_proves_no_external_action_capability() {
    let report = doctor();
    assert!(!report.network_client_linked);
    assert!(!report.secret_resolution_enabled);
    assert!(!report.execution_enabled);
}
