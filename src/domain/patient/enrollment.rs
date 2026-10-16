#[derive(Debug, Clone)]
pub struct Enrollment {
    pub program: String,
    pub enrollment_date: Option<String>,
    pub status: Option<String>,
}
