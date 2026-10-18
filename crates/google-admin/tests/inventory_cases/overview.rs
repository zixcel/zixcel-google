use super::*;
#[test]
fn overview_keeps_independent_results_and_unknown_costs_after_permission_failures() {
    let mut fake = Fake::values(vec![
        project(),
        json!({"name":"projects/sample-project/billingInfo","projectId":"sample-project","billingEnabled":false}),
        json!({}),
        json!({}),
        json!({"items":{"zones/zone-a":{"instances":[{"name":"sample-vm","status":"TERMINATED"}]}}}),
        json!({}),
        json!({}),
        json!({}),
        json!({}),
        json!({}),
        json!({}),
        json!({}),
    ]);
    fake.replies.push_back(Ok(b"{}".to_vec()));
    fake.replies[3] = Err(Error::Forbidden);
    let overview = inventory::overview(
        &mut fake,
        &query("overview.project", &[("project", "sample-project")]),
    )
    .unwrap();
    let value = serde_json::to_value(overview).unwrap();
    assert_eq!(value["selected_scans_complete"], false);
    assert_eq!(value["sections"].as_array().unwrap().len(), 12);
    let assets = &value["sections"][2];
    assert_eq!(assets["error"], "forbidden");
    assert!(assets["count"].is_null());
    assert_eq!(value["sections"][3]["states"]["TERMINATED"], 1);
    assert!(
        value["cost_conclusion"]
            .as_str()
            .unwrap()
            .starts_with("unknown")
    );
    assert_eq!(fake.paths.len(), 13);
}
#[test]
fn accessible_overview_does_not_claim_visibility_of_unlisted_accounts() {
    let mut fake = Fake::values(vec![
        json!({}),
        json!({"billingAccounts":[{"name":"billingAccounts/012345-678901-ABCDEF","open":true}]}),
    ]);
    fake.replies[0] = Err(Error::Forbidden);
    let v = serde_json::to_value(
        inventory::overview(&mut fake, &query("overview.accessible", &[])).unwrap(),
    )
    .unwrap();
    assert_eq!(v["selected_scans_complete"], false);
    assert_eq!(v["sections"][1]["count"], 1);
    assert!(v["sections"][0]["count"].is_null());
    assert_eq!(fake.paths.len(), 2);
}
