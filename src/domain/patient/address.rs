#[derive(Debug, Clone)]
pub struct Address {
    pub street: String,
    pub city: String,
    pub county: Option<String>,
    pub state: Option<String>,
    pub zip: Option<String>,
    pub country: Option<String>,
}
