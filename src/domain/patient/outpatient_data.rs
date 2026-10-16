
#[derive(Debug, Clone)]
pub struct OutpatientData {
    pub visit_date: String,
    pub clinic: Option<String>,
    pub provider: Option<String>,
}
