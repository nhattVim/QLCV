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

    let uri_x = xor_bytes(uri.as_bytes(), key);
    let db_x = xor_bytes(db_name.as_bytes(), key);

    let dest = PathBuf::from(env::var("OUT_DIR").unwrap()).join("embedded_cfg.rs");
    let body = format!(
        "pub const EMBED_KEY: &[u8] = &{key:?};\npub const EMBED_URI: &[u8] = &{uri:?};\npub const EMBED_DB: &[u8] = &{db:?};\n",
        key = key,
        uri = uri_x,
        db = db_x
    );
    fs::write(dest, body).expect("failed to write embedded_cfg.rs");
}
