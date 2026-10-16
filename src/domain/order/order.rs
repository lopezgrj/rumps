use super::order_action::OrderAction;
use super::order_activity::OrderActivity;

#[derive(Debug, Clone)]
pub struct Order {
    pub id: String, // OrderIEN
    pub patient_id: String, // Patient DFN
    pub order_type: String, // e.g., Lab, Med, Consult
    pub status: String,     // Current status
    pub provider: Option<String>,
    pub location: Option<String>,
    pub start_date: Option<String>,
    pub stop_date: Option<String>,
    pub order_text: Option<String>,
    pub orderable_item: Option<String>,
    pub actions: Vec<OrderAction>,
    pub activity_log: Vec<OrderActivity>,
    pub reason: Option<String>,
    pub signature: Option<String>,
    pub verification: Option<String>,
}
