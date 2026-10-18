use super::{
    catalog::Model,
    decode::{flag, quantity, required, text},
    model::Record,
    scope::ProjectScope,
};
use crate::{Error, Result};
use serde_json::Value;
mod base;
mod billing;
mod configuration;
pub fn record(
    value: &Value,
    model: Model,
    scope: Option<&ProjectScope>,
    billing: Option<&str>,
) -> Result<Record> {
    if !value.is_object() {
        return Err(Error::InvalidResponse);
    }
    if let Some(scope) = scope {
        scope.bind(value)?;
    }
    let mut r = base::record(value, model)?;
    match model {
        Model::Project => {
            crate::query::validate_project(r.project_id.as_deref().ok_or(Error::InvalidResponse)?)
                .map_err(|_| Error::InvalidResponse)?;
            let number = r
                .name
                .strip_prefix("projects/")
                .ok_or(Error::InvalidResponse)?;
            if !(1..=20).contains(&number.len()) || !number.bytes().all(|b| b.is_ascii_digit()) {
                return Err(Error::InvalidResponse);
            }
            r.project_number = Some(number.into());
        }
        Model::BillingAccount => billing::account(value, &mut r)?,
        Model::ProjectBilling => billing::project(value, &mut r, billing)?,
        Model::Account => {
            let email = required(value, "email")?;
            if email.matches('@').count() != 1
                || !email.ends_with(".gserviceaccount.com")
                || !email
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"@.-_".contains(&b))
            {
                return Err(Error::InvalidResponse);
            }
            r.service_account_email = Some(email);
            r.state =
                flag(value, "disabled")?.map(|b| if b { "DISABLED" } else { "ENABLED" }.into());
        }
        Model::Asset => {
            r.resource_type = Some(required(value, "assetType")?);
            r.charge_hint = "asset_presence_does_not_prove_current_usage";
        }
        Model::Address => {
            r.charge_hint = "allocated_IP_address_may_be_billable";
        }
        Model::Disk => {
            r.quantity = quantity(value, "sizeGb")?;
            r.quantity_unit = Some("sizeGb");
            r.charge_hint = "allocated_storage_may_be_billable";
        }
        Model::Snapshot => {
            r.quantity = quantity(value, "storageBytes")?;
            r.quantity_unit = Some("bytes");
            r.charge_hint = "retained_snapshot_may_be_billable";
        }
        Model::Commitment => {
            r.end_time = text(value, "endTimestamp")?;
            r.commitment_plan = text(value, "plan")?;
            r.charge_hint = "commitment_may_continue_without_running_instances";
        }
        Model::Sql => {
            if let Some(settings) = value.get("settings") {
                r.quantity = quantity(settings, "dataDiskSizeGb")?;
                r.quantity_unit = Some("dataDiskSizeGb");
            }
            r.charge_hint = "database_and_storage_may_be_billable";
        }
        Model::Bucket => {
            r.charge_hint = "stored_data_may_be_billable_even_without_active_compute";
        }
        Model::Instance => {
            r.charge_hint = "execution_or_attached_resources_may_be_billable";
        }
        Model::Run => {
            r.state = value
                .get("terminalCondition")
                .map(|v| text(v, "state"))
                .transpose()?
                .flatten();
            r.charge_hint = "requests_or_provisioned_capacity_may_be_billable";
        }
        Model::Service => {
            r.charge_hint = "enabled_api_is_not_running_resource_or_charge_evidence";
        }
    }
    Ok(r)
}
