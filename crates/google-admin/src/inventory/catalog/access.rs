use super::{Model, Spec};
use crate::GoogleApi;
pub fn specs() -> Vec<Spec> {
    vec![
        Spec {
            id: "services.list",
            api: GoogleApi::Usage,
            path: "/v1/projects/{number}/services",
            array: "services",
            aggregate: false,
            model: Model::Service,
            project: true,
            billing: false,
            paged: true,
            permissions: &["resourcemanager.projects.get", "serviceusage.services.list"],
        },
        Spec {
            id: "iam.accounts.list",
            api: GoogleApi::Iam,
            path: "/v1/projects/{project}/serviceAccounts",
            array: "accounts",
            aggregate: false,
            model: Model::Account,
            project: true,
            billing: false,
            paged: true,
            permissions: &["resourcemanager.projects.get", "iam.serviceAccounts.list"],
        },
        Spec {
            id: "assets.search",
            api: GoogleApi::Asset,
            path: "/v1/projects/{number}:searchAllResources",
            array: "results",
            aggregate: false,
            model: Model::Asset,
            project: true,
            billing: false,
            paged: true,
            permissions: &[
                "resourcemanager.projects.get",
                "cloudasset.assets.searchAllResources",
            ],
        },
    ]
}
