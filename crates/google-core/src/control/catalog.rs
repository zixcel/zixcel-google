use serde::Serialize;
#[derive(Clone, Copy, Serialize)]
pub struct Operation {
    pub id: &'static str,
    pub effect: &'static str,
    pub required_permissions: &'static [&'static str],
    pub request_charge: &'static str,
    pub creates_billable_resources: bool,
}
pub const OPERATIONS: [Operation; 6] = [
    operation("accounts.search", "read", &["resourcemanager.projects.get"]),
    operation("services.list", "read", &["serviceusage.services.list"]),
    operation("account.read", "read", &["resourcemanager.projects.get"]),
    operation("permissions.test", "read", &[]),
    operation(
        "project.set-labels",
        "write",
        &["resourcemanager.projects.update"],
    ),
    operation(
        "billing.project-read",
        "read",
        &["resourcemanager.projects.get"],
    ),
];
const fn operation(
    id: &'static str,
    effect: &'static str,
    permissions: &'static [&'static str],
) -> Operation {
    Operation {
        id,
        effect,
        required_permissions: permissions,
        request_charge: "not_assessed",
        creates_billable_resources: false,
    }
}
pub fn catalog() -> serde_json::Value {
    serde_json::json!({"schema":super::SCHEMA,"plan_only":true,"approval_required":true,
        "allowed_origins":[crate::GoogleApi::Resource.origin(),crate::GoogleApi::Billing.origin(),crate::GoogleApi::Usage.origin()],
        "operations":OPERATIONS})
}
