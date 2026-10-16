#[derive(Debug, Clone)]
pub struct Allergy {
    pub substance: String,
    pub reaction: Option<String>,
}
