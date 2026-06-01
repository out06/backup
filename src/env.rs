use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;

static ENV: OnceLock<Env> = OnceLock::new();

pub struct Env {
    data: HashMap<String, String>,
}

impl Env {
    fn new() -> Self {
        let mut data = HashMap::new();
        load_file(".env", &mut data);
        load_file(".env.local", &mut data);
        Self { data }
    }
}

fn env() -> &'static Env {
    ENV.get_or_init(Env::new)
}

fn load_file(path: &str, data: &mut HashMap<String, String>) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, val)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim().to_string();
        let mut val = val.trim().to_string();
        if val.len() >= 2 && val.starts_with('"') && val.ends_with('"') {
            val = val[1..val.len() - 1].to_string();
        }
        if val.len() >= 2 && val.starts_with('\'') && val.ends_with('\'') {
            val = val[1..val.len() - 1].to_string();
        }
        data.insert(key, val);
    }
}

fn get(key: &str, fallback: &str) -> String {
    if let Ok(v) = std::env::var(key)
        && !v.is_empty() {
            return v;
        }
    if let Some(v) = env().data.get(key)
        && !v.is_empty() {
            return v.clone();
        }
    fallback.to_string()
}

pub fn app_dir() -> String {
    get("APP_DIR", "application")
}

pub fn config_name() -> String {
    get("CONFIG_NAME", "default.toml")
}

pub fn log_dir() -> String {
    get("LOG_DIR", "logs")
}

pub fn upgrade_url() -> String {
    get("UPGRADE_URL", "")
}

pub fn docker_path() -> String {
    get("DOCKER_PATH", "docker")
}

pub fn backup_path() -> PathBuf {
    if let Ok(v) = std::env::var("BACKUP_PATH")
        && !v.is_empty() {
            return PathBuf::from(v);
        }
    if let Some(v) = env().data.get("BACKUP_PATH")
        && !v.is_empty() {
            return PathBuf::from(v);
        }
    std::env::current_exe()
        .ok()
        .and_then(|p| std::fs::canonicalize(&p).ok().or(Some(p)))
        .and_then(|p| p.parent().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn app_path() -> PathBuf {
    if let Ok(v) = std::env::var("APP_PATH")
        && !v.is_empty() {
            return PathBuf::from(v);
        }
    if let Some(v) = env().data.get("APP_PATH")
        && !v.is_empty() {
            return PathBuf::from(v);
        }
    backup_path().join(app_dir())
}

pub fn backup_db_path() -> PathBuf {
    if let Ok(v) = std::env::var("BACKUP_DB_PATH")
        && !v.is_empty() {
            return PathBuf::from(v);
        }
    if let Some(v) = env().data.get("BACKUP_DB_PATH")
        && !v.is_empty() {
            return PathBuf::from(v);
        }
    backup_path().join("db")
}

pub fn all() -> HashMap<String, String> {
    env().data.clone()
}
