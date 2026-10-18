use serde::Serialize;
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Configuration {
    Instance(Instance),
    Disk(Disk),
    Snapshot(Snapshot),
    Bucket(Bucket),
}
#[derive(Clone, Debug, Serialize)]
pub struct Instance {
    pub created_at: Option<String>,
    pub machine_type: Option<String>,
    pub cpu_platform: Option<String>,
    pub last_started_at: Option<String>,
    pub last_stopped_at: Option<String>,
    pub disks: Vec<AttachedDisk>,
    pub instance_startup_script_present: Option<bool>,
    pub instance_container_declaration_present: Option<bool>,
    pub runtime_contents_observed: bool,
}
#[derive(Clone, Debug, Serialize)]
pub struct AttachedDisk {
    pub source: Option<String>,
    pub boot: Option<bool>,
    pub auto_delete: Option<bool>,
    pub mode: Option<String>,
}
#[derive(Clone, Debug, Serialize)]
pub struct Disk {
    pub created_at: Option<String>,
    pub disk_type: Option<String>,
    pub source_image: Option<String>,
    pub source_snapshot: Option<String>,
    pub attached_users: Option<u16>,
}
#[derive(Clone, Debug, Serialize)]
pub struct Snapshot {
    pub created_at: Option<String>,
    pub source_disk: Option<String>,
    pub disk_size_gb: Option<String>,
}
#[derive(Clone, Debug, Serialize)]
pub struct Bucket {
    pub created_at: Option<String>,
    pub storage_class: Option<String>,
    pub versioning_enabled: Option<bool>,
    pub soft_delete_retention_seconds: Option<String>,
    pub retention_period_seconds: Option<String>,
    pub object_contents_observed: bool,
}
