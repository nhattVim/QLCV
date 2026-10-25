use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserSafe {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub email: String,
    pub full_name: String,
    pub role: String, // "admin" or "user"
    pub is_active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub title: String,
    pub description: String,
    pub assigned_to: Vec<String>,
    pub priority: String, // "Urgent", "High", "Medium", "Low"
    pub status: String,   // "TODO", "IN_PROGRESS", "REVIEW", "DONE"
    pub due_date: Option<String>,
    pub created_by: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub remind_before_days: Option<i32>,
    pub remind_at: Option<String>,
    pub reminded_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DashboardStats {
    pub total: usize,
    pub todo: usize,
    pub in_progress: usize,
    pub review: usize,
    pub done: usize,
    pub overdue: usize,
    pub urgent_or_near: usize,
}
