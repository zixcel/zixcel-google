use super::*;
pub(super) fn observe(value: &Value) -> Result<Instance> {
    let mut disks = Vec::new();
    for disk in array(value, "disks")? {
        if !disk.is_object() {
            return Err(Error::InvalidResponse);
        }
        disks.push(AttachedDisk {
            source: text(disk, "source")?,
            boot: flag(disk, "boot")?,
            auto_delete: flag(disk, "autoDelete")?,
            mode: text(disk, "mode")?,
        });
    }
    let metadata = nested(value, "metadata")?;
    let mut startup = false;
    let mut container = false;
    if let Some(metadata) = metadata {
        for entry in array(metadata, "items")? {
            let key = required(entry, "key")?;
            startup |= matches!(
                key.as_str(),
                "startup-script"
                    | "startup-script-url"
                    | "windows-startup-script-ps1"
                    | "windows-startup-script-url"
            );
            container |= key == "gce-container-declaration";
        }
    }
    Ok(Instance {
        created_at: text(value, "creationTimestamp")?,
        machine_type: text(value, "machineType")?,
        cpu_platform: text(value, "cpuPlatform")?,
        last_started_at: text(value, "lastStartTimestamp")?,
        last_stopped_at: text(value, "lastStopTimestamp")?,
        disks,
        instance_startup_script_present: metadata.map(|_| startup),
        instance_container_declaration_present: metadata.map(|_| container),
        runtime_contents_observed: false,
    })
}
