use serde_json::{Value, json};
use std::collections::{BTreeMap, VecDeque};
use zixcel_google_admin::{
    Error, GoogleRead, ReadRequest, Result,
    inventory::{self, query::InventoryQuery},
};
struct Fake {
    replies: VecDeque<Result<Vec<u8>>>,
    paths: Vec<String>,
}
impl Fake {
    fn values(values: Vec<Value>) -> Self {
        Self {
            replies: values
                .into_iter()
                .map(|v| Ok(serde_json::to_vec(&v).unwrap()))
                .collect(),
            paths: vec![],
        }
    }
}
impl GoogleRead for Fake {
    fn read(&mut self, request: &ReadRequest) -> Result<Vec<u8>> {
        self.paths.push(request.url());
        self.replies.pop_front().expect("unexpected request")
    }
}
fn project() -> Value {
    json!({"name":"projects/123456789012","projectId":"sample-project","state":"ACTIVE","displayName":"SYNTHETIC_DISPLAY_ONLY"})
}
fn query(op: &str, args: &[(&str, &str)]) -> InventoryQuery {
    InventoryQuery::parse(
        op,
        &args
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
    )
    .unwrap()
}
mod admission;
mod binding;
mod data_safety;
mod overview;
mod pagination;
mod routing;

mod routing_discovery;
mod routing_resources;
