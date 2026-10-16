#[derive(Debug, Clone)]
pub struct Admission {
    pub date: String,
    pub ward: String,
    pub discharge_date: Option<String>,
}
