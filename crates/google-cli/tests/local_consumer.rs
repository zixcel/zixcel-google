use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    fs,
    os::unix::fs::{DirBuilderExt, PermissionsExt},
    path::Path,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
fn material(root: &Path, name: &str, bytes: &[u8]) {
    fs::write(root.join(name), bytes).unwrap();
    let sha = Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    use std::io::Write;
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(root.join("CONTENTS.sha256"))
        .unwrap();
    writeln!(f, "{sha}  {name}").unwrap();
}
#[test]
fn report_consumer_merges_billing_only_targets_and_keeps_partial_failures() {
    let root = std::env::temp_dir().join(format!(
        "zix-controller-test-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
    let cli = root.join("cli");
    let google = root.join("google");
    fs::create_dir(&cli).unwrap();
    fs::create_dir(&google).unwrap();
    // This stub tests the shell report controller, not Google HTTP or native dispatch.
    let fixture=br##"#!/usr/bin/env bash
set -eu
if [[ $1 == extensions ]]; then printf '{"state":"updated"}\n'; exit 0; fi
[[ $1 == google && ${ZIX_GOOGLE_ACCESS_TOKEN:-} == SYNTHETIC_ONLY ]] || exit 9
shift
if [[ $1 == projects ]]; then
 printf '{"result":{"records":[],"coverage":{"complete":true}}}\n'
elif [[ $1 == billing && $2 == accounts ]]; then
 printf '{"result":{"records":[{"name":"billingAccounts/012345-678901-ABCDEF"}],"coverage":{"complete":true}}}\n'
elif [[ $1 == billing && $2 == projects ]]; then
 printf '{"result":{"records":[{"project_id":"sample-hidden-project"}],"coverage":{"complete":true}}}\n'
elif [[ $1 == overview && $4 == sample-hidden-project ]]; then
 printf '{"result":{"selected_scans_complete":false,"sections":[{"error":"forbidden","count":null}]}}\n'
else
 exit 10
fi
"##;
    material(&cli, "zix", fixture);
    fs::set_permissions(cli.join("zix"), fs::Permissions::from_mode(0o700)).unwrap();
    material(&google, "google.extension.json", b"{}");
    material(&google, "zix-google", b"synthetic-unused-program");
    let script =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../playground/inspect-accessible-local.sh");
    let output = root.join("reports");
    let result = Command::new("bash")
        .arg(&script)
        .arg(&cli)
        .arg(&google)
        .arg(&output)
        .env("ZIX_GOOGLE_ACCESS_TOKEN", "SYNTHETIC_ONLY")
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(2));
    let summary: Value =
        serde_json::from_slice(&fs::read(output.join("summary.json")).unwrap()).unwrap();
    assert_eq!(summary["discoveredProjects"], 0);
    assert_eq!(summary["discoveredProjectTargets"], 1);
    assert_eq!(summary["selectedProjectCount"], 1);
    assert_eq!(summary["allDiscoveredTargetsInspected"], false);
    assert_eq!(summary["credentialsSaved"], false);
    assert!(!output.join("registry").exists());
    for entry in fs::read_dir(&output).unwrap() {
        let file = entry.unwrap().path();
        if file.is_file() {
            assert!(!fs::read_to_string(file).unwrap().contains("SYNTHETIC_ONLY"));
        }
    }
    let missing = root.join("missing-credential");
    let result = Command::new("bash")
        .arg(script)
        .arg(cli)
        .arg(google)
        .arg(&missing)
        .env_remove("ZIX_GOOGLE_ACCESS_TOKEN")
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(2));
    assert!(!missing.join("commands.jsonl").exists());
    assert!(!missing.join("registry").exists());
    fs::remove_dir_all(root).unwrap();
}
