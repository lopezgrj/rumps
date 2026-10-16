
#[derive(Debug, Clone)]
pub struct Vital {
    pub date: String,
    pub type_: String,
    pub value: String,
    pub units: Option<String>,
}
