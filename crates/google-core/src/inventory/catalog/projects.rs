use super::{Model, Spec};
use crate::GoogleApi;
pub fn specs() -> Vec<Spec> {
    vec![
        Spec {
            id: "projects.list",
            api: GoogleApi::Resource,
            path: "/v3/projects:search",
            array: "projects",
            aggregate: false,
            model: Model::Project,
            project: false,
            billing: false,
            paged: true,
            permissions: &["resourcemanager.projects.get"],
        },
        Spec {
            id: "projects.inspect",
            api: GoogleApi::Resource,
            path: "/v3/projects/{project}",
            array: "",
            aggregate: false,
            model: Model::Project,
            project: true,
            billing: false,
            paged: false,
            permissions: &["resourcemanager.projects.get"],
        },
    ]
}
