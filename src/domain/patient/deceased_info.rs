#[derive(Debug, Clone)]
pub struct DeceasedInfo {
    pub date_time: Option<String>,
    pub location: Option<String>,
    pub cause: Option<String>,
    pub autopsy: Option<bool>,
    pub notified: Option<String>,
    pub notified_date: Option<String>,
    pub notified_by: Option<String>,
}
