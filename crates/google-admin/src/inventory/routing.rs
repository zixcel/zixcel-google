use super::{catalog::Spec, query::InventoryQuery, scope::ProjectScope};
use crate::{GoogleApi, ReadRequest, Result};
pub fn request(
    spec: Spec,
    query: &InventoryQuery,
    scope: Option<&ProjectScope>,
    page: Option<&str>,
) -> Result<ReadRequest> {
    let project = scope
        .map(|s| s.id.as_str())
        .or(query.project.as_deref())
        .unwrap_or("");
    let number = scope.map(|s| s.number.as_str()).unwrap_or(project);
    let path = spec
        .path
        .replace("{project}", project)
        .replace("{number}", number)
        .replace("{billing}", query.billing_account.as_deref().unwrap_or(""));
    let mut args = Vec::new();
    if spec.paged {
        let size = match spec.api {
            GoogleApi::Compute | GoogleApi::Sql | GoogleApi::Storage => "maxResults",
            _ => "pageSize",
        };
        args.push((size.into(), "50".into()));
        if let Some(page) = page {
            args.push(("pageToken".into(), page.into()));
        }
    }
    match spec.id {
        "services.list" => args.push(("filter".into(), "state:ENABLED".into())),
        "storage.buckets.list" => {
            args.push(("project".into(), project.into()));
            args.push(("projection".into(), "noAcl".into()));
        }
        "assets.search" => args.push((
            "readMask".into(),
            "name,assetType,project,location,state".into(),
        )),
        _ => {}
    }
    if spec.api == GoogleApi::Compute {
        args.push(("returnPartialSuccess".into(), "true".into()));
    }
    ReadRequest::new(spec.api, path, args)
}
