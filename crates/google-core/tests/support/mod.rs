use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use zixcel_google_core::{Error, GoogleRead, KeyQuery, ReadRequest};
pub struct Fake {
    pub replies: Vec<Result<Vec<u8>, Error>>,
    pub paths: Vec<String>,
}
impl GoogleRead for Fake {
    fn read(&mut self, request: &ReadRequest) -> Result<Vec<u8>, Error> {
        self.paths.push(request.path_and_query());
        self.replies.remove(0)
    }
}
pub fn query() -> KeyQuery {
    KeyQuery {
        project: "sample-project".into(),
        service_account: "reader-account@sample-project.iam.gserviceaccount.com".into(),
        key_id: "a".repeat(40),
    }
}
pub fn account() -> Value {
    json!({"projectId":"sample-project","email":query().service_account,"uniqueId":"123456789012345678901",
        "name":"projects/sample-project/serviceAccounts/123456789012345678901"})
}
pub fn key() -> Value {
    json!({"name":format!("projects/-/serviceAccounts/123456789012345678901/keys/{}",query().key_id),
        "keyType":"USER_MANAGED","validAfterTime":"2020-01-01T00:00:00Z","validBeforeTime":"9999-12-31T23:59:59Z"})
}
pub fn now() -> DateTime<Utc> {
    DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
        .unwrap()
        .into()
}
pub fn fake(a: Value, keys: Value) -> Fake {
    Fake {
        replies: vec![
            Ok(serde_json::to_vec(&a).unwrap()),
            Ok(serde_json::to_vec(&keys).unwrap()),
        ],
        paths: vec![],
    }
}
