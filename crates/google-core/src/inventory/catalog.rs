use crate::GoogleApi;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Model {
    Project,
    BillingAccount,
    ProjectBilling,
    Service,
    Account,
    Asset,
    Instance,
    Disk,
    Address,
    Snapshot,
    Commitment,
    Sql,
    Bucket,
    Run,
}
#[derive(Clone, Copy)]
pub struct Spec {
    pub id: &'static str,
    pub api: GoogleApi,
    pub path: &'static str,
    pub array: &'static str,
    pub aggregate: bool,
    pub model: Model,
    pub project: bool,
    pub billing: bool,
    pub paged: bool,
    pub permissions: &'static [&'static str],
}
mod access;
mod billing;
mod compute;
mod managed;
mod network;
mod projects;
pub fn specifications() -> Vec<Spec> {
    [
        projects::specs(),
        billing::specs(),
        access::specs(),
        compute::specs(),
        managed::specs(),
        network::specs(),
    ]
    .into_iter()
    .flatten()
    .collect()
}
pub fn specification(id: &str) -> Option<Spec> {
    specifications().into_iter().find(|s| s.id == id)
}
