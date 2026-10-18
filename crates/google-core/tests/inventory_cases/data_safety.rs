use super::*;
#[test]
fn malformed_or_oversized_provider_payloads_do_not_turn_into_zero_cost_reports() {
    for body in [
        json!({"projects":{}}),
        json!({"error":{"message":"SYNTHETIC_PRIVATE_ERROR"}}),
        json!({"projects":[{"name":"projects/123456789012","projectId":123}]}),
    ] {
        let mut fake = Fake::values(vec![body]);
        assert!(inventory::inspect(&mut fake, &query("projects.list", &[])).is_err());
    }
    let mut fake = Fake {
        replies: vec![Ok(vec![b' '; 1024 * 1024 + 1])].into(),
        paths: vec![],
    };
    assert_eq!(
        inventory::inspect(&mut fake, &query("projects.list", &[])).unwrap_err(),
        Error::ResponseTooLarge
    );
}
#[test]
fn api_metadata_secrets_and_free_form_configuration_are_not_forwarded() {
    let mut p = project();
    p["labels"] = json!({"private":"SYNTHETIC_PRIVATE_VALUE"});
    p["displayName"] = json!("SYNTHETIC_PRIVATE_VALUE");
    let mut fake = Fake::values(vec![json!({"projects":[p]})]);
    let report = inventory::inspect(&mut fake, &query("projects.list", &[])).unwrap();
    assert!(
        !serde_json::to_string(&report)
            .unwrap()
            .contains("SYNTHETIC_PRIVATE_VALUE")
    );
}
