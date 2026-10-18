use super::{Model, Spec};
use crate::GoogleApi;
pub fn specs() -> Vec<Spec> {
    vec![
        Spec {
            id: "billing.accounts.list",
            api: GoogleApi::Billing,
            path: "/v1/billingAccounts",
            array: "billingAccounts",
            aggregate: false,
            model: Model::BillingAccount,
            project: false,
            billing: false,
            paged: true,
            permissions: &[],
        },
        Spec {
            id: "billing.projects.list",
            api: GoogleApi::Billing,
            path: "/v1/billingAccounts/{billing}/projects",
            array: "projectBillingInfo",
            aggregate: false,
            model: Model::ProjectBilling,
            project: false,
            billing: true,
            paged: true,
            permissions: &["billing.resourceAssociations.list"],
        },
        Spec {
            id: "billing.project.inspect",
            api: GoogleApi::Billing,
            path: "/v1/projects/{project}/billingInfo",
            array: "",
            aggregate: false,
            model: Model::ProjectBilling,
            project: true,
            billing: false,
            paged: false,
            permissions: &[
                "resourcemanager.projects.get",
                "resourcemanager.projects.get",
            ],
        },
    ]
}
