#[derive(Debug, Clone)]
pub struct Appointment {
    pub date: String,
    pub department: String,
    pub status: Option<String>,
}
