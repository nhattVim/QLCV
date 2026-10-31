mod models;
mod db;
mod storage;

use db::DbState;
use models::{DashboardStats, TaskAttachment, TaskDto, TeamDto, UserSafe, WorkspaceDto};
use tauri::State;

#[tauri::command]
async fn get_connection_status(state: State<'_, DbState>) -> Result<bool, String> {
    let db_lock = state.db.read().await;
    Ok(db_lock.is_some())
}

#[tauri::command]
async fn reconnect_mongo(state: State<'_, DbState>, uri: String) -> Result<bool, String> {
    state.connect(&uri).await?;
    state.init_default_data().await?;
    Ok(true)
}

#[tauri::command]
async fn login(state: State<'_, DbState>, email: String, password: String) -> Result<UserSafe, String> {
    state.authenticate(&email, &password).await
}

// ==========================================
// TEAMS COMMANDS
// ==========================================

#[tauri::command]
async fn get_teams(state: State<'_, DbState>) -> Result<Vec<TeamDto>, String> {
    state.get_teams().await
}

#[tauri::command]
async fn create_team(
    state: State<'_, DbState>,
    name: String,
    description: String,
    enable_review_step: Option<bool>,
    enable_result_column: Option<bool>,
) -> Result<TeamDto, String> {
    state.create_team(&name, &description, enable_review_step, enable_result_column).await
}

#[tauri::command]
async fn update_team(
    state: State<'_, DbState>,
    id: String,
    name: String,
    description: String,
    enable_review_step: Option<bool>,
    enable_result_column: Option<bool>,
) -> Result<bool, String> {
    state.update_team(&id, &name, &description, enable_review_step, enable_result_column).await
}

#[tauri::command]
async fn delete_team(state: State<'_, DbState>, id: String) -> Result<bool, String> {
    state.delete_team(&id).await
}

// ==========================================
// WORKSPACES COMMANDS
// ==========================================

#[tauri::command]
async fn get_workspaces(state: State<'_, DbState>, team_id: String) -> Result<Vec<WorkspaceDto>, String> {
    state.get_workspaces(&team_id).await
}

#[tauri::command]
async fn create_workspace(
    state: State<'_, DbState>,
    team_id: String,
    name: String,
    description: String,
    created_by: String,
) -> Result<WorkspaceDto, String> {
    state.create_workspace(&team_id, &name, &description, &created_by).await
}

#[tauri::command]
async fn update_workspace(
    state: State<'_, DbState>,
    id: String,
    name: String,
    description: String,
) -> Result<bool, String> {
    state.update_workspace(&id, &name, &description).await
}

#[tauri::command]
async fn delete_workspace(state: State<'_, DbState>, id: String) -> Result<bool, String> {
    state.delete_workspace(&id).await
}

// ==========================================
// USERS COMMANDS
// ==========================================

#[tauri::command]
async fn get_users(state: State<'_, DbState>, team_id: Option<String>) -> Result<Vec<UserSafe>, String> {
    state.get_all_users(team_id).await
}

#[tauri::command]
async fn create_user(
    state: State<'_, DbState>,
    full_name: String,
    email: String,
    password: String,
    role: String,
    team_id: Option<String>,
    job_title: Option<String>,
) -> Result<UserSafe, String> {
    state.create_user(&full_name, &email, &password, &role, team_id, job_title).await
}

#[tauri::command]
async fn update_user(
    state: State<'_, DbState>,
    email: String,
    full_name: Option<String>,
    role: Option<String>,
    is_active: Option<bool>,
    password: Option<String>,
    team_id: Option<String>,
    job_title: Option<String>,
) -> Result<bool, String> {
    state.update_user(&email, full_name, role, is_active, password, team_id, job_title).await
}

#[tauri::command]
async fn delete_user(state: State<'_, DbState>, email: String) -> Result<bool, String> {
    state.delete_user(&email).await
}

// ==========================================
// TASKS COMMANDS
// ==========================================

#[tauri::command]
async fn get_tasks(
    state: State<'_, DbState>,
    team_id: Option<String>,
    workspace_id: Option<String>,
    status: Option<String>,
    priority: Option<String>,
    user_email: Option<String>,
) -> Result<Vec<TaskDto>, String> {
    state.get_tasks(team_id, workspace_id, status, priority, user_email).await
}

#[tauri::command]
async fn create_task(
    state: State<'_, DbState>,
    team_id: Option<String>,
    workspace_id: Option<String>,
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
            team_id,
            workspace_id,
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
    workspace_id: Option<String>,
    result: Option<String>,
    note: Option<String>,
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
            workspace_id,
            result,
            note,
        )
        .await
}

fn apply_excel_table_styling(path_str: &str) {
    let ps_style = format!(
        r#"try {{
    Add-Type -AssemblyName System.IO.Compression;
    Add-Type -AssemblyName System.IO.Compression.FileSystem;
    $zip = [System.IO.Compression.ZipFile]::Open('{}', [System.IO.Compression.ZipArchiveMode]::Update);

    $stylesXml = '<?xml version="1.0" encoding="UTF-8" standalone="yes"?><styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><fonts count="2"><font><sz val="11"/><name val="Times New Roman"/><family val="2"/></font><font><b/><sz val="11"/><name val="Times New Roman"/><family val="2"/></font></fonts><fills count="3"><fill><patternFill patternType="none"/></fill><fill><patternFill patternType="gray125"/></fill><fill><patternFill patternType="solid"><fgColor rgb="FFF1F5F9"/><bgColor indexed="64"/></patternFill></fill></fills><borders count="2"><border><left/><right/><top/><bottom/><diagonal/></border><border><left style="thin"><color rgb="FF000000"/></left><right style="thin"><color rgb="FF000000"/></right><top style="thin"><color rgb="FF000000"/></top><bottom style="thin"><color rgb="FF000000"/></bottom><diagonal/></border></borders><cellStyleXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0"/></cellStyleXfs><cellXfs count="4"><xf numFmtId="0" fontId="0" fillId="0" borderId="0" xfId="0"/><xf numFmtId="0" fontId="1" fillId="2" borderId="1" xfId="0" applyFont="1" applyFill="1" applyBorder="1" applyAlignment="1"><alignment horizontal="center" vertical="center" wrapText="1"/></xf><xf numFmtId="0" fontId="0" fillId="0" borderId="1" xfId="0" applyFont="1" applyBorder="1" applyAlignment="1"><alignment horizontal="center" vertical="center" wrapText="1"/></xf><xf numFmtId="0" fontId="0" fillId="0" borderId="1" xfId="0" applyFont="1" applyBorder="1" applyAlignment="1"><alignment horizontal="left" vertical="center" wrapText="1"/></xf></cellXfs><cellStyles count="1"><cellStyle name="Normal" xfId="0" builtinId="0"/></cellStyles></styleSheet>';

    $entry = $zip.GetEntry('xl/styles.xml');
    if ($entry) {{ $entry.Delete(); }}
    $newEntry = $zip.CreateEntry('xl/styles.xml');
    $writer = New-Object System.IO.StreamWriter($newEntry.Open(), [System.Text.Encoding]::UTF8);
    $writer.Write($stylesXml);
    $writer.Dispose();

    $sheetEntry = $zip.GetEntry('xl/worksheets/sheet1.xml');
    if ($sheetEntry) {{
        $reader = New-Object System.IO.StreamReader($sheetEntry.Open(), [System.Text.Encoding]::UTF8);
        $sheetContent = $reader.ReadToEnd();
        $reader.Dispose();

        $sheetContent = [regex]::Replace($sheetContent, '<c r="([A-Z]+)([0-9]+)"', {{
            param($m)
            $col = $m.Groups[1].Value
            $row = $m.Groups[2].Value
            if ($row -eq "1") {{
                return "<c r=`"$col$row`" s=`"1`""
            }} elseif ($col -eq "A" -or $col -eq "D") {{
                return "<c r=`"$col$row`" s=`"2`""
            }} else {{
                return "<c r=`"$col$row`" s=`"3`""
            }}
        }});
        $sheetContent = [regex]::Replace($sheetContent, '<autoFilter[^>]*/>', '');
        $sheetContent = $sheetContent.Replace('<sheetView workbookViewId="0"/>', '<sheetView showGridLines="1" workbookViewId="0"/>');

        $sheetEntry.Delete();
        $newSheetEntry = $zip.CreateEntry('xl/worksheets/sheet1.xml');
        $sheetWriter = New-Object System.IO.StreamWriter($newSheetEntry.Open(), [System.Text.Encoding]::UTF8);
        $sheetWriter.Write($sheetContent);
        $sheetWriter.Dispose();
    }}

    $zip.Dispose();
}} catch {{}}"#,
        path_str.replace('\'', "''")
    );

    let _ = std::process::Command::new("powershell")
        .args(&["-NoProfile", "-WindowStyle", "Hidden", "-Command", &ps_style])
        .output();
}

#[tauri::command]
async fn save_excel_file_with_dialog(
    default_file_name: String,
    file_data: Vec<u8>,
) -> Result<Option<String>, String> {
    let ps_dialog = format!(
        r#"Add-Type -AssemblyName System.Windows.Forms; $d = New-Object System.Windows.Forms.SaveFileDialog; $d.Filter = 'Excel Workbook (*.xlsx)|*.xlsx|All Files (*.*)|*.*'; $d.FileName = '{}'; $d.InitialDirectory = [Environment]::GetFolderPath('UserProfile') + '\Downloads'; $d.Title = 'Chọn vị trí lưu tệp Báo cáo Excel'; if ($d.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {{ [Console]::Out.Write($d.FileName) }}"#,
        default_file_name.replace('\'', "''")
    );

    let output = std::process::Command::new("powershell")
        .args(&["-NoProfile", "-WindowStyle", "Hidden", "-Command", &ps_dialog])
        .output()
        .map_err(|e| format!("Không thể mở hộp thoại chọn tệp: {}", e))?;

    let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path_str.is_empty() {
        return Ok(None);
    }

    std::fs::write(&path_str, file_data)
        .map_err(|e| format!("Không thể ghi tệp Excel: {}", e))?;

    apply_excel_table_styling(&path_str);

    Ok(Some(path_str))
}

#[tauri::command]
async fn export_excel_file(file_name: String, file_data: Vec<u8>) -> Result<String, String> {
    let download_dir = if let Ok(userprofile) = std::env::var("USERPROFILE") {
        let p = std::path::PathBuf::from(userprofile).join("Downloads");
        if p.exists() {
            p
        } else {
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
        }
    } else if let Ok(home) = std::env::var("HOME") {
        let p = std::path::PathBuf::from(home).join("Downloads");
        if p.exists() {
            p
        } else {
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
        }
    } else {
        std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
    };

    let target_path = download_dir.join(&file_name);
    std::fs::write(&target_path, file_data)
        .map_err(|e| format!("Không thể ghi tệp Excel: {}", e))?;

    let path_str = target_path.to_string_lossy().to_string();
    apply_excel_table_styling(&path_str);

    Ok(path_str)
}

#[tauri::command]
async fn update_task_status(
    state: State<'_, DbState>,
    id: String,
    status: String,
    result: Option<String>,
) -> Result<bool, String> {
    state.update_task_status(&id, &status, result).await
}

#[tauri::command]
async fn delete_task(state: State<'_, DbState>, id: String) -> Result<bool, String> {
    state.delete_task(&id).await
}

#[tauri::command]
async fn get_dashboard_stats(
    state: State<'_, DbState>,
    team_id: Option<String>,
    workspace_id: Option<String>,
    for_user: Option<String>,
) -> Result<DashboardStats, String> {
    state.get_dashboard_stats(team_id, workspace_id, for_user).await
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
    team_id: Option<String>,
    workspace_id: Option<String>,
    task_id: Option<String>,
    file_name: String,
    file_data: Vec<u8>,
    content_type: String,
    uploaded_by: String,
    category: Option<String>,
) -> Result<TaskAttachment, String> {
    let tid_str = team_id.unwrap_or_else(|| "default_team".to_string());
    let wid_str = workspace_id.unwrap_or_else(|| "default_workspace".to_string());
    let task_id_str = task_id.clone().unwrap_or_else(|| "draft".to_string());

    let att = storage::upload_attachment(
        &tid_str,
        &wid_str,
        &task_id_str,
        &file_name,
        &file_data,
        &content_type,
        &uploaded_by,
        category,
    ).await?;

    if let Some(ref tid) = task_id {
        if !tid.trim().is_empty() && tid.trim() != "draft" {
            state.add_task_attachment(tid, att.clone()).await?;
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
            println!("[DB Success] Connected to MongoDB Atlas and initialized successfully!");
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
            get_teams,
            create_team,
            update_team,
            delete_team,
            get_workspaces,
            create_workspace,
            update_workspace,
            delete_workspace,
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
            export_excel_file,
            save_excel_file_with_dialog,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
