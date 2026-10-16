
#[derive(Debug, Clone)]
pub struct LabResult {
    pub date: String,
    pub test: String,
    pub value: String,
    pub units: Option<String>,
    pub reference_range: Option<String>,
}
