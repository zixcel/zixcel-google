use serde_json::{Value, json};
pub const TABLE: &str = "gcp_billing_export_v1_012345_678901_ABCDEF";
pub fn args() -> Vec<(&'static str, &'static str)> {
    vec![
        ("project", "sample-project"),
        ("dataset", "sample_billing"),
        ("table", TABLE),
        ("billing-account", "012345-678901-ABCDEF"),
    ]
}
pub fn metadata() -> Value {
    json!({"etag":"metadata-etag","type":"TABLE","lastModifiedTime":"1234567890",
    "tableReference":{"projectId":"sample-project","datasetId":"sample_billing","tableId":TABLE},
    "schema":{"fields":[
        {"name":"billing_account_id","type":"STRING"},
        {"name":"project","type":"RECORD","fields":[{"name":"id","type":"STRING"},{"name":"name","type":"STRING"}]},
        {"name":"service","type":"RECORD","fields":[{"name":"description","type":"STRING"}]},
        {"name":"sku","type":"RECORD","fields":[{"name":"description","type":"STRING"}]},
        {"name":"cost","type":"FLOAT"},{"name":"currency","type":"STRING"},
        {"name":"invoice","type":"RECORD","fields":[{"name":"month","type":"STRING"}]},
        {"name":"credits","type":"RECORD","mode":"REPEATED","fields":[{"name":"amount","type":"FLOAT"},{"name":"name","type":"STRING"}]},
        {"name":"usage_start_time","type":"TIMESTAMP"},{"name":"usage_end_time","type":"TIMESTAMP"},
        {"name":"cost_type","type":"STRING"}
    ]}})
}
pub fn row() -> Value {
    json!({"f":[{"v":"012345-678901-ABCDEF"},
        {"v":{"f":[{"v":"sample-project"},{"v":"SYNTHETIC_PRIVATE_LABEL"}]}},
        {"v":{"f":[{"v":"Compute Engine"}]}},{"v":{"f":[{"v":"Snapshot storage"}]}},
        {"v":"9007199254740993.000000001"},{"v":"USD"},
        {"v":{"f":[{"v":"202609"}]}},
        {"v":[{"v":{"f":[{"v":"-0.123456789123"},{"v":"SYNTHETIC_PRIVATE_LABEL"}]}}]},
        {"v":"1788220800"},{"v":"1788224400"},{"v":"regular"}
    ]})
}
