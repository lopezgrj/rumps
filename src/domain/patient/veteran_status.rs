
#[derive(Debug, Clone)]
pub struct VeteranStatus {
    pub is_veteran: bool,
    pub service_branch: Option<String>,
    pub period_of_service: Option<String>,
}
