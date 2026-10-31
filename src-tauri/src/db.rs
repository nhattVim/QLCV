use std::sync::Arc;
use tokio::sync::RwLock;
use mongodb::{
    bson::{doc, oid::ObjectId, Bson, Document},
    options::ClientOptions,
    Client, Database,
};
use chrono::{DateTime, Duration, Utc};
use crate::models::{DashboardStats, TaskAttachment, TaskDto, TeamDto, UserSafe, WorkspaceDto};

#[allow(dead_code)]
mod embedded {
    include!(concat!(env!("OUT_DIR"), "/embedded_cfg.rs"));
}

fn xor_decode(data: &[u8], key: &[u8]) -> String {
    let bytes: Vec<u8> = data
        .iter()
        .enumerate()
        .map(|(i, b)| b ^ key[i % key.len()])
        .collect();
    String::from_utf8(bytes).unwrap_or_default()
}

fn resolve_mongo_uri() -> String {
    let _ = dotenvy::dotenv();
    std::env::var("MONGODB_URI")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| xor_decode(embedded::EMBED_URI, embedded::EMBED_KEY))
}

fn resolve_db_name() -> String {
    let _ = dotenvy::dotenv();
    std::env::var("DB_NAME")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            let name = xor_decode(embedded::EMBED_DB, embedded::EMBED_KEY);
            if name.is_empty() {
                "qlcv_db".to_string()
            } else {
                name
            }
        })
}

fn resolve_default_admin_email() -> String {
    let _ = dotenvy::dotenv();
    std::env::var("DEFAULT_ADMIN_EMAIL")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| xor_decode(embedded::EMBED_ADMIN_EMAIL, embedded::EMBED_KEY))
}

fn resolve_default_admin_password() -> String {
    let _ = dotenvy::dotenv();
    std::env::var("DEFAULT_ADMIN_PASSWORD")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| xor_decode(embedded::EMBED_ADMIN_PASSWORD, embedded::EMBED_KEY))
}

fn resolve_default_admin_name() -> String {
    let _ = dotenvy::dotenv();
    std::env::var("DEFAULT_ADMIN_NAME")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| xor_decode(embedded::EMBED_ADMIN_NAME, embedded::EMBED_KEY))
}

fn parse_due_date(due_str: &str) -> Option<DateTime<Utc>> {
    if let Ok(parsed_dt) = DateTime::parse_from_rfc3339(due_str) {
        Some(parsed_dt.with_timezone(&Utc))
    } else if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(due_str, "%Y-%m-%dT%H:%M") {
        Some(DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc))
    } else {
        None
    }
}

fn clamp_remind_days(days: Option<i32>) -> i32 {
    match days.unwrap_or(2) {
        1 => 1,
        3 => 3,
        _ => 2,
    }
}

fn doc_datetime(doc: &Document, key: &str) -> Option<DateTime<Utc>> {
    match doc.get(key) {
        Some(Bson::DateTime(dt)) => Some(dt.to_chrono()),
        Some(Bson::String(s)) => parse_due_date(s),
        _ => None,
    }
}

fn reminder_schedule_changed(
    old: &Document,
    new_days: i32,
    new_remind_at: Option<DateTime<Utc>>,
) -> bool {
    let old_days = old
        .get_i32("remind_before_days")
        .ok()
        .or_else(|| old.get_i64("remind_before_days").ok().map(|v| v as i32))
        .unwrap_or(2);
    if old_days != new_days {
        return true;
    }

    match (doc_datetime(old, "remind_at"), new_remind_at) {
        (Some(old_at), Some(new_at)) => (old_at - new_at).num_seconds().abs() > 60,
        (None, Some(_)) => true,
        (Some(_), None) => true,
        (None, None) => false,
    }
}

#[derive(Clone)]
pub struct DbState {
    pub db: Arc<RwLock<Option<Database>>>,
    pub current_uri: Arc<RwLock<String>>,
    pub is_initializing: Arc<RwLock<bool>>,
    pub init_error: Arc<RwLock<Option<String>>>,
}

impl DbState {
    pub fn new() -> Self {
        Self {
            db: Arc::new(RwLock::new(None)),
            current_uri: Arc::new(RwLock::new(String::new())),
            is_initializing: Arc::new(RwLock::new(true)),
            init_error: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn init(&self) -> Result<(), String> {
        *self.is_initializing.write().await = true;
        *self.init_error.write().await = None;

        let uri = resolve_mongo_uri();
        if uri.is_empty() {
            let err = "Thiếu cấu hình kết nối MongoDB.".to_string();
            *self.init_error.write().await = Some(err.clone());
            *self.is_initializing.write().await = false;
            return Err(err);
        }

        match self.connect(&uri).await {
            Ok(_) => {
                if let Err(e) = self.init_default_data().await {
                    eprintln!("[DB Warning] init_default_data error: {}", e);
                }
                *self.is_initializing.write().await = false;
                Ok(())
            }
            Err(e) => {
                *self.init_error.write().await = Some(e.clone());
                *self.is_initializing.write().await = false;
                Err(e)
            }
        }
    }

    pub async fn connect(&self, uri: &str) -> Result<(), String> {
        let mut client_options = ClientOptions::parse(uri)
            .await
            .map_err(|e| format!("Lỗi cấu hình MongoDB URI: {}", e))?;
        
        client_options.app_name = Some("QLCV_Tauri_Desktop".to_string());
        client_options.server_selection_timeout = Some(std::time::Duration::from_secs(6));

        let client = Client::with_options(client_options)
            .map_err(|e| format!("Không thể khởi tạo client: {}", e))?;

        // Ping test
        client
            .database("admin")
            .run_command(doc! { "ping": 1 })
            .await
            .map_err(|e| format!("Không thể kết nối đến MongoDB Atlas: {}", e))?;

        let db_name = resolve_db_name();
        let database = client.database(&db_name);

        // Save active DB
        let mut db_lock = self.db.write().await;
        *db_lock = Some(database);

        let mut uri_lock = self.current_uri.write().await;
        *uri_lock = uri.to_string();

        Ok(())
    }

    async fn get_db(&self) -> Result<Database, String> {
        let start = std::time::Instant::now();
        loop {
            {
                let db_lock = self.db.read().await;
                if let Some(ref db) = *db_lock {
                    return Ok(db.clone());
                }
            }

            let is_init = *self.is_initializing.read().await;
            let err_opt = self.init_error.read().await.clone();

            if !is_init && err_opt.is_some() {
                return Err(format!("Lỗi kết nối MongoDB Atlas: {}", err_opt.unwrap()));
            }

            if start.elapsed() > std::time::Duration::from_secs(15) {
                return Err("Quá thời gian chờ kết nối MongoDB Atlas (15s)!".to_string());
            }

            tokio::time::sleep(std::time::Duration::from_millis(80)).await;
        }
    }

    // ==========================================
    // AUTO-MIGRATION & DEFAULT DATA INITIALIZATION
    // ==========================================

    pub async fn init_default_data(&self) -> Result<(), String> {
        let db = self.get_db().await?;
        let users_col = db.collection::<Document>("users");
        let teams_col = db.collection::<Document>("teams");
        let workspaces_col = db.collection::<Document>("workspaces");
        let tasks_col = db.collection::<Document>("tasks");

        let now = Utc::now();

        // 1. Super Admin setup from .env
        let admin_email = resolve_default_admin_email().trim().to_lowercase();
        let admin_pass = resolve_default_admin_password();
        let admin_name = resolve_default_admin_name();

        if !admin_email.is_empty() {
            let existing_admin = users_col.find_one(doc! { "email": &admin_email }).await.map_err(|e| e.to_string())?;
            if let Some(doc) = existing_admin {
                // Ensure role is super_admin
                let current_role = doc.get_str("role").unwrap_or_default();
                if current_role != "super_admin" {
                    let _ = users_col.update_one(
                        doc! { "email": &admin_email },
                        doc! { "$set": { "role": "super_admin" } }
                    ).await;
                }
            } else {
                // Create super admin
                if let Ok(hashed) = bcrypt::hash(&admin_pass, bcrypt::DEFAULT_COST) {
                    let _ = users_col.insert_one(doc! {
                        "email": &admin_email,
                        "password_hash": hashed,
                        "full_name": &admin_name,
                        "role": "super_admin",
                        "is_active": true,
                        "created_at": mongodb::bson::DateTime::from_chrono(now)
                    }).await;
                }
            }
        }

        // 2. Default Team check/create
        let first_team = teams_col.find_one(doc! {}).await.map_err(|e| e.to_string())?;
        let default_team_id = match first_team {
            Some(t) => t.get_object_id("_id").ok().map(|oid| oid.to_hex()).unwrap_or_default(),
            None => {
                let res = teams_col.insert_one(doc! {
                    "name": "Đội ngũ chính",
                    "description": "Team mặc định cho toàn bộ thành viên hệ thống",
                    "enable_review_step": true,
                    "enable_result_column": true,
                    "created_at": mongodb::bson::DateTime::from_chrono(now),
                    "updated_at": mongodb::bson::DateTime::from_chrono(now)
                }).await.map_err(|e| e.to_string())?;
                res.inserted_id.as_object_id().map(|oid| oid.to_hex()).unwrap_or_default()
            }
        };

        // 3. Default Workspace check/create for default team
        if !default_team_id.is_empty() {
            let first_workspace = workspaces_col.find_one(doc! { "team_id": &default_team_id }).await.map_err(|e| e.to_string())?;
            let default_ws_id = match first_workspace {
                Some(ws) => ws.get_object_id("_id").ok().map(|oid| oid.to_hex()).unwrap_or_default(),
                None => {
                    let res = workspaces_col.insert_one(doc! {
                        "team_id": &default_team_id,
                        "name": "Không gian chung",
                        "description": "Không gian công việc chung mặc định của đội ngũ",
                        "created_by": &admin_email,
                        "created_at": mongodb::bson::DateTime::from_chrono(now),
                        "updated_at": mongodb::bson::DateTime::from_chrono(now)
                    }).await.map_err(|e| e.to_string())?;
                    res.inserted_id.as_object_id().map(|oid| oid.to_hex()).unwrap_or_default()
                }
            };

            // 4. Migrate users without team_id (except super_admin)
            let _ = users_col.update_many(
                doc! {
                    "role": { "$ne": "super_admin" },
                    "$or": [
                        { "team_id": { "$exists": false } },
                        { "team_id": Bson::Null },
                        { "team_id": "" }
                    ]
                },
                doc! { "$set": { "team_id": &default_team_id } }
            ).await;

            // 5. Migrate tasks without team_id
            let _ = tasks_col.update_many(
                doc! {
                    "$or": [
                        { "team_id": { "$exists": false } },
                        { "team_id": Bson::Null },
                        { "team_id": "" }
                    ]
                },
                doc! { "$set": { "team_id": &default_team_id } }
            ).await;

            // 6. Migrate tasks without workspace_id
            if !default_ws_id.is_empty() {
                let _ = tasks_col.update_many(
                    doc! {
                        "$or": [
                            { "workspace_id": { "$exists": false } },
                            { "workspace_id": Bson::Null },
                            { "workspace_id": "" }
                        ]
                    },
                    doc! { "$set": { "workspace_id": &default_ws_id } }
                ).await;
            }
        }

        Ok(())
    }

    // ==========================================
    // TEAMS CRUD
    // ==========================================

    pub async fn get_teams(&self) -> Result<Vec<TeamDto>, String> {
        let db = self.get_db().await?;
        let teams_col = db.collection::<Document>("teams");
        let users_col = db.collection::<Document>("users");
        let ws_col = db.collection::<Document>("workspaces");
        let tasks_col = db.collection::<Document>("tasks");

        let mut cursor = teams_col.find(doc! {})
            .await
            .map_err(|e| format!("Lỗi tải danh sách team: {}", e))?;

        let mut teams = Vec::new();
        while cursor.advance().await.map_err(|e| e.to_string())? {
            let doc = cursor.deserialize_current().map_err(|e| e.to_string())?;
            let id = doc.get_object_id("_id").ok().map(|oid| oid.to_hex());
            let name = doc.get_str("name").unwrap_or_default().to_string();
            let description = doc.get_str("description").unwrap_or_default().to_string();
            let enable_review_step = doc.get_bool("enable_review_step").unwrap_or(true);
            let enable_result_column = doc.get_bool("enable_result_column").unwrap_or(true);

            let created_at = match doc.get("created_at") {
                Some(Bson::DateTime(dt)) => Some(dt.to_chrono().to_rfc3339()),
                Some(Bson::String(s)) => Some(s.clone()),
                _ => None,
            };

            let updated_at = match doc.get("updated_at") {
                Some(Bson::DateTime(dt)) => Some(dt.to_chrono().to_rfc3339()),
                Some(Bson::String(s)) => Some(s.clone()),
                _ => None,
            };

            let mut member_count = 0;
            let mut workspace_count = 0;
            let mut task_count = 0;

            if let Some(ref tid) = id {
                member_count = users_col.count_documents(doc! { "team_id": tid }).await.unwrap_or(0) as usize;
                workspace_count = ws_col.count_documents(doc! { "team_id": tid }).await.unwrap_or(0) as usize;
                task_count = tasks_col.count_documents(doc! { "team_id": tid }).await.unwrap_or(0) as usize;
            }

            teams.push(TeamDto {
                id,
                name,
                description,
                created_at,
                updated_at,
                member_count,
                workspace_count,
                task_count,
                enable_review_step,
                enable_result_column,
            });
        }

        Ok(teams)
    }

    pub async fn create_team(
        &self,
        name: &str,
        description: &str,
        enable_review_step: Option<bool>,
        enable_result_column: Option<bool>,
    ) -> Result<TeamDto, String> {
        let db = self.get_db().await?;
        let teams_col = db.collection::<Document>("teams");
        let ws_col = db.collection::<Document>("workspaces");

        let now = Utc::now();
        let review_step = enable_review_step.unwrap_or(true);
        let result_column = enable_result_column.unwrap_or(true);
        let new_doc = doc! {
            "name": name.trim(),
            "description": description.trim(),
            "enable_review_step": review_step,
            "enable_result_column": result_column,
            "created_at": mongodb::bson::DateTime::from_chrono(now),
            "updated_at": mongodb::bson::DateTime::from_chrono(now),
        };

        let res = teams_col.insert_one(new_doc).await
            .map_err(|e| format!("Lỗi tạo team: {}", e))?;

        let id = res.inserted_id.as_object_id().map(|oid| oid.to_hex());

        // Create default workspace for new team
        if let Some(ref tid) = id {
            let _ = ws_col.insert_one(doc! {
                "team_id": tid,
                "name": "Không gian chung",
                "description": format!("Không gian công việc chung của {}", name.trim()),
                "created_at": mongodb::bson::DateTime::from_chrono(now),
                "updated_at": mongodb::bson::DateTime::from_chrono(now),
            }).await;
        }

        Ok(TeamDto {
            id,
            name: name.trim().to_string(),
            description: description.trim().to_string(),
            created_at: Some(now.to_rfc3339()),
            updated_at: Some(now.to_rfc3339()),
            member_count: 0,
            workspace_count: 1,
            task_count: 0,
            enable_review_step: review_step,
            enable_result_column: result_column,
        })
    }

    pub async fn update_team(
        &self,
        id: &str,
        name: &str,
        description: &str,
        enable_review_step: Option<bool>,
        enable_result_column: Option<bool>,
    ) -> Result<bool, String> {
        let db = self.get_db().await?;
        let teams_col = db.collection::<Document>("teams");

        let oid = ObjectId::parse_str(id).map_err(|_| "Mã Team không hợp lệ!".to_string())?;
        let now = Utc::now();

        let mut set_doc = doc! {
            "name": name.trim(),
            "description": description.trim(),
            "updated_at": mongodb::bson::DateTime::from_chrono(now),
        };
        if let Some(rev) = enable_review_step {
            set_doc.insert("enable_review_step", rev);
            if !rev {
                // Khi tắt tính năng duyệt: chuyển toàn bộ ticket đang ở tag "REVIEW" về "IN_PROGRESS"
                let tasks_col = db.collection::<Document>("tasks");
                let filter_review = doc! {
                    "team_id": id.trim(),
                    "status": "REVIEW"
                };
                let update_review = doc! {
                    "$set": {
                        "status": "IN_PROGRESS",
                        "updated_at": mongodb::bson::DateTime::from_chrono(now)
                    }
                };
                let _ = tasks_col.update_many(filter_review, update_review).await;
            }
        }
        if let Some(res) = enable_result_column {
            set_doc.insert("enable_result_column", res);
        }

        let update_doc = doc! {
            "$set": set_doc
        };

        teams_col.update_one(doc! { "_id": oid }, update_doc).await
            .map_err(|e| format!("Lỗi cập nhật team: {}", e))?;

        Ok(true)
    }

    pub async fn delete_team(&self, id: &str) -> Result<bool, String> {
        let db = self.get_db().await?;
        let teams_col = db.collection::<Document>("teams");
        let ws_col = db.collection::<Document>("workspaces");
        let tasks_col = db.collection::<Document>("tasks");
        let users_col = db.collection::<Document>("users");

        let oid = ObjectId::parse_str(id).map_err(|_| "Mã Team không hợp lệ!".to_string())?;

        // 1. Thu thập tất cả object_key của các công việc thuộc team này để xóa trên R2
        let mut object_keys = Vec::new();
        if let Ok(mut cursor) = tasks_col.find(doc! { "team_id": id }).await {
            while let Ok(true) = cursor.advance().await {
                if let Ok(task_doc) = cursor.deserialize_current() {
                    object_keys.extend(Self::extract_attachment_keys_from_doc(&task_doc));
                }
            }
        }

        // 2. Delete team
        teams_col.delete_one(doc! { "_id": oid }).await
            .map_err(|e| format!("Lỗi xóa team: {}", e))?;

        // 3. Delete all workspaces in team
        let _ = ws_col.delete_many(doc! { "team_id": id }).await;

        // 4. Delete all tasks in team
        let _ = tasks_col.delete_many(doc! { "team_id": id }).await;

        // 5. Clear team_id for users who were in this team
        let _ = users_col.update_many(
            doc! { "team_id": id },
            doc! { "$set": { "team_id": Bson::Null } }
        ).await;

        // 6. Xóa các tệp đính kèm trên Cloudflare R2
        if !object_keys.is_empty() {
            let _ = crate::storage::delete_attachments_batch(&object_keys).await;
        }

        Ok(true)
    }

    // ==========================================
    // WORKSPACES CRUD
    // ==========================================

    pub async fn get_workspaces(&self, team_id: &str) -> Result<Vec<WorkspaceDto>, String> {
        let db = self.get_db().await?;
        let ws_col = db.collection::<Document>("workspaces");
        let tasks_col = db.collection::<Document>("tasks");

        let filter = if team_id.trim().is_empty() {
            doc! {}
        } else {
            doc! { "team_id": team_id.trim() }
        };

        let mut cursor = ws_col.find(filter)
            .await
            .map_err(|e| format!("Lỗi tải không gian làm việc: {}", e))?;

        let mut workspaces = Vec::new();
        while cursor.advance().await.map_err(|e| e.to_string())? {
            let doc = cursor.deserialize_current().map_err(|e| e.to_string())?;
            let id = doc.get_object_id("_id").ok().map(|oid| oid.to_hex());
            let tid = doc.get_str("team_id").unwrap_or_default().to_string();
            let name = doc.get_str("name").unwrap_or_default().to_string();
            let description = doc.get_str("description").unwrap_or_default().to_string();
            let created_by = doc.get_str("created_by").ok().map(|s| s.to_string());

            let created_at = match doc.get("created_at") {
                Some(Bson::DateTime(dt)) => Some(dt.to_chrono().to_rfc3339()),
                Some(Bson::String(s)) => Some(s.clone()),
                _ => None,
            };

            let updated_at = match doc.get("updated_at") {
                Some(Bson::DateTime(dt)) => Some(dt.to_chrono().to_rfc3339()),
                Some(Bson::String(s)) => Some(s.clone()),
                _ => None,
            };

            let mut task_count = 0;
            if let Some(ref wid) = id {
                task_count = tasks_col.count_documents(doc! { "workspace_id": wid }).await.unwrap_or(0) as usize;
            }

            workspaces.push(WorkspaceDto {
                id,
                team_id: tid,
                name,
                description,
                created_by,
                created_at,
                updated_at,
                task_count,
            });
        }

        Ok(workspaces)
    }

    pub async fn create_workspace(
        &self,
        team_id: &str,
        name: &str,
        description: &str,
        created_by: &str,
    ) -> Result<WorkspaceDto, String> {
        let db = self.get_db().await?;
        let ws_col = db.collection::<Document>("workspaces");

        let now = Utc::now();
        let new_doc = doc! {
            "team_id": team_id.trim(),
            "name": name.trim(),
            "description": description.trim(),
            "created_by": created_by.trim(),
            "created_at": mongodb::bson::DateTime::from_chrono(now),
            "updated_at": mongodb::bson::DateTime::from_chrono(now),
        };

        let res = ws_col.insert_one(new_doc).await
            .map_err(|e| format!("Lỗi tạo không gian làm việc: {}", e))?;

        let id = res.inserted_id.as_object_id().map(|oid| oid.to_hex());

        Ok(WorkspaceDto {
            id,
            team_id: team_id.trim().to_string(),
            name: name.trim().to_string(),
            description: description.trim().to_string(),
            created_by: Some(created_by.trim().to_string()),
            created_at: Some(now.to_rfc3339()),
            updated_at: Some(now.to_rfc3339()),
            task_count: 0,
        })
    }

    pub async fn update_workspace(&self, id: &str, name: &str, description: &str) -> Result<bool, String> {
        let db = self.get_db().await?;
        let ws_col = db.collection::<Document>("workspaces");

        let oid = ObjectId::parse_str(id).map_err(|_| "Mã Không gian làm việc không hợp lệ!".to_string())?;
        let now = Utc::now();

        let update_doc = doc! {
            "$set": {
                "name": name.trim(),
                "description": description.trim(),
                "updated_at": mongodb::bson::DateTime::from_chrono(now),
            }
        };

        ws_col.update_one(doc! { "_id": oid }, update_doc).await
            .map_err(|e| format!("Lỗi cập nhật không gian làm việc: {}", e))?;

        Ok(true)
    }

    pub async fn delete_workspace(&self, id: &str) -> Result<bool, String> {
        let db = self.get_db().await?;
        let ws_col = db.collection::<Document>("workspaces");
        let tasks_col = db.collection::<Document>("tasks");

        let oid = ObjectId::parse_str(id).map_err(|_| "Mã Không gian làm việc không hợp lệ!".to_string())?;

        // 1. Thu thập tất cả object_key của các công việc thuộc workspace này để xóa trên R2
        let mut object_keys = Vec::new();
        if let Ok(mut cursor) = tasks_col.find(doc! { "workspace_id": id }).await {
            while let Ok(true) = cursor.advance().await {
                if let Ok(task_doc) = cursor.deserialize_current() {
                    object_keys.extend(Self::extract_attachment_keys_from_doc(&task_doc));
                }
            }
        }

        // 2. Delete workspace
        ws_col.delete_one(doc! { "_id": oid }).await
            .map_err(|e| format!("Lỗi xóa không gian làm việc: {}", e))?;

        // 3. Delete tasks in this workspace
        let _ = tasks_col.delete_many(doc! { "workspace_id": id }).await;

        // 4. Xóa các tệp đính kèm trên Cloudflare R2
        if !object_keys.is_empty() {
            let _ = crate::storage::delete_attachments_batch(&object_keys).await;
        }

        Ok(true)
    }

    // ==========================================
    // AUTH & USERS
    // ==========================================

    pub async fn authenticate(&self, email: &str, password: &str) -> Result<UserSafe, String> {
        let db = self.get_db().await?;
        let col = db.collection::<Document>("users");

        let filter = doc! { "email": email.trim().to_lowercase() };
        let user_doc = col.find_one(filter).await
            .map_err(|e| format!("Lỗi truy vấn cơ sở dữ liệu: {}", e))?
            .ok_or_else(|| "Email hoặc mật khẩu không chính xác!".to_string())?;

        let is_active = user_doc.get_bool("is_active").unwrap_or(true);
        if !is_active {
            return Err("Tài khoản này đã bị khóa. Vui lòng liên hệ Admin!".to_string());
        }

        let hash = user_doc.get_str("password_hash").unwrap_or_default();
        let valid = bcrypt::verify(password, hash).unwrap_or(false);
        if !valid {
            return Err("Email hoặc mật khẩu không chính xác!".to_string());
        }

        let id = user_doc.get_object_id("_id").ok().map(|oid| oid.to_hex());
        let full_name = user_doc.get_str("full_name").unwrap_or("Cán bộ").to_string();
        let role = user_doc.get_str("role").unwrap_or("user").to_string();
        let email = user_doc.get_str("email").unwrap_or("").to_string();
        let team_id = user_doc.get_str("team_id").ok().map(|s| s.to_string());
        let job_title = user_doc.get_str("job_title").ok().map(|s| s.to_string());

        let created_at = match user_doc.get("created_at") {
            Some(Bson::DateTime(dt)) => Some(dt.to_chrono().to_rfc3339()),
            Some(Bson::String(s)) => Some(s.clone()),
            _ => None,
        };

        Ok(UserSafe {
            id,
            email,
            full_name,
            role,
            is_active,
            team_id,
            created_at,
            job_title,
        })
    }

    pub async fn get_all_users(&self, team_id: Option<String>) -> Result<Vec<UserSafe>, String> {
        let db = self.get_db().await?;
        let col = db.collection::<Document>("users");

        let filter = match team_id {
            Some(tid) if !tid.trim().is_empty() => {
                doc! {
                    "$or": [
                        { "team_id": tid.trim() },
                        { "role": "super_admin" }
                    ]
                }
            }
            _ => doc! {}
        };

        let mut cursor = col.find(filter)
            .await
            .map_err(|e| format!("Lỗi tải danh sách người dùng: {}", e))?;

        let mut users = Vec::new();
        while cursor.advance().await.map_err(|e| e.to_string())? {
            let doc = cursor.deserialize_current().map_err(|e| e.to_string())?;
            let id = doc.get_object_id("_id").ok().map(|oid| oid.to_hex());
            let email = doc.get_str("email").unwrap_or_default().to_string();
            let full_name = doc.get_str("full_name").unwrap_or_default().to_string();
            let role = doc.get_str("role").unwrap_or("user").to_string();
            let is_active = doc.get_bool("is_active").unwrap_or(true);
            let team_id = doc.get_str("team_id").ok().map(|s| s.to_string());
            let job_title = doc.get_str("job_title").ok().map(|s| s.to_string());
            let created_at = match doc.get("created_at") {
                Some(Bson::DateTime(dt)) => Some(dt.to_chrono().to_rfc3339()),
                Some(Bson::String(s)) => Some(s.clone()),
                _ => None,
            };

            users.push(UserSafe {
                id,
                email,
                full_name,
                role,
                is_active,
                team_id,
                created_at,
                job_title,
            });
        }

        Ok(users)
    }

    pub async fn create_user(
        &self,
        full_name: &str,
        email: &str,
        password: &str,
        role: &str,
        team_id: Option<String>,
        job_title: Option<String>,
    ) -> Result<UserSafe, String> {
        if role == "super_admin" {
            return Err("Không được phép cấp vai trò Super Admin!".to_string());
        }

        let db = self.get_db().await?;
        let col = db.collection::<Document>("users");

        let email = email.trim().to_lowercase();
        if col.find_one(doc! { "email": &email }).await.map_err(|e| e.to_string())?.is_some() {
            return Err("Email này đã được sử dụng bởi cán bộ khác!".to_string());
        }

        let hashed = bcrypt::hash(password, bcrypt::DEFAULT_COST)
            .map_err(|e| format!("Lỗi mã hóa mật khẩu: {}", e))?;

        let now = Utc::now();
        let mut new_doc = doc! {
            "email": &email,
            "password_hash": hashed,
            "full_name": full_name.trim(),
            "role": role,
            "is_active": true,
            "created_at": mongodb::bson::DateTime::from_chrono(now)
        };

        if let Some(ref tid) = team_id {
            if !tid.trim().is_empty() {
                new_doc.insert("team_id", tid.trim());
            }
        }

        if let Some(ref jt) = job_title {
            if !jt.trim().is_empty() {
                new_doc.insert("job_title", jt.trim());
            }
        }

        let insert_res = col.insert_one(new_doc).await
            .map_err(|e| format!("Không thể tạo cán bộ: {}", e))?;

        let id = insert_res.inserted_id.as_object_id().map(|oid| oid.to_hex());

        Ok(UserSafe {
            id,
            email,
            full_name: full_name.trim().to_string(),
            role: role.to_string(),
            is_active: true,
            team_id,
            created_at: Some(now.to_rfc3339()),
            job_title,
        })
    }

    pub async fn update_user(
        &self,
        email: &str,
        full_name: Option<String>,
        role: Option<String>,
        is_active: Option<bool>,
        password: Option<String>,
        team_id: Option<String>,
        job_title: Option<String>,
    ) -> Result<bool, String> {
        let db = self.get_db().await?;
        let col = db.collection::<Document>("users");

        let mut update_doc = doc! {};
        if let Some(name) = full_name {
            update_doc.insert("full_name", name.trim());
        }
        if let Some(r) = role {
            if r == "super_admin" {
                let existing = col.find_one(doc! { "email": email.trim().to_lowercase() }).await.map_err(|e| e.to_string())?;
                let existing_role = existing.as_ref().and_then(|d| d.get_str("role").ok()).unwrap_or("");
                if existing_role != "super_admin" {
                    return Err("Không được phép nâng quyền lên Super Admin!".to_string());
                }
            }
            update_doc.insert("role", r);
        }
        if let Some(active) = is_active {
            update_doc.insert("is_active", active);
        }
        if let Some(tid) = team_id {
            if tid.trim().is_empty() {
                update_doc.insert("team_id", Bson::Null);
            } else {
                update_doc.insert("team_id", tid.trim());
            }
        }
        if let Some(jt) = job_title {
            update_doc.insert("job_title", jt.trim());
        }
        if let Some(pwd) = password {
            if !pwd.trim().is_empty() {
                let hashed = bcrypt::hash(&pwd, bcrypt::DEFAULT_COST)
                    .map_err(|e| format!("Lỗi mã hóa mật khẩu: {}", e))?;
                update_doc.insert("password_hash", hashed);
            }
        }

        if update_doc.is_empty() {
            return Ok(true);
        }

        let filter = doc! { "email": email.trim().to_lowercase() };
        col.update_one(filter, doc! { "$set": update_doc }).await
            .map_err(|e| format!("Lỗi cập nhật cán bộ: {}", e))?;

        Ok(true)
    }

    pub async fn delete_user(&self, email: &str) -> Result<bool, String> {
        let db = self.get_db().await?;
        let normalized_email = email.trim().to_lowercase();

        // 1. Delete user from users collection
        let col = db.collection::<Document>("users");
        let filter = doc! { "email": &normalized_email };
        let res = col.delete_one(filter).await
            .map_err(|e| format!("Lỗi xóa cán bộ: {}", e))?;

        // 2. Remove user from assigned_to array in all tasks
        let tasks_col = db.collection::<Document>("tasks");
        let task_filter = doc! { "assigned_to": &normalized_email };
        let task_update = doc! {
            "$pull": { "assigned_to": &normalized_email }
        };
        let _ = tasks_col.update_many(task_filter, task_update).await
            .map_err(|e| format!("Lỗi gỡ cán bộ khỏi các công việc: {}", e))?;

        Ok(res.deleted_count > 0)
    }

    // ==========================================
    // TASKS (TICKETS)
    // ==========================================

    pub async fn get_tasks(
        &self,
        team_id: Option<String>,
        workspace_id: Option<String>,
        status: Option<String>,
        priority: Option<String>,
        user_email: Option<String>,
    ) -> Result<Vec<TaskDto>, String> {
        let db = self.get_db().await?;
        let col = db.collection::<Document>("tasks");

        let mut filter = doc! {};
        if let Some(tid) = team_id {
            if !tid.trim().is_empty() {
                filter.insert("team_id", tid.trim());
            }
        }
        if let Some(wid) = workspace_id {
            if !wid.trim().is_empty() {
                filter.insert("workspace_id", wid.trim());
            }
        }
        if let Some(st) = status {
            if !st.is_empty() && st != "ALL" {
                filter.insert("status", st);
            }
        }
        if let Some(pr) = priority {
            if !pr.is_empty() && pr != "ALL" {
                filter.insert("priority", pr);
            }
        }
        if let Some(em) = user_email {
            if !em.is_empty() {
                filter.insert("assigned_to", em);
            }
        }

        let mut cursor = col.find(filter)
            .await
            .map_err(|e| format!("Lỗi tải danh sách công việc: {}", e))?;

        let mut tasks = Vec::new();
        while cursor.advance().await.map_err(|e| e.to_string())? {
            let doc = cursor.deserialize_current().map_err(|e| e.to_string())?;
            tasks.push(Self::doc_to_task(doc));
        }

        // Sắp xếp: Quá hạn & sắp đến hạn lên trước
        tasks.sort_by(|a, b| {
            let a_due = a.due_date.as_deref().unwrap_or("9999");
            let b_due = b.due_date.as_deref().unwrap_or("9999");
            a_due.cmp(b_due)
        });

        Ok(tasks)
    }

    pub async fn create_task(
        &self,
        team_id: Option<String>,
        workspace_id: Option<String>,
        title: &str,
        description: &str,
        assigned_to: Vec<String>,
        priority: &str,
        due_date: Option<String>,
        created_by: &str,
        remind_before_days: Option<i32>,
        attachments: Option<Vec<TaskAttachment>>,
    ) -> Result<TaskDto, String> {
        let db = self.get_db().await?;
        let col = db.collection::<Document>("tasks");

        let now = Utc::now();
        let days = clamp_remind_days(remind_before_days);
        let atts_bson = match attachments {
            Some(atts) => Bson::Array(atts.into_iter().map(Self::attachment_to_doc).map(Bson::Document).collect()),
            None => Bson::Array(Vec::new()),
        };

        let mut new_doc = doc! {
            "title": title.trim(),
            "description": description,
            "assigned_to": assigned_to,
            "priority": priority,
            "status": "TODO",
            "created_by": created_by,
            "created_at": mongodb::bson::DateTime::from_chrono(now),
            "updated_at": mongodb::bson::DateTime::from_chrono(now),
            "remind_before_days": days,
            "attachments": atts_bson,
        };

        if let Some(ref tid) = team_id {
            if !tid.trim().is_empty() {
                new_doc.insert("team_id", tid.trim());
            }
        }

        if let Some(ref wid) = workspace_id {
            if !wid.trim().is_empty() {
                new_doc.insert("workspace_id", wid.trim());
            }
        }

        if let Some(due_str) = due_date {
            if let Some(due_utc) = parse_due_date(&due_str) {
                new_doc.insert("due_date", mongodb::bson::DateTime::from_chrono(due_utc));
                let remind_at = due_utc - Duration::days(days as i64);
                new_doc.insert("remind_at", mongodb::bson::DateTime::from_chrono(remind_at));
            }
        }

        let res = col.insert_one(new_doc.clone()).await
            .map_err(|e| format!("Không thể tạo công việc: {}", e))?;

        new_doc.insert("_id", res.inserted_id);
        Ok(Self::doc_to_task(new_doc))
    }

    pub async fn update_task(
        &self,
        id: &str,
        title: &str,
        description: &str,
        assigned_to: Vec<String>,
        priority: &str,
        due_date: Option<String>,
        status: &str,
        remind_before_days: Option<i32>,
        attachments: Option<Vec<TaskAttachment>>,
        workspace_id: Option<String>,
        result: Option<String>,
        note: Option<String>,
    ) -> Result<bool, String> {
        let db = self.get_db().await?;
        let col = db.collection::<Document>("tasks");

        let oid = ObjectId::parse_str(id)
            .map_err(|_| "Mã công việc không hợp lệ!".to_string())?;

        let old_doc = col
            .find_one(doc! { "_id": oid })
            .await
            .map_err(|e| format!("Lỗi truy vấn công việc cũ: {}", e))?
            .ok_or_else(|| "Không tìm thấy công việc!".to_string())?;

        let now = Utc::now();
        let days = clamp_remind_days(remind_before_days);

        let parsed_due_utc = due_date.as_deref().and_then(parse_due_date);
        let new_remind_at = parsed_due_utc.map(|due| due - Duration::days(days as i64));
        let schedule_changed = reminder_schedule_changed(&old_doc, days, new_remind_at);

        let mut set_doc = doc! {
            "title": title.trim(),
            "description": description,
            "assigned_to": assigned_to,
            "priority": priority,
            "status": status,
            "updated_at": mongodb::bson::DateTime::from_chrono(now),
            "remind_before_days": days,
        };
        let mut unset_doc = doc! {};

        if let Some(ref res) = result {
            set_doc.insert("result", res.trim());
        }
        if let Some(ref n) = note {
            set_doc.insert("note", n.trim());
        }
        let old_status = old_doc.get_str("status").unwrap_or("TODO");
        if status == "DONE" {
            if old_status != "DONE" || !old_doc.contains_key("completed_at") {
                set_doc.insert("completed_at", mongodb::bson::DateTime::from_chrono(now));
            }
        } else {
            unset_doc.insert("completed_at", "");
        }

        if let Some(ref wid) = workspace_id {
            if !wid.trim().is_empty() {
                set_doc.insert("workspace_id", wid.trim());
            }
        }

        if let Some(atts) = attachments {
            let arr = Bson::Array(atts.into_iter().map(Self::attachment_to_doc).map(Bson::Document).collect());
            set_doc.insert("attachments", arr);
        }

        if let Some(due_utc) = parsed_due_utc {
            set_doc.insert("due_date", mongodb::bson::DateTime::from_chrono(due_utc));
            if let Some(remind_at) = new_remind_at {
                set_doc.insert("remind_at", mongodb::bson::DateTime::from_chrono(remind_at));
            }
            if schedule_changed {
                unset_doc.insert("reminded_at", "");
            }
        } else {
            unset_doc.insert("due_date", "");
            unset_doc.insert("remind_at", "");
            unset_doc.insert("reminded_at", "");
        }

        let mut update = doc! { "$set": set_doc };
        if !unset_doc.is_empty() {
            update.insert("$unset", unset_doc);
        }

        col.update_one(doc! { "_id": oid }, update).await
            .map_err(|e| format!("Lỗi cập nhật công việc: {}", e))?;

        Ok(true)
    }

    pub async fn update_task_status(&self, id: &str, status: &str, result: Option<String>) -> Result<bool, String> {
        let db = self.get_db().await?;
        let col = db.collection::<Document>("tasks");

        let oid = ObjectId::parse_str(id)
            .map_err(|_| "Mã công việc không hợp lệ!".to_string())?;

        let old_doc = col.find_one(doc! { "_id": oid }).await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Công việc không tồn tại!".to_string())?;

        let old_status = old_doc.get_str("status").unwrap_or("TODO");
        let now = Utc::now();
        let mut set_doc = doc! {
            "status": status,
            "updated_at": mongodb::bson::DateTime::from_chrono(now)
        };
        let mut unset_doc = doc! {};

        if let Some(ref res) = result {
            set_doc.insert("result", res.trim());
        }
        if status == "DONE" {
            if old_status != "DONE" || !old_doc.contains_key("completed_at") {
                set_doc.insert("completed_at", mongodb::bson::DateTime::from_chrono(now));
            }
        } else {
            unset_doc.insert("completed_at", "");
        }

        let mut update_doc = doc! {
            "$set": set_doc
        };
        if !unset_doc.is_empty() {
            update_doc.insert("$unset", unset_doc);
        }

        col.update_one(doc! { "_id": oid }, update_doc).await
            .map_err(|e| format!("Lỗi cập nhật trạng thái: {}", e))?;

        Ok(true)
    }

    pub async fn delete_task(&self, id: &str) -> Result<bool, String> {
        let db = self.get_db().await?;
        let col = db.collection::<Document>("tasks");

        let oid = ObjectId::parse_str(id)
            .map_err(|_| "Mã công việc không hợp lệ!".to_string())?;

        // 1. Thu thập object_keys trước khi xóa
        let mut object_keys = Vec::new();
        if let Ok(Some(task_doc)) = col.find_one(doc! { "_id": oid }).await {
            object_keys.extend(Self::extract_attachment_keys_from_doc(&task_doc));
        }

        // 2. Xóa công việc trong MongoDB
        col.delete_one(doc! { "_id": oid }).await
            .map_err(|e| format!("Lỗi xóa công việc: {}", e))?;

        // 3. Xóa các tệp đính kèm trên Cloudflare R2
        if !object_keys.is_empty() {
            let _ = crate::storage::delete_attachments_batch(&object_keys).await;
        }

        Ok(true)
    }

    // Attachment Helpers
    pub async fn add_task_attachment(&self, task_id: &str, attachment: TaskAttachment) -> Result<(), String> {
        let db = self.get_db().await?;
        let col = db.collection::<Document>("tasks");

        let oid = ObjectId::parse_str(task_id).map_err(|_| "Mã công việc không hợp lệ".to_string())?;
        let att_doc = Self::attachment_to_doc(attachment);

        col.update_one(
            doc! { "_id": oid },
            doc! {
                "$push": { "attachments": att_doc },
                "$set": { "updated_at": mongodb::bson::DateTime::from_chrono(Utc::now()) }
            }
        ).await.map_err(|e| format!("Lỗi lưu tệp đính kèm vào công việc: {}", e))?;

        Ok(())
    }

    pub async fn remove_task_attachment(&self, task_id: &str, attachment_id: &str) -> Result<(), String> {
        let db = self.get_db().await?;
        let col = db.collection::<Document>("tasks");

        let oid = ObjectId::parse_str(task_id).map_err(|_| "Mã công việc không hợp lệ".to_string())?;

        col.update_one(
            doc! { "_id": oid },
            doc! {
                "$pull": { "attachments": { "id": attachment_id } },
                "$set": { "updated_at": mongodb::bson::DateTime::from_chrono(Utc::now()) }
            }
        ).await.map_err(|e| format!("Lỗi xóa tệp đính kèm khỏi công việc: {}", e))?;

        Ok(())
    }

    // ==========================================
    // DASHBOARD & STATS
    // ==========================================

    pub async fn get_dashboard_stats(
        &self,
        team_id: Option<String>,
        workspace_id: Option<String>,
        for_user: Option<String>,
    ) -> Result<DashboardStats, String> {
        let db = self.get_db().await?;
        let col = db.collection::<Document>("tasks");

        let mut filter = doc! {};
        if let Some(tid) = team_id {
            if !tid.trim().is_empty() {
                filter.insert("team_id", tid.trim());
            }
        }
        if let Some(wid) = workspace_id {
            if !wid.trim().is_empty() {
                filter.insert("workspace_id", wid.trim());
            }
        }
        if let Some(user_email) = for_user {
            if !user_email.is_empty() {
                filter.insert("assigned_to", user_email);
            }
        }

        let mut cursor = col.find(filter)
            .await
            .map_err(|e| format!("Lỗi thống kê: {}", e))?;

        let mut stats = DashboardStats::default();
        let now = Utc::now();

        while cursor.advance().await.map_err(|e| e.to_string())? {
            let doc = cursor.deserialize_current().map_err(|e| e.to_string())?;
            stats.total += 1;

            let st = doc.get_str("status").unwrap_or("TODO");
            match st {
                "TODO" => stats.todo += 1,
                "IN_PROGRESS" => stats.in_progress += 1,
                "REVIEW" => stats.review += 1,
                "DONE" => stats.done += 1,
                _ => {}
            }

            if st != "DONE" {
                if let Some(due_dt) = doc_datetime(&doc, "due_date") {
                    if due_dt < now {
                        stats.overdue += 1;
                    } else {
                        let diff_hours = (due_dt - now).num_hours();
                        let prio = doc.get_str("priority").unwrap_or("Low");
                        if prio == "Urgent" || diff_hours <= 48 {
                            stats.urgent_or_near += 1;
                        }
                    }
                }
            }
        }

        Ok(stats)
    }

    pub fn extract_attachment_keys_from_doc(doc: &Document) -> Vec<String> {
        let mut keys = Vec::new();
        if let Some(Bson::Array(arr)) = doc.get("attachments") {
            for b in arr {
                if let Bson::Document(att_doc) = b {
                    if let Ok(key) = att_doc.get_str("object_key") {
                        let trimmed = key.trim();
                        if !trimmed.is_empty() {
                            keys.push(trimmed.to_string());
                        }
                    }
                }
            }
        }
        keys
    }

    pub fn attachment_to_doc(att: TaskAttachment) -> Document {
        doc! {
            "id": att.id,
            "file_name": att.file_name,
            "file_size": att.file_size as i64,
            "content_type": att.content_type,
            "object_key": att.object_key,
            "uploaded_by": att.uploaded_by,
            "uploaded_at": att.uploaded_at,
            "download_url": att.download_url.unwrap_or_default(),
            "category": att.category.unwrap_or_else(|| "document".to_string()),
        }
    }

    // Helper: Convert Document to TaskDto
    fn doc_to_task(doc: Document) -> TaskDto {
        let id = doc.get_object_id("_id").ok().map(|oid| oid.to_hex());
        let team_id = doc.get_str("team_id").ok().map(|s| s.to_string());
        let workspace_id = doc.get_str("workspace_id").ok().map(|s| s.to_string());
        let title = doc.get_str("title").unwrap_or_default().to_string();
        let description = doc.get_str("description").unwrap_or_default().to_string();

        let assigned_to = match doc.get("assigned_to") {
            Some(Bson::Array(arr)) => arr.iter().filter_map(|b| b.as_str().map(|s| s.to_string())).collect(),
            _ => Vec::new(),
        };

        let priority = doc.get_str("priority").unwrap_or("Medium").to_string();
        let status = doc.get_str("status").unwrap_or("TODO").to_string();

        let due_date = match doc.get("due_date") {
            Some(Bson::DateTime(dt)) => Some(dt.to_chrono().to_rfc3339()),
            Some(Bson::String(s)) => Some(s.clone()),
            _ => None,
        };

        let created_by = doc.get_str("created_by").unwrap_or("Admin").to_string();

        let created_at = match doc.get("created_at") {
            Some(Bson::DateTime(dt)) => Some(dt.to_chrono().to_rfc3339()),
            Some(Bson::String(s)) => Some(s.clone()),
            _ => None,
        };

        let updated_at = match doc.get("updated_at") {
            Some(Bson::DateTime(dt)) => Some(dt.to_chrono().to_rfc3339()),
            Some(Bson::String(s)) => Some(s.clone()),
            _ => None,
        };

        let remind_before_days = doc
            .get_i32("remind_before_days")
            .ok()
            .or_else(|| doc.get_i64("remind_before_days").ok().map(|v| v as i32));

        let remind_at = match doc.get("remind_at") {
            Some(Bson::DateTime(dt)) => Some(dt.to_chrono().to_rfc3339()),
            Some(Bson::String(s)) => Some(s.clone()),
            _ => None,
        };

        let reminded_at = match doc.get("reminded_at") {
            Some(Bson::DateTime(dt)) => Some(dt.to_chrono().to_rfc3339()),
            Some(Bson::String(s)) => Some(s.clone()),
            _ => None,
        };

        let attachments = match doc.get("attachments") {
            Some(Bson::Array(arr)) => arr.iter().filter_map(|b| {
                if let Bson::Document(d) = b {
                    let dl = d.get_str("download_url").ok().filter(|s| !s.is_empty()).map(|s| s.to_string());
                    let category = d.get_str("category").ok().map(|s| s.to_string());
                    Some(TaskAttachment {
                        id: d.get_str("id").unwrap_or_default().to_string(),
                        file_name: d.get_str("file_name").unwrap_or_default().to_string(),
                        file_size: d.get_i64("file_size").ok().map(|v| v as u64).unwrap_or(0),
                        content_type: d.get_str("content_type").unwrap_or("application/octet-stream").to_string(),
                        object_key: d.get_str("object_key").unwrap_or_default().to_string(),
                        uploaded_by: d.get_str("uploaded_by").unwrap_or_default().to_string(),
                        uploaded_at: match d.get("uploaded_at") {
                            Some(Bson::DateTime(dt)) => dt.to_chrono().to_rfc3339(),
                            Some(Bson::String(s)) => s.clone(),
                            _ => String::new(),
                        },
                        download_url: dl,
                        category,
                    })
                } else {
                    None
                }
            }).collect(),
            _ => Vec::new(),
        };

        let result = doc.get_str("result").ok().map(|s| s.to_string());
        let completed_at = match doc.get("completed_at") {
            Some(Bson::DateTime(dt)) => Some(dt.to_chrono().to_rfc3339()),
            Some(Bson::String(s)) => Some(s.clone()),
            _ => None,
        };
        let note = doc.get_str("note").ok().map(|s| s.to_string());

        TaskDto {
            id,
            team_id,
            workspace_id,
            title,
            description,
            assigned_to,
            priority,
            status,
            due_date,
            created_by,
            created_at,
            updated_at,
            remind_before_days,
            remind_at,
            reminded_at,
            attachments,
            result,
            completed_at,
            note,
        }
    }
}
