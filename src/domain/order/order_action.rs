#[derive(Debug, Clone)]
pub struct OrderAction {
    pub action_type: String, // e.g., sign, hold, cancel
    pub date: Option<String>,
    pub by: Option<String>,
    pub comment: Option<String>,
}
