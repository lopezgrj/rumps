#[derive(Debug, Clone)]
pub struct InsuranceInfo {
    pub company: String,
    pub policy_number: String,
    pub group_number: Option<String>,
    pub effective_date: Option<String>,
    pub expiration_date: Option<String>,
}
