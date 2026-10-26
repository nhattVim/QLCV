mod models;
mod db;
mod storage;

use db::DbState;
use models::{DashboardStats, TaskAttachment, TaskDto, UserSafe};
use tauri::State;

#[tauri::command]
async fn get_connection_status(state: State<'_, DbState>) -> Result<bool, String> {
    let db_lock = state.db.read().await;
    Ok(db_lock.is_some())
}

#[tauri::command]
async fn reconnect_mongo(state: State<'_, DbState>, uri: String) -> Result<bool, String> {
    state.connect(&uri).await?;
    Ok(true)
}

#[tauri::command]
async fn login(state: State<'_, DbState>, email: String, password: String) -> Result<UserSafe, String> {
    state.authenticate(&email, &password).await
}

#[tauri::command]
async fn get_users(state: State<'_, DbState>) -> Result<Vec<UserSafe>, String> {
    state.get_all_users().await
}

#[tauri::command]
async fn create_user(
    state: State<'_, DbState>,
    full_name: String,
    email: String,
    password: String,
    role: String,
) -> Result<UserSafe, String> {
    state.create_user(&full_name, &email, &password, &role).await
}

#[tauri::command]
async fn update_user(
    state: State<'_, DbState>,
    email: String,
    full_name: Option<String>,
    role: Option<String>,
    is_active: Option<bool>,
    password: Option<String>,
) -> Result<bool, String> {
    state.update_user(&email, full_name, role, is_active, password).await
}

#[tauri::command]
async fn delete_user(state: State<'_, DbState>, email: String) -> Result<bool, String> {
    state.delete_user(&email).await
}

#[tauri::command]
async fn get_tasks(
    state: State<'_, DbState>,
    status: Option<String>,
    priority: Option<String>,
    user_email: Option<String>,
) -> Result<Vec<TaskDto>, String> {
    state.get_tasks(status, priority, user_email).await
}

#[tauri::command]
async fn create_task(
    state: State<'_, DbState>,
    title: String,
    description: String,
    assigned_to: Vec<String>,
    priority: String,
    due_date: Option<String>,
    created_by: String,
    remind_before_days: Option<i32>,
    attachments: Option<Vec<TaskAttachment>>,
) -> Result<TaskDto, String> {
    state
        .create_task(
            &title,
            &description,
            assigned_to,
            &priority,
            due_date,
            &created_by,
            remind_before_days,
            attachments,
        )
        .await
}

#[tauri::command]
async fn update_task(
    state: State<'_, DbState>,
    id: String,
    title: String,
    description: String,
    assigned_to: Vec<String>,
    priority: String,
    due_date: Option<String>,
    status: String,
    remind_before_days: Option<i32>,
    attachments: Option<Vec<TaskAttachment>>,
) -> Result<bool, String> {
    state
        .update_task(
            &id,
            &title,
            &description,
            assigned_to,
            &priority,
            due_date,
            &status,
            remind_before_days,
            attachments,
        )
        .await
}

#[tauri::command]
async fn update_task_status(
    state: State<'_, DbState>,
    id: String,
    status: String,
) -> Result<bool, String> {
    state.update_task_status(&id, &status).await
}

#[tauri::command]
async fn delete_task(state: State<'_, DbState>, id: String) -> Result<bool, String> {
    state.delete_task(&id).await
}

#[tauri::command]
async fn get_dashboard_stats(
    state: State<'_, DbState>,
    for_user: Option<String>,
) -> Result<DashboardStats, String> {
    state.get_dashboard_stats(for_user).await
}

// ==========================================
// ATTACHMENT COMMANDS (CLOUDFLARE R2 / S3)
// ==========================================

#[tauri::command]
async fn is_storage_configured() -> Result<bool, String> {
    Ok(storage::is_storage_configured())
}

#[tauri::command]
async fn upload_attachment(
    state: State<'_, DbState>,
    task_id: Option<String>,
    file_name: String,
    file_data: Vec<u8>,
    content_type: String,
    uploaded_by: String,
) -> Result<TaskAttachment, String> {
    let att = storage::upload_attachment(&file_name, &file_data, &content_type, &uploaded_by).await?;

    if let Some(tid) = task_id {
        if !tid.trim().is_empty() {
            state.add_task_attachment(&tid, att.clone()).await?;
        }
    }

    Ok(att)
}

#[tauri::command]
async fn delete_attachment(
    state: State<'_, DbState>,
    task_id: Option<String>,
    attachment_id: String,
    object_key: String,
) -> Result<bool, String> {
    let _ = storage::delete_attachment(&object_key).await?;

    if let Some(tid) = task_id {
        if !tid.trim().is_empty() {
            state.remove_task_attachment(&tid, &attachment_id).await?;
        }
    }

    Ok(true)
}

#[tauri::command]
async fn get_attachment_url(object_key: String) -> Result<String, String> {
    storage::get_attachment_download_url(&object_key).await
}

#[tauri::command]
async fn open_attachment(object_key: String, file_name: String) -> Result<String, String> {
    storage::open_attachment_in_viewer(&object_key, &file_name).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db_state = DbState::new();

    // Async init MongoDB in background thread on startup
    let state_clone = db_state.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = state_clone.init().await {
            eprintln!("[DB Warning] Initial MongoDB connection error: {}", e);
        } else {
            println!("[DB Success] Connected to MongoDB Atlas successfully!");
        }
    });

    tauri::Builder::default()
        .manage(db_state)
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![
            get_connection_status,
            reconnect_mongo,
            login,
            get_users,
            create_user,
            update_user,
            delete_user,
            get_tasks,
            create_task,
            update_task,
            update_task_status,
            delete_task,
            get_dashboard_stats,
            is_storage_configured,
            upload_attachment,
            delete_attachment,
            get_attachment_url,
            open_attachment,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
