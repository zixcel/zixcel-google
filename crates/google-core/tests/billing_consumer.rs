use sha2::{Digest, Sha256};
use std::{
    fs,
    os::unix::fs::{DirBuilderExt, PermissionsExt},
    path::Path,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
fn material(root: &Path, name: &str, bytes: &[u8]) {
    use std::io::Write;
    fs::write(root.join(name), bytes).unwrap();
    let hash = Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(root.join("CONTENTS.sha256"))
        .unwrap();
    writeln!(f, "{hash}  {name}").unwrap();
}
#[test]
fn billing_report_consumer_pages_existing_exports_and_discards_credentials() {
    let root = std::env::temp_dir().join(format!(
        "zix-billing-consumer-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
    for name in ["cli", "google", "prior"] {
        fs::create_dir(root.join(name)).unwrap();
    }
    let cli = root.join("cli");
    let google = root.join("google");
    // Synthetic shell controller only; actual HTTP adapters are tested separately.
    let stub=br##"#!/usr/bin/env bash
set -eu
if [[ $1 == extensions ]]; then printf '{}'; exit 0; fi
[[ $1 == google && ${ZIX_GOOGLE_ACCESS_TOKEN:-} == SYNTHETIC_ONLY ]] || exit 9
shift
if [[ $1 == overview ]]; then
 printf '{"result":{"selected_scans_complete":true}}'
elif [[ $3 == discover ]]; then
 if [[ $* == *--dataset* ]]; then
  printf '{"result":{"records":[{"table":"gcp_billing_export_v1_012345_678901_ABCDEF","billing_account":"012345-678901-ABCDEF"}],"coverage":{"complete":true}}}'
 else
  printf '{"result":{"records":[{"dataset":"sample_billing"}],"coverage":{"complete":true}}}'
 fi
elif [[ $3 == read ]]; then
 if [[ $* == *--page-token* ]]; then cursor=null; else cursor='"next"'; fi
 printf '{"result":{"source":{"table_etag":"stable"},"records":[{"cost":"0.123456789123","currency":"USD","credits":["-0.01"]}],"coverage":{"next_page_token":%s,"table_stable_during_page_read":true}}}' "$cursor"
else exit 10; fi
"##;
    material(&cli, "zix", stub);
    fs::set_permissions(cli.join("zix"), fs::Permissions::from_mode(0o700)).unwrap();
    material(&google, "zix-google", b"synthetic-unused");
    material(&google, "google.extension.json", b"{}");
    let sources = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../playground");
    material(
        &google,
        "billing-export-observations.sh",
        &fs::read(sources.join("billing-export-observations.sh")).unwrap(),
    );
    fs::write(
        root.join("prior/project-targets.json"),
        br#"["sample-project"]"#,
    )
    .unwrap();
    let invoke = |name: &str, token: bool| {
        let mut command = Command::new("bash");
        command
            .arg(sources.join("inspect-billing-local.sh"))
            .arg(&cli)
            .arg(&google)
            .arg(root.join("prior"))
            .arg(root.join(name))
            .stdin(std::process::Stdio::null());
        command.env_remove("ZIX_GOOGLE_ACCESS_TOKEN");
        if token {
            command.env("ZIX_GOOGLE_ACCESS_TOKEN", "SYNTHETIC_ONLY");
        }
        command.output().unwrap()
    };
    let result = invoke("report", true);
    assert!(result.status.success(), "{:?}", result);
    let value: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("report/summary.json")).unwrap()).unwrap();
    assert_eq!(value["billingPagesRead"], 2);
    assert_eq!(value["invoiceFinalityConfirmed"], false);
    assert!(!root.join("report/registry").exists());
    for entry in fs::read_dir(root.join("report")).unwrap() {
        let p = entry.unwrap().path();
        assert!(!fs::read_to_string(p).unwrap().contains("SYNTHETIC_ONLY"));
    }
    let result = invoke("missing", false);
    assert_eq!(result.status.code(), Some(2));
    assert!(!root.join("missing/registry").exists());
    assert!(!root.join("missing/commands.jsonl").exists());
    fs::remove_dir_all(root).unwrap();
}
