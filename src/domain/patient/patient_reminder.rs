
#[derive(Debug, Clone)]
pub struct PatientReminder {
    pub reminder: String,
    pub due_date: Option<String>,
    pub status: Option<String>,
}
