use super::*;
#[test]
fn pages_are_collected_without_reusing_or_interpreting_the_cursor() {
    let token = "opaque+/=&";
    let mut second = project();
    second["name"] = json!("projects/222222222222");
    second["projectId"] = json!("second-project");
    let mut fake = Fake::values(vec![
        json!({"projects":[project()],"nextPageToken":token}),
        json!({"projects":[second]}),
    ]);
    let report = inventory::inspect(&mut fake, &query("projects.list", &[])).unwrap();
    assert_eq!(report.records.len(), 2);
    assert!(report.coverage.complete);
    assert_eq!(report.coverage.pages_read, 2);
    assert!(fake.paths[1].contains("pageToken=opaque%2B%2F%3D%26"));
}
#[test]
fn page_budget_and_unreachable_regions_remain_incomplete() {
    let mut fake = Fake::values(vec![
        json!({"projects":[project()],"nextPageToken":"next-page"}),
    ]);
    let report =
        inventory::inspect(&mut fake, &query("projects.list", &[("max-pages", "1")])).unwrap();
    assert!(!report.coverage.complete);
    assert_eq!(
        report.coverage.next_page_token.as_deref(),
        Some("next-page")
    );
    assert_eq!(fake.paths.len(), 1);
    let mut fake = Fake::values(vec![
        project(),
        json!({"services":[],"unreachable":["region-a"]}),
    ]);
    let report = inventory::inspect(
        &mut fake,
        &query("run.services.list", &[("project", "sample-project")]),
    )
    .unwrap();
    assert!(!report.coverage.complete);
    assert!(report.records.is_empty());
}
#[test]
fn repeated_cursors_and_duplicate_records_are_not_complete_snapshots() {
    let mut second = project();
    second["name"] = json!("projects/222222222222");
    second["projectId"] = json!("second-project");
    let mut fake = Fake::values(vec![
        json!({"projects":[project()],"nextPageToken":"cycle"}),
        json!({"projects":[second],"nextPageToken":"cycle"}),
    ]);
    assert_eq!(
        inventory::inspect(&mut fake, &query("projects.list", &[("max-pages", "3")])).unwrap_err(),
        Error::InvalidResponse
    );
    assert_eq!(fake.paths.len(), 2);
    let mut fake = Fake::values(vec![
        json!({"projects":[project()],"nextPageToken":"page-two"}),
        json!({"projects":[project()]}),
    ]);
    assert_eq!(
        inventory::inspect(&mut fake, &query("projects.list", &[])).unwrap_err(),
        Error::InvalidResponse
    );
}
#[test]
fn compute_empty_scope_warnings_and_partial_provider_failures_are_distinct() {
    let mut fake = Fake::values(vec![
        project(),
        json!({"items":{"zones/zone-a":{"warning":{"code":"NO_RESULTS_ON_PAGE"}}}}),
    ]);
    assert!(
        inventory::inspect(
            &mut fake,
            &query("compute.instances.list", &[("project", "sample-project")])
        )
        .unwrap()
        .coverage
        .complete
    );
    let mut fake = Fake::values(vec![
        project(),
        json!({"items":{"zones/zone-a":{"warning":{"code":"UNREACHABLE"}}}}),
    ]);
    assert!(
        !inventory::inspect(
            &mut fake,
            &query("compute.instances.list", &[("project", "sample-project")])
        )
        .unwrap()
        .coverage
        .complete
    );
}

#[test]
fn record_budget_refuses_a_new_complete_snapshot() {
    let mut pages = Vec::new();
    for page in 0..6 {
        let values = (0..100)
            .map(|n| {
                let id = page * 100 + n;
                json!({"name":format!("projects/{}",100000+id),"projectId":format!("sample-p{id}")})
            })
            .collect::<Vec<_>>();
        pages.push(json!({"projects":values,"nextPageToken":format!("page-{}",page+1)}));
    }
    let mut f = Fake::values(pages);
    assert_eq!(
        inventory::inspect(&mut f, &query("projects.list", &[("max-pages", "8")])).unwrap_err(),
        Error::ResponseTooLarge
    );
    assert_eq!(f.paths.len(), 6);
}
