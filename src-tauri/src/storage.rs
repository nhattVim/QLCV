use std::path::Path;
use std::fs;
use chrono::Utc;
use s3::creds::Credentials;
use s3::{Bucket, Region};
use uuid::Uuid;
use crate::models::TaskAttachment;

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

pub struct StorageConfig {
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: String,
    pub bucket: String,
    pub region: String,
    pub public_domain: String,
}

pub fn resolve_storage_config() -> StorageConfig {
    let _ = dotenvy::dotenv();

    let endpoint = std::env::var("S3_ENDPOINT")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| xor_decode(embedded::EMBED_S3_ENDPOINT, embedded::EMBED_KEY));

    let access_key = std::env::var("S3_ACCESS_KEY_ID")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| xor_decode(embedded::EMBED_S3_ACCESS_KEY, embedded::EMBED_KEY));

    let secret_key = std::env::var("S3_SECRET_ACCESS_KEY")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| xor_decode(embedded::EMBED_S3_SECRET_KEY, embedded::EMBED_KEY));

    let bucket = std::env::var("S3_BUCKET")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| xor_decode(embedded::EMBED_S3_BUCKET, embedded::EMBED_KEY));

    let region = std::env::var("S3_REGION")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            let r = xor_decode(embedded::EMBED_S3_REGION, embedded::EMBED_KEY);
            if r.is_empty() { "auto".to_string() } else { r }
        });

    let public_domain = std::env::var("S3_PUBLIC_DOMAIN")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| xor_decode(embedded::EMBED_S3_PUBLIC_DOMAIN, embedded::EMBED_KEY));

    StorageConfig {
        endpoint,
        access_key,
        secret_key,
        bucket,
        region,
        public_domain,
    }
}

pub fn is_storage_configured() -> bool {
    let cfg = resolve_storage_config();
    !cfg.endpoint.trim().is_empty()
        && !cfg.access_key.trim().is_empty()
        && !cfg.secret_key.trim().is_empty()
        && !cfg.bucket.trim().is_empty()
}

fn get_bucket_client() -> Result<(Box<Bucket>, StorageConfig), String> {
    let cfg = resolve_storage_config();
    if !is_storage_configured() {
        return Err("Chưa cấu hình Cloudflare R2 / S3 Storage (thiếu S3_ENDPOINT, S3_ACCESS_KEY_ID, S3_SECRET_ACCESS_KEY hoặc S3_BUCKET)".to_string());
    }

    let credentials = Credentials::new(
        Some(&cfg.access_key),
        Some(&cfg.secret_key),
        None,
        None,
        None,
    ).map_err(|e| format!("Lỗi xác thực S3: {}", e))?;

    // Normalize endpoint URL (add https:// if missing)
    let mut ep = cfg.endpoint.trim().to_string();
    if !ep.starts_with("http://") && !ep.starts_with("https://") {
        ep = format!("https://{}", ep);
    }
    // If user provided just account id, format into Cloudflare R2 endpoint
    if !ep.contains('.') {
        ep = format!("https://{}.r2.cloudflarestorage.com", ep);
    }

    let region = Region::Custom {
        region: if cfg.region.is_empty() { "auto".to_string() } else { cfg.region.clone() },
        endpoint: ep,
    };

    let bucket = Bucket::new(&cfg.bucket, region, credentials)
        .map_err(|e| format!("Không thể khởi tạo Bucket client: {}", e))?
        .with_path_style();

    Ok((bucket, cfg))
}

pub async fn upload_attachment(
    team_id: &str,
    workspace_id: &str,
    ticket_id: &str,
    file_name: &str,
    data: &[u8],
    content_type: &str,
    uploaded_by: &str,
) -> Result<TaskAttachment, String> {
    let (bucket, cfg) = get_bucket_client()?;

    let att_id = Uuid::new_v4().to_string();
    let safe_file_name = Path::new(file_name)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("document");

    let clean_team = if team_id.trim().is_empty() { "default_team" } else { team_id.trim() };
    let clean_workspace = if workspace_id.trim().is_empty() { "default_workspace" } else { workspace_id.trim() };
    let clean_ticket = if ticket_id.trim().is_empty() { "draft" } else { ticket_id.trim() };

    let object_key = format!("{}/{}/{}/{}_{}", clean_team, clean_workspace, clean_ticket, att_id, safe_file_name);

    let mime = if content_type.is_empty() {
        "application/octet-stream"
    } else {
        content_type
    };

    let response = bucket
        .put_object_with_content_type(&object_key, data, mime)
        .await
        .map_err(|e| format!("Lỗi tải tệp lên Cloudflare R2: {}", e))?;

    if response.status_code() < 200 || response.status_code() >= 300 {
        return Err(format!(
            "Cloudflare R2 trả về lỗi HTTP {}: {}",
            response.status_code(),
            String::from_utf8_lossy(response.as_slice())
        ));
    }

    // Determine download URL
    let download_url = if !cfg.public_domain.trim().is_empty() {
        Some(format!("{}/{}", cfg.public_domain.trim_end_matches('/'), object_key))
    } else {
        // Generate presigned URL for 24 hours (86400 seconds)
        bucket.presign_get(&object_key, 86400, None).await.ok()
    };

    Ok(TaskAttachment {
        id: att_id,
        file_name: safe_file_name.to_string(),
        file_size: data.len() as u64,
        content_type: mime.to_string(),
        object_key,
        uploaded_by: uploaded_by.to_string(),
        uploaded_at: Utc::now().to_rfc3339(),
        download_url,
    })
}

pub async fn delete_attachment(object_key: &str) -> Result<(), String> {
    let (bucket, _) = get_bucket_client()?;
    let _ = bucket.delete_object(object_key).await
        .map_err(|e| format!("Lỗi xóa tệp trên Cloudflare R2: {}", e))?;
    Ok(())
}

pub async fn get_attachment_download_url(object_key: &str) -> Result<String, String> {
    let (bucket, cfg) = get_bucket_client()?;

    if !cfg.public_domain.trim().is_empty() {
        return Ok(format!("{}/{}", cfg.public_domain.trim_end_matches('/'), object_key));
    }

    // Presign for 2 hours (7200s)
    bucket.presign_get(object_key, 7200, None)
        .await
        .map_err(|e| format!("Không thể tạo liên kết tải tệp: {}", e))
}

pub async fn download_attachment_bytes(object_key: &str) -> Result<Vec<u8>, String> {
    let (bucket, _) = get_bucket_client()?;
    let resp = bucket.get_object(object_key).await
        .map_err(|e| format!("Lỗi tải dữ liệu tệp từ Cloudflare R2: {}", e))?;
    Ok(resp.bytes().to_vec())
}

pub async fn open_attachment_in_viewer(object_key: &str, file_name: &str) -> Result<String, String> {
    let bytes = download_attachment_bytes(object_key).await?;

    let safe_name = Path::new(file_name)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("document");

    let temp_dir = std::env::temp_dir().join("qlcv_cache");
    let _ = fs::create_dir_all(&temp_dir);

    let target_path = temp_dir.join(safe_name);
    fs::write(&target_path, bytes)
        .map_err(|e| format!("Không thể ghi file tạm: {}", e))?;

    let path_str = target_path.to_string_lossy().to_string();

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let mut cmd = std::process::Command::new("cmd");
        cmd.args(["/c", "start", "", &path_str]);
        cmd.creation_flags(CREATE_NO_WINDOW);
        let _ = cmd.spawn();
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg(&path_str)
            .spawn();
    }

    Ok(path_str)
}

