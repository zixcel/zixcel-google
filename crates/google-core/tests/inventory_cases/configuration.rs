use super::*;
fn observe(op: &str, body: Value) -> Result<Value> {
    let mut fake = Fake::values(vec![project(), body]);
    serde_json::to_value(inventory::inspect(
        &mut fake,
        &query(op, &[("project", "sample-project")]),
    )?)
    .map_err(|_| Error::InvalidResponse)
}
#[test]
fn instance_configuration_explains_structure_without_disclosing_metadata_values() {
    let value = observe("compute.instances.list", json!({"items":{"zones/zone-a":{"instances":[{
        "name":"sample-vm","status":"RUNNING","machineType":"zones/zone-a/machineTypes/e2-small",
        "cpuPlatform":"Synthetic CPU","creationTimestamp":"2024-01-01T00:00:00Z",
        "disks":[{"source":"projects/sample-project/zones/zone-a/disks/sample-disk","boot":true,"autoDelete":false,"mode":"READ_WRITE"}],
        "metadata":{"items":[{"key":"startup-script","value":"SYNTHETIC_SECRET"},
            {"key":"gce-container-declaration","value":"SYNTHETIC_SECRET"}]},
        "serviceAccounts":[{"email":"SYNTHETIC_SECRET"}],"labels":{"private":"SYNTHETIC_SECRET"}
    }]}}})).unwrap();
    let c = &value["records"][0]["configuration"];
    assert_eq!(c["machine_type"], "zones/zone-a/machineTypes/e2-small");
    assert_eq!(c["disks"][0]["auto_delete"], false);
    assert_eq!(c["instance_startup_script_present"], true);
    assert_eq!(c["instance_container_declaration_present"], true);
    assert_eq!(c["runtime_contents_observed"], false);
    assert!(!value.to_string().contains("SYNTHETIC_SECRET"));
}
#[test]
fn storage_lineage_and_retention_are_structured_and_do_not_imply_contents_access() {
    let disk = observe("compute.disks.list", json!({"items":{"zones/a":{"disks":[{
        "name":"sample-disk","sizeGb":"10","sourceImage":"projects/base-project/global/images/base-image",
        "users":["private-user-reference"],"type":"zones/a/diskTypes/pd-balanced"
    }]}}})).unwrap();
    assert_eq!(disk["records"][0]["configuration"]["attached_users"], 1);
    assert!(!disk.to_string().contains("private-user-reference"));
    let snapshot = observe("compute.snapshots.list", json!({"items":[{
        "name":"sample-snapshot","sourceDisk":"projects/sample-project/zones/a/disks/sample-disk",
        "diskSizeGb":"9007199254740993","storageBytes":"123"
    }]})).unwrap();
    assert_eq!(
        snapshot["records"][0]["configuration"]["disk_size_gb"],
        "9007199254740993"
    );
    let bucket = observe("storage.buckets.list", json!({"items":[{"name":"sample-bucket",
        "storageClass":"STANDARD","versioning":{"enabled":false},
        "softDeletePolicy":{"retentionDurationSeconds":"604800"},"retentionPolicy":{"retentionPeriod":"123"},
        "acl":["SYNTHETIC_SECRET"],"labels":{"private":"SYNTHETIC_SECRET"}
    }]})).unwrap();
    let c = &bucket["records"][0]["configuration"];
    assert_eq!(c["soft_delete_retention_seconds"], "604800");
    assert_eq!(c["versioning_enabled"], false);
    assert_eq!(c["object_contents_observed"], false);
    assert!(!bucket.to_string().contains("SYNTHETIC_SECRET"));
}
#[test]
fn omitted_and_malformed_configuration_remain_unknown_or_errors() {
    let result = observe(
        "storage.buckets.list",
        json!({"items":[{"name":"sample-bucket"}]}),
    )
    .unwrap();
    assert!(result["records"][0]["configuration"]["versioning_enabled"].is_null());
    for field in ["versioning", "retentionPolicy", "softDeletePolicy"] {
        let mut item = json!({"name":"sample-bucket"});
        item[field] = json!("malformed");
        assert_eq!(
            observe("storage.buckets.list", json!({"items":[item]})).unwrap_err(),
            Error::InvalidResponse
        );
    }
    let value = observe(
        "compute.instances.list",
        json!({"items":{"zones/a":{"instances":[{"name":"sample-vm"}]}}}),
    )
    .unwrap();
    assert!(value["records"][0]["configuration"]["instance_startup_script_present"].is_null());
    assert_eq!(
        observe(
            "compute.instances.list",
            json!({"items":{"zones/a":{"instances":[{
                "name":"sample-vm","metadata":{"items":[{"key":23}]}
            }]}}})
        )
        .unwrap_err(),
        Error::InvalidResponse
    );
}
