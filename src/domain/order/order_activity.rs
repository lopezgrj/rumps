#[derive(Debug, Clone)]
pub struct OrderActivity {
    pub activity_type: String, // e.g., create, update, sign
    pub date: Option<String>,
    pub by: Option<String>,
    pub details: Option<String>,
}
