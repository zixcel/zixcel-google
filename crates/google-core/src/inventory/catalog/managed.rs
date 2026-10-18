use super::{Model, Spec};
use crate::GoogleApi;
pub fn specs() -> Vec<Spec> {
    vec![
        Spec {
            id: "sql.instances.list",
            api: GoogleApi::Sql,
            path: "/sql/v1beta4/projects/{project}/instances",
            array: "items",
            aggregate: false,
            model: Model::Sql,
            project: true,
            billing: false,
            paged: true,
            permissions: &["resourcemanager.projects.get", "cloudsql.instances.list"],
        },
        Spec {
            id: "storage.buckets.list",
            api: GoogleApi::Storage,
            path: "/storage/v1/b",
            array: "items",
            aggregate: false,
            model: Model::Bucket,
            project: true,
            billing: false,
            paged: true,
            permissions: &["resourcemanager.projects.get", "storage.buckets.list"],
        },
        Spec {
            id: "run.services.list",
            api: GoogleApi::Run,
            path: "/v2/projects/{project}/locations/-/services",
            array: "services",
            aggregate: false,
            model: Model::Run,
            project: true,
            billing: false,
            paged: true,
            permissions: &["resourcemanager.projects.get", "run.services.list"],
        },
    ]
}
