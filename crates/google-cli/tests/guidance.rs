use std::process::Command;
#[test]
fn guided_metadata_is_complete_without_credentials_and_legacy_bridge_remains_separate() {
    let (m, c) = zixcel_google_cli::guided::metadata().unwrap();
    assert_eq!(m.protocol, "zix-provider/v2");
    for op in &m.commands {
        assert!(
            c.entries
                .iter()
                .any(|g| g.operation == op.id && g.selectors.is_empty())
        );
    }
    let o = Command::new(env!("CARGO_BIN_EXE_zix-google"))
        .arg("--guidance")
        .env_clear()
        .output()
        .unwrap();
    assert!(o.status.success());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&o.stdout).unwrap(),
        serde_json::to_value(c).unwrap()
    );
    assert_eq!(zixcel_google_cli::protocol::SCHEMA, "zix-provider/v1");
}
