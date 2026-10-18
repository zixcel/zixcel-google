use super::{Model, Spec};
use crate::GoogleApi;
pub fn specs() -> Vec<Spec> {
    vec![Spec {
        id: "compute.addresses.list",
        api: GoogleApi::Compute,
        path: "/compute/v1/projects/{project}/aggregated/addresses",
        array: "addresses",
        aggregate: true,
        model: Model::Address,
        project: true,
        billing: false,
        paged: true,
        permissions: &["resourcemanager.projects.get", "compute.addresses.list"],
    }]
}
