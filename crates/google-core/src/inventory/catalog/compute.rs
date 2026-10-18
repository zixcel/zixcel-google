use super::{Model, Spec};
use crate::GoogleApi;
pub fn specs() -> Vec<Spec> {
    vec![
        Spec {
            id: "compute.instances.list",
            api: GoogleApi::Compute,
            path: "/compute/v1/projects/{project}/aggregated/instances",
            array: "instances",
            aggregate: true,
            model: Model::Instance,
            project: true,
            billing: false,
            paged: true,
            permissions: &["resourcemanager.projects.get", "compute.instances.list"],
        },
        Spec {
            id: "compute.disks.list",
            api: GoogleApi::Compute,
            path: "/compute/v1/projects/{project}/aggregated/disks",
            array: "disks",
            aggregate: true,
            model: Model::Disk,
            project: true,
            billing: false,
            paged: true,
            permissions: &["resourcemanager.projects.get", "compute.disks.list"],
        },
        Spec {
            id: "compute.snapshots.list",
            api: GoogleApi::Compute,
            path: "/compute/v1/projects/{project}/global/snapshots",
            array: "items",
            aggregate: false,
            model: Model::Snapshot,
            project: true,
            billing: false,
            paged: true,
            permissions: &["resourcemanager.projects.get", "compute.snapshots.list"],
        },
        Spec {
            id: "compute.commitments.list",
            api: GoogleApi::Compute,
            path: "/compute/v1/projects/{project}/aggregated/commitments",
            array: "commitments",
            aggregate: true,
            model: Model::Commitment,
            project: true,
            billing: false,
            paged: true,
            permissions: &["resourcemanager.projects.get", "compute.commitments.list"],
        },
    ]
}
