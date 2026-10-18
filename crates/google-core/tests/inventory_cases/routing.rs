use super::*;
#[test]
fn every_inventory_uses_its_fixed_api_and_expected_provider_shape() {
    let cases = super::routing_discovery::cases()
        .into_iter()
        .chain(super::routing_resources::cases());
    for (op, body, url, scoped) in cases {
        let args = if scoped {
            vec![("project", "sample-project")]
        } else {
            vec![]
        };
        let replies = if scoped && op != "projects.inspect" {
            vec![project(), body]
        } else {
            vec![body]
        };
        let mut fake = Fake::values(replies);
        let result = inventory::inspect(&mut fake, &query(op, &args)).unwrap();
        assert_eq!(result.records.len(), 1, "{op}");
        assert!(result.coverage.complete, "{op}");
        assert!(fake.paths.last().unwrap().starts_with(url), "{op}");
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("SYNTHETIC_PRIVATE_CONFIGURATION"));
        if op == "compute.disks.list" {
            assert_eq!(
                result.records[0].quantity.as_deref(),
                Some("9007199254740993")
            );
        }
        if op == "services.list" {
            assert!(
                fake.paths
                    .last()
                    .unwrap()
                    .contains("filter=state%3AENABLED")
            );
        }
        if op == "storage.buckets.list" {
            assert!(fake.paths.last().unwrap().contains("projection=noAcl"));
        }
    }
}
#[test]
fn billing_projects_are_scoped_to_the_selected_account() {
    let mut fake = Fake::values(vec![
        json!({"projectBillingInfo":[{"name":"projects/sample-project/billingInfo","projectId":"sample-project","billingAccountName":"billingAccounts/012345-678901-ABCDEF","billingEnabled":true}]}),
    ]);
    let report = inventory::inspect(
        &mut fake,
        &query(
            "billing.projects.list",
            &[("billing-account", "012345-678901-ABCDEF")],
        ),
    )
    .unwrap();
    assert_eq!(report.records[0].billing_enabled, Some(true));
    assert!(fake.paths[0].starts_with(
        "https://cloudbilling.googleapis.com/v1/billingAccounts/012345-678901-ABCDEF/projects"
    ));
}

#[test]
fn reserved_addresses_are_exposed_as_potential_cost_not_free_resources() {
    let mut fake = Fake::values(vec![
        project(),
        json!({"items":{"regions/region-a":{"addresses":[{"name":"sample-address","status":"RESERVED","region":"regions/region-a"}]}}}),
    ]);
    let result = inventory::inspect(
        &mut fake,
        &query("compute.addresses.list", &[("project", "sample-project")]),
    )
    .unwrap();
    assert_eq!(result.records[0].state.as_deref(), Some("RESERVED"));
    assert_eq!(
        result.records[0].charge_hint,
        "allocated_IP_address_may_be_billable"
    );
    assert!(fake.paths[1].contains("/aggregated/addresses"));
}
