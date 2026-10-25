use std::env;
use std::fs;
use std::path::PathBuf;

fn xor_bytes(data: &[u8], key: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, b)| b ^ key[i % key.len()])
        .collect()
}

fn load_env_value(env_path: &PathBuf, key: &str) -> Option<String> {
    if let Ok(v) = env::var(key) {
        if !v.is_empty() {
            return Some(v);
        }
    }
    if let Ok(content) = fs::read_to_string(env_path) {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                if k.trim() == key {
                    return Some(v.trim().trim_matches('"').to_string());
                }
            }
        }
    }
    None
}

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let env_path = manifest_dir.join("..").join(".env");
    println!("cargo:rerun-if-changed={}", env_path.display());

    tauri_build::build();

    let key = b"QLCV_CFG_XOR_7K";
    let uri = load_env_value(&env_path, "MONGODB_URI").unwrap_or_default();
    let db_name = load_env_value(&env_path, "DB_NAME").unwrap_or_else(|| "qlcv_db".to_string());

    let s3_endpoint = load_env_value(&env_path, "S3_ENDPOINT").unwrap_or_default();
    let s3_access_key = load_env_value(&env_path, "S3_ACCESS_KEY_ID").unwrap_or_default();
    let s3_secret_key = load_env_value(&env_path, "S3_SECRET_ACCESS_KEY").unwrap_or_default();
    let s3_bucket = load_env_value(&env_path, "S3_BUCKET").unwrap_or_default();
    let s3_region = load_env_value(&env_path, "S3_REGION").unwrap_or_else(|| "auto".to_string());
    let s3_public_domain = load_env_value(&env_path, "S3_PUBLIC_DOMAIN").unwrap_or_default();

    let uri_x = xor_bytes(uri.as_bytes(), key);
    let db_x = xor_bytes(db_name.as_bytes(), key);
    let s3_endpoint_x = xor_bytes(s3_endpoint.as_bytes(), key);
    let s3_access_key_x = xor_bytes(s3_access_key.as_bytes(), key);
    let s3_secret_key_x = xor_bytes(s3_secret_key.as_bytes(), key);
    let s3_bucket_x = xor_bytes(s3_bucket.as_bytes(), key);
    let s3_region_x = xor_bytes(s3_region.as_bytes(), key);
    let s3_public_domain_x = xor_bytes(s3_public_domain.as_bytes(), key);

    let dest = PathBuf::from(env::var("OUT_DIR").unwrap()).join("embedded_cfg.rs");
    let body = format!(
        "pub const EMBED_KEY: &[u8] = &{key:?};\n\
         pub const EMBED_URI: &[u8] = &{uri:?};\n\
         pub const EMBED_DB: &[u8] = &{db:?};\n\
         pub const EMBED_S3_ENDPOINT: &[u8] = &{s3_endpoint:?};\n\
         pub const EMBED_S3_ACCESS_KEY: &[u8] = &{s3_access_key:?};\n\
         pub const EMBED_S3_SECRET_KEY: &[u8] = &{s3_secret_key:?};\n\
         pub const EMBED_S3_BUCKET: &[u8] = &{s3_bucket:?};\n\
         pub const EMBED_S3_REGION: &[u8] = &{s3_region:?};\n\
         pub const EMBED_S3_PUBLIC_DOMAIN: &[u8] = &{s3_public_domain:?};\n",
        key = key,
        uri = uri_x,
        db = db_x,
        s3_endpoint = s3_endpoint_x,
        s3_access_key = s3_access_key_x,
        s3_secret_key = s3_secret_key_x,
        s3_bucket = s3_bucket_x,
        s3_region = s3_region_x,
        s3_public_domain = s3_public_domain_x,
    );
    fs::write(dest, body).expect("failed to write embedded_cfg.rs");
}
