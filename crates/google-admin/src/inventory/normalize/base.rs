use super::super::{
    catalog::Model,
    decode::{required, text},
    model::Record,
};
use crate::Result;
use serde_json::Value;
pub(super) fn record(value: &Value, model: Model) -> Result<Record> {
    let name = required(value, "name")?;
    let kind = match model {
        Model::Project => "project",
        Model::BillingAccount => "billing_account",
        Model::ProjectBilling => "project_billing",
        Model::Service => "enabled_api",
        Model::Account => "service_account",
        Model::Asset => "asset",
        Model::Instance => "compute_instance",
        Model::Disk => "disk",
        Model::Address => "ip_address",
        Model::Snapshot => "snapshot",
        Model::Commitment => "commitment",
        Model::Sql => "sql_instance",
        Model::Bucket => "bucket",
        Model::Run => "run_service",
    };
    let r = Record {
        name,
        kind,
        state: text(value, "state")?.or(text(value, "status")?),
        project_id: text(value, "projectId")?,
        project_number: None,
        billing_account: None,
        billing_enabled: None,
        open: None,
        location: text(value, "location")?
            .or(text(value, "region")?)
            .or(text(value, "zone")?.map(|s| s.rsplit('/').next().unwrap_or(&s).to_owned())),
        quantity: None,
        quantity_unit: None,
        resource_type: None,
        service_account_email: None,
        end_time: None,
        commitment_plan: None,
        charge_hint: "not_determined",
    };
    Ok(r)
}
