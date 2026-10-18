use serde_json::{Value, json};
pub fn argument_names(project: bool, billing: bool, paged: bool) -> Vec<&'static str> {
    let mut names = vec!["quota-project"];
    if project {
        names.push("project");
    }
    if billing {
        names.push("billing-account");
    }
    if paged {
        names.extend(["max-pages", "page-token"]);
    }
    names
}
fn arguments(project: bool, billing: bool, paged: bool) -> Vec<Value> {
    argument_names(project,billing,paged).into_iter().map(|name|json!({"name":name,
        "required":name=="project"||name=="billing-account", "maximum":match name{"page-token"=>2048,"max-pages"=>1,"billing-account"=>20,_=>30}})).collect()
}
pub fn commands() -> Vec<Value> {
    let mut commands=super::catalog::specifications().into_iter().map(|s|json!({"id":s.id,
        "words":s.id.split('.').collect::<Vec<_>>(),"effect":"read","description":format!("Observe {} within caller visibility; not a cost statement.",s.id),
        "required_permissions":s.permissions,"arguments":arguments(s.project,s.billing,s.paged)})).collect::<Vec<_>>();
    for (id, project) in [("overview.accessible", false), ("overview.project", true)] {
        let mut args = arguments(project, false, true);
        args.retain(|a| a["name"] != "page-token");
        commands.push(json!({"id":id,"words":id.split('.').collect::<Vec<_>>(),"effect":"read",
            "description":"Summarize selected read operations; report partial visibility and unknown charges.",
            "required_permissions":if project{super::catalog::specifications().iter().filter(|s|s.project).flat_map(|s|s.permissions.iter().copied()).collect::<std::collections::BTreeSet<_>>().into_iter().collect::<Vec<_>>()}else{vec!["resourcemanager.projects.get"]},"arguments":args}));
    }
    commands
}
