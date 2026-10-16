#[derive(Debug, Clone)]
pub struct InpatientData {
    pub admission_date: String,
    pub discharge_date: Option<String>,
    pub ward: Option<String>,
}
