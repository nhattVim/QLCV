use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TeamDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub member_count: usize,
    #[serde(default)]
    pub workspace_count: usize,
    #[serde(default)]
    pub task_count: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkspaceDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub team_id: String,
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub task_count: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserSafe {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub email: String,
    pub full_name: String,
    pub role: String, // "super_admin", "admin", "user"
    pub is_active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskAttachment {
    pub id: String,
    pub file_name: String,
    pub file_size: u64,
    pub content_type: String,
    pub object_key: String,
    pub uploaded_by: String,
    pub uploaded_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
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
    #[serde(default)]
    pub attachments: Vec<TaskAttachment>,
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
