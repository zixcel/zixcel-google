use serde::Serialize;
#[derive(Clone, Debug, Serialize)]
pub struct Record {
    pub name: String,
    pub kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub billing_account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub billing_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity_unit: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_account_email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commitment_plan: Option<String>,
    pub charge_hint: &'static str,
}
#[derive(Clone, Debug, Serialize)]
pub struct Coverage {
    pub complete: bool,
    pub pages_read: u8,
    pub next_page_token: Option<String>,
    pub limitations: Vec<String>,
    pub visibility: &'static str,
}
#[derive(Clone, Debug, Serialize)]
pub struct Collection {
    pub operation: String,
    pub scope: Option<String>,
    pub observed_at: String,
    pub records: Vec<Record>,
    pub coverage: Coverage,
    pub cost_amount_status: &'static str,
    pub data_trust: &'static str,
}
