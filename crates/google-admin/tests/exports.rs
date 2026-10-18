use serde_json::{Value, json};
use std::collections::{BTreeMap, VecDeque};
use zixcel_google_admin::{
    Error, GoogleRead, ReadRequest, Result,
    exports::{self, Query},
};
mod export_fixtures;
use export_fixtures::*;
struct Fake {
    replies: VecDeque<Value>,
    urls: Vec<String>,
}
impl GoogleRead for Fake {
    fn read(&mut self, r: &ReadRequest) -> Result<Vec<u8>> {
        self.urls.push(r.url());
        Ok(serde_json::to_vec(&self.replies.pop_front().expect("unexpected request")).unwrap())
    }
}
fn invoke(operation: &str, args: &[(&str, &str)], replies: Vec<Value>) -> (Result<Value>, Fake) {
    let mut fake = Fake {
        replies: replies.into(),
        urls: vec![],
    };
    let args = args
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect::<BTreeMap<_, _>>();
    let result = Query::parse(operation, &args).and_then(|q| exports::execute(&mut fake, &q));
    (result, fake)
}
#[test]
fn discovery_binds_dataset_scopes_filters_non_billing_tables_and_reports_paging() {
    let (v, f) = invoke(
        "billing.exports.discover",
        &[("project", "sample-project")],
        vec![json!({
            "datasets":[{"datasetReference":{"projectId":"sample-project","datasetId":"sample_billing"}}],"nextPageToken":"opaque"
        })],
    );
    let v = v.unwrap();
    assert_eq!(v["coverage"]["complete"], false);
    assert!(f.urls[0].starts_with(
        "https://bigquery.googleapis.com/bigquery/v2/projects/sample-project/datasets?"
    ));
    let (v, _) = invoke(
        "billing.exports.discover",
        &[("project", "sample-project"), ("dataset", "sample_billing")],
        vec![json!({"tables":[
            {"tableReference":{"projectId":"sample-project","datasetId":"sample_billing","tableId":TABLE}},
            {"tableReference":{"projectId":"sample-project","datasetId":"sample_billing","tableId":"private_unrelated"}}
        ]})],
    );
    assert_eq!(v.unwrap()["records"].as_array().unwrap().len(), 1);
    let (v, _) = invoke(
        "billing.exports.discover",
        &[("project", "sample-project")],
        vec![json!({"datasets":[
            {"datasetReference":{"projectId":"other-project","datasetId":"sample_billing"}}
        ]})],
    );
    assert_eq!(v.unwrap_err(), Error::ScopeMismatch);
}
#[test]
fn exported_decimal_rows_retain_credits_scope_month_and_do_not_echo_unrelated_columns() {
    let m = metadata();
    let d = json!({"totalRows":"1","rows":[row()],"etag":"page-etag"});
    let (v, f) = invoke("billing.exports.read", &args(), vec![m.clone(), d, m]);
    let v = v.unwrap();
    assert_eq!(v["records"][0]["cost"], "9007199254740993.000000001");
    assert_eq!(v["records"][0]["credits"][0], "-0.123456789123");
    assert_eq!(v["records"][0]["invoice_month"], "202609");
    assert_eq!(v["records"][0]["project_id"], "sample-project");
    assert_eq!(v["coverage"]["complete"], true);
    assert_eq!(v["query_job_created"], false);
    assert!(!v.to_string().contains("SYNTHETIC_PRIVATE_LABEL"));
    assert!(f.urls[1].contains("/data?maxResults=50&selectedFields="));
    assert!(f.urls.iter().all(|u| !u.contains("/jobs")));
}
#[test]
fn table_changes_and_continuation_do_not_become_complete_invoice_snapshots() {
    let m = metadata();
    let mut changed = m.clone();
    changed["etag"] = json!("changed");
    let (v, _) = invoke(
        "billing.exports.read",
        &args(),
        vec![m.clone(), json!({"totalRows":"1","rows":[row()]}), changed],
    );
    assert_eq!(v.unwrap()["coverage"]["complete"], false);
    let (v, _) = invoke(
        "billing.exports.read",
        &args(),
        vec![
            m.clone(),
            json!({"totalRows":"100","pageToken":"next","rows":[row()]}),
            m,
        ],
    );
    let v = v.unwrap();
    assert_eq!(v["coverage"]["complete"], false);
    assert_eq!(v["coverage"]["next_page_token"], "next");
}

mod export_failures;
