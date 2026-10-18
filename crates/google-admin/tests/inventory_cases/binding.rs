use super::*;
#[test]
fn wrong_project_metadata_stops_before_resource_queries() {
    let mut wrong = project();
    wrong["projectId"] = json!("other-project");
    let mut fake = Fake::values(vec![wrong]);
    assert_eq!(
        inventory::inspect(
            &mut fake,
            &query("storage.buckets.list", &[("project", "sample-project")])
        )
        .unwrap_err(),
        Error::ScopeMismatch
    );
    assert_eq!(fake.paths.len(), 1);
}
#[test]
fn cross_project_assets_and_cross_billing_associations_are_refused() {
    let mut fake = Fake::values(vec![
        project(),
        json!({"results":[{"name":"//storage.googleapis.com/sample-bucket","assetType":"storage.googleapis.com/Bucket","project":"projects/999999999999"}]}),
    ]);
    assert_eq!(
        inventory::inspect(
            &mut fake,
            &query("assets.search", &[("project", "sample-project")])
        )
        .unwrap_err(),
        Error::ScopeMismatch
    );
    let mut fake = Fake::values(vec![
        json!({"projectBillingInfo":[{"name":"projects/sample-project/billingInfo","projectId":"sample-project","billingAccountName":"billingAccounts/999999-999999-FFFFFF","billingEnabled":true}]}),
    ]);
    assert_eq!(
        inventory::inspect(
            &mut fake,
            &query(
                "billing.projects.list",
                &[("billing-account", "012345-678901-ABCDEF")]
            )
        )
        .unwrap_err(),
        Error::ScopeMismatch
    );
}
#[test]
fn denied_or_unavailable_queries_are_never_empty_successful_inventory() {
    for error in [
        Error::Unauthorized,
        Error::Forbidden,
        Error::Unavailable,
        Error::Deadline,
        Error::RateLimited,
    ] {
        let mut fake = Fake {
            replies: vec![Err(error)].into(),
            paths: vec![],
        };
        assert_eq!(
            inventory::inspect(&mut fake, &query("projects.list", &[])).unwrap_err(),
            error
        );
    }
}
#[test]
fn numeric_scope_alias_is_bound_to_observed_project_identity() {
    let mut fake = Fake::values(vec![project(), json!({"services":[]})]);
    let result = inventory::inspect(
        &mut fake,
        &query("services.list", &[("project", "123456789012")]),
    )
    .unwrap();
    assert_eq!(result.scope.as_deref(), Some("sample-project"));
    assert!(fake.paths[1].contains("/projects/123456789012/services"));
}

#[test]
fn missing_project_is_distinguished_from_a_missing_resource_collection() {
    let mut f = Fake {
        replies: vec![Err(Error::Unavailable)].into(),
        paths: vec![],
    };
    assert_eq!(
        inventory::inspect(
            &mut f,
            &query("storage.buckets.list", &[("project", "sample-project")])
        )
        .unwrap_err(),
        Error::ProjectUnavailable
    );
    assert_eq!(f.paths.len(), 1);
    let mut f = Fake::values(vec![project(), json!({})]);
    f.replies[1] = Err(Error::Unavailable);
    assert_eq!(
        inventory::inspect(
            &mut f,
            &query("storage.buckets.list", &[("project", "sample-project")])
        )
        .unwrap_err(),
        Error::Unavailable
    );
}
