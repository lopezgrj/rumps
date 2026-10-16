#[derive(Debug, Clone)]
pub struct EmergencyContact {
    pub name: String,
    pub relationship: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
}
