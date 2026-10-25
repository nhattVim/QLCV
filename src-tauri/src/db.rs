use std::sync::Arc;
use tokio::sync::RwLock;
use mongodb::{
    bson::{doc, oid::ObjectId, Bson, Document},
    options::ClientOptions,
    Client, Database,
};
use chrono::{DateTime, Duration, Utc};
use crate::models::{DashboardStats, TaskDto, UserSafe};

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
}

impl DbState {
    pub fn new() -> Self {
        Self {
            db: Arc::new(RwLock::new(None)),
            current_uri: Arc::new(RwLock::new(String::new())),
        }
    }

    pub async fn init(&self) -> Result<(), String> {
        let uri = resolve_mongo_uri();
        if uri.is_empty() {
            return Err("Thiếu cấu hình kết nối MongoDB.".to_string());
        }
        self.connect(&uri).await
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
        let db_lock = self.db.read().await;
        match &*db_lock {
            Some(db) => Ok(db.clone()),
            None => Err("Chưa kết nối đến cơ sở dữ liệu MongoDB Atlas!".to_string()),
        }
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
            created_at,
        })
    }

    pub async fn get_all_users(&self) -> Result<Vec<UserSafe>, String> {
        let db = self.get_db().await?;
        let col = db.collection::<Document>("users");

        let mut cursor = col.find(doc! {})
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
                created_at,
            });
        }

        Ok(users)
    }

    pub async fn create_user(&self, full_name: &str, email: &str, password: &str, role: &str) -> Result<UserSafe, String> {
        let db = self.get_db().await?;
        let col = db.collection::<Document>("users");

        let email = email.trim().to_lowercase();
        if col.find_one(doc! { "email": &email }).await.map_err(|e| e.to_string())?.is_some() {
            return Err("Email này đã được sử dụng bởi cán bộ khác!".to_string());
        }

        let hashed = bcrypt::hash(password, bcrypt::DEFAULT_COST)
            .map_err(|e| format!("Lỗi mã hóa mật khẩu: {}", e))?;

        let now = Utc::now();
        let new_doc = doc! {
            "email": &email,
            "password_hash": hashed,
            "full_name": full_name.trim(),
            "role": role,
            "is_active": true,
            "created_at": mongodb::bson::DateTime::from_chrono(now)
        };

        let insert_res = col.insert_one(new_doc).await
            .map_err(|e| format!("Không thể tạo cán bộ: {}", e))?;

        let id = insert_res.inserted_id.as_object_id().map(|oid| oid.to_hex());

        Ok(UserSafe {
            id,
            email,
            full_name: full_name.trim().to_string(),
            role: role.to_string(),
            is_active: true,
            created_at: Some(now.to_rfc3339()),
        })
    }

    pub async fn update_user(
        &self,
        email: &str,
        full_name: Option<String>,
        role: Option<String>,
        is_active: Option<bool>,
        password: Option<String>,
    ) -> Result<bool, String> {
        let db = self.get_db().await?;
        let col = db.collection::<Document>("users");

        let mut update_doc = doc! {};
        if let Some(name) = full_name {
            update_doc.insert("full_name", name.trim());
        }
        if let Some(r) = role {
            update_doc.insert("role", r);
        }
        if let Some(active) = is_active {
            update_doc.insert("is_active", active);
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
        let col = db.collection::<Document>("users");

        let filter = doc! { "email": email.trim().to_lowercase() };
        let res = col.delete_one(filter).await
            .map_err(|e| format!("Lỗi xóa cán bộ: {}", e))?;

        Ok(res.deleted_count > 0)
    }

    // ==========================================
    // TASKS
    // ==========================================

    pub async fn get_tasks(
        &self,
        status: Option<String>,
        priority: Option<String>,
        user_email: Option<String>,
    ) -> Result<Vec<TaskDto>, String> {
        let db = self.get_db().await?;
        let col = db.collection::<Document>("tasks");

        let mut filter = doc! {};
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
        title: &str,
        description: &str,
        assigned_to: Vec<String>,
        priority: &str,
        due_date: Option<String>,
        created_by: &str,
        remind_before_days: Option<i32>,
    ) -> Result<TaskDto, String> {
        let db = self.get_db().await?;
        let col = db.collection::<Document>("tasks");

        let now = Utc::now();
        let days = clamp_remind_days(remind_before_days);
        let mut new_doc = doc! {
            "title": title.trim(),
            "description": description,
            "assigned_to": assigned_to,
            "priority": priority,
            "status": "TODO",
            "created_by": created_by,
            "created_at": mongodb::bson::DateTime::from_chrono(now),
            "updated_at": mongodb::bson::DateTime::from_chrono(now),
            "remind_before_days": days
        };

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
    ) -> Result<bool, String> {
        let db = self.get_db().await?;
        let col = db.collection::<Document>("tasks");

        let oid = ObjectId::parse_str(id)
            .map_err(|_| "Mã công việc không hợp lệ!".to_string())?;

        let existing = col
            .find_one(doc! { "_id": oid })
            .await
            .map_err(|e| format!("Lỗi tải công việc: {}", e))?
            .ok_or_else(|| "Không tìm thấy công việc!".to_string())?;

        let now = Utc::now();
        let days = clamp_remind_days(remind_before_days);
        let mut new_remind_at: Option<DateTime<Utc>> = None;
        let mut update_doc = doc! {
            "title": title.trim(),
            "description": description,
            "assigned_to": assigned_to,
            "priority": priority,
            "status": status,
            "updated_at": mongodb::bson::DateTime::from_chrono(now),
            "remind_before_days": days
        };

        if let Some(due_str) = due_date {
            if let Some(due_utc) = parse_due_date(&due_str) {
                update_doc.insert("due_date", mongodb::bson::DateTime::from_chrono(due_utc));
                let remind_at = due_utc - Duration::days(days as i64);
                new_remind_at = Some(remind_at);
                update_doc.insert("remind_at", mongodb::bson::DateTime::from_chrono(remind_at));
            }
        }

        let reminder_changed = reminder_schedule_changed(&existing, days, new_remind_at);
        if reminder_changed {
            update_doc.insert("reminded_at", Bson::Null);
        }

        let mut update_cmd = doc! { "$set": update_doc };
        if reminder_changed {
            update_cmd.insert(
                "$unset",
                doc! {
                    "reminded_48h": "",
                    "last_reminded_at": ""
                },
            );
        }

        let res = col
            .update_one(doc! { "_id": oid }, update_cmd)
            .await
            .map_err(|e| format!("Lỗi cập nhật công việc: {}", e))?;

        Ok(res.matched_count > 0)
    }

    pub async fn update_task_status(&self, id: &str, status: &str) -> Result<bool, String> {
        let db = self.get_db().await?;
        let col = db.collection::<Document>("tasks");

        let oid = ObjectId::parse_str(id)
            .map_err(|_| "Mã công việc không hợp lệ!".to_string())?;

        let now = Utc::now();
        let res = col.update_one(
            doc! { "_id": oid },
            doc! { "$set": { "status": status, "updated_at": mongodb::bson::DateTime::from_chrono(now) } }
        ).await.map_err(|e| format!("Lỗi cập nhật trạng thái: {}", e))?;

        Ok(res.matched_count > 0)
    }

    pub async fn delete_task(&self, id: &str) -> Result<bool, String> {
        let db = self.get_db().await?;
        let col = db.collection::<Document>("tasks");

        let oid = ObjectId::parse_str(id)
            .map_err(|_| "Mã công việc không hợp lệ!".to_string())?;

        let res = col.delete_one(doc! { "_id": oid }).await
            .map_err(|e| format!("Lỗi xóa công việc: {}", e))?;

        Ok(res.deleted_count > 0)
    }

    // ==========================================
    // DASHBOARD STATS
    // ==========================================

    pub async fn get_dashboard_stats(&self, for_user: Option<String>) -> Result<DashboardStats, String> {
        let tasks = self.get_tasks(None, None, for_user).await?;
        let now = Utc::now();

        let mut stats = DashboardStats::default();
        stats.total = tasks.len();

        for t in tasks {
            match t.status.as_str() {
                "TODO" => stats.todo += 1,
                "IN_PROGRESS" => stats.in_progress += 1,
                "REVIEW" => stats.review += 1,
                "DONE" => stats.done += 1,
                _ => {}
            }

            if t.status != "DONE" {
                if let Some(due_str) = &t.due_date {
                    if let Ok(due_dt) = DateTime::parse_from_rfc3339(due_str) {
                        let due_utc = due_dt.with_timezone(&Utc);
                        if due_utc < now {
                            stats.overdue += 1;
                        } else if (due_utc - now).num_hours() <= 48 {
                            stats.urgent_or_near += 1;
                        }
                    }
                }
            }
        }

        Ok(stats)
    }

    // Helper: Convert Document to TaskDto
    fn doc_to_task(doc: Document) -> TaskDto {
        let id = doc.get_object_id("_id").ok().map(|oid| oid.to_hex());
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

        TaskDto {
            id,
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
        }
    }
}
