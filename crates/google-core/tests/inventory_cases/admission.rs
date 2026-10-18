use super::*;
#[test]
fn unsupported_actions_scope_injection_and_unbounded_options_fail_before_transport() {
    for (op, args) in [
        ("projects.delete", vec![]),
        ("projects.inspect", vec![]),
        ("projects.inspect", vec![("project", "../other-project")]),
        ("projects.list", vec![("url", "https://other.invalid/")]),
        ("projects.list", vec![("max-pages", "0")]),
        ("projects.list", vec![("max-pages", "9")]),
        ("projects.list", vec![("page-token", "bad\nvalue")]),
        ("projects.list", vec![("quota-project", "../other-project")]),
        (
            "billing.projects.list",
            vec![("billing-account", "012345-678901-ABCDEF/projects")],
        ),
        ("overview.accessible", vec![("page-token", "cursor")]),
    ] {
        let args: BTreeMap<String, String> = args
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect();
        assert!(InventoryQuery::parse(op, &args).is_err(), "{op}");
    }
}
#[test]
fn long_opaque_cursor_is_allowed_but_never_becomes_a_path_or_header() {
    let token = "+".repeat(2048);
    let mut fake = Fake::values(vec![json!({})]);
    inventory::inspect(
        &mut fake,
        &query("projects.list", &[("page-token", &token)]),
    )
    .unwrap();
    assert!(
        fake.paths[0]
            .starts_with("https://cloudresourcemanager.googleapis.com/v3/projects:search?")
    );
    assert!(fake.paths[0].contains("pageToken=%2B%2B"));
}
