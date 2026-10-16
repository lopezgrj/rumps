
#[derive(Debug, Clone)]
pub struct Medication {
    pub name: String,
    pub dose: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}
