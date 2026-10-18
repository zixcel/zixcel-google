use super::super::{catalog::Model, configuration::*, decode::*};
use crate::{Error, Result};
use serde_json::Value;
mod instance;
pub(super) fn observe(value: &Value, model: Model) -> Result<Option<Configuration>> {
    let created_at = text(value, "creationTimestamp")?;
    Ok(match model {
        Model::Instance => Some(Configuration::Instance(instance::observe(value)?)),
        Model::Disk => Some(Configuration::Disk(Disk {
            created_at,
            disk_type: text(value, "type")?,
            source_image: text(value, "sourceImage")?,
            source_snapshot: text(value, "sourceSnapshot")?,
            attached_users: value
                .get("users")
                .map(|v| {
                    let users = v.as_array().ok_or(Error::InvalidResponse)?;
                    if users.len() > 1000 {
                        return Err(Error::InvalidResponse);
                    }
                    Ok(users.len() as u16)
                })
                .transpose()?,
        })),
        Model::Snapshot => Some(Configuration::Snapshot(Snapshot {
            created_at,
            source_disk: text(value, "sourceDisk")?,
            disk_size_gb: quantity(value, "diskSizeGb")?,
        })),
        Model::Bucket => Some(Configuration::Bucket(Bucket {
            created_at: text(value, "timeCreated")?,
            storage_class: text(value, "storageClass")?,
            versioning_enabled: nested(value, "versioning")?
                .map(|v| flag(v, "enabled"))
                .transpose()?
                .flatten(),
            soft_delete_retention_seconds: nested(value, "softDeletePolicy")?
                .map(|v| quantity(v, "retentionDurationSeconds"))
                .transpose()?
                .flatten(),
            retention_period_seconds: nested(value, "retentionPolicy")?
                .map(|v| quantity(v, "retentionPeriod"))
                .transpose()?
                .flatten(),
            object_contents_observed: false,
        })),
        _ => None,
    })
}
fn nested<'a>(value: &'a Value, key: &str) -> Result<Option<&'a Value>> {
    value
        .get(key)
        .map(|v| {
            if v.is_object() {
                Ok(v)
            } else {
                Err(Error::InvalidResponse)
            }
        })
        .transpose()
}
