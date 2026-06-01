use std::fs;
use crate::env;

pub struct Config {
    pub name: String,
    pub script: String,
    pub description: String,
    pub args: Vec<String>,
}

pub fn load(name: &str) -> Result<Config, String> {
    let path = env::app_path().join(name).join(env::config_name());
    let data = fs::read_to_string(&path)
        .map_err(|_| format!("配置文件不存在: {}", path.display()))?;
    let cfg = parse_toml(&data).map_err(|e| format!("解析 TOML 失败: {}", e))?;
    if cfg.script.is_empty() {
        return Err("配置缺少 script 字段".to_string());
    }
    Ok(cfg)
}

pub fn list() -> Result<Vec<String>, std::io::Error> {
    let app_dir = env::app_path();
    let mut names = Vec::new();
    let entries = match fs::read_dir(&app_dir) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(names),
        Err(e) => return Err(e),
    };
    for entry in entries.flatten() {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let toml_path = app_dir.join(entry.file_name()).join(env::config_name());
        if toml_path.exists() {
            names.push(entry.file_name().to_string_lossy().to_string());
        }
    }
    Ok(names)
}

fn parse_toml(s: &str) -> Result<Config, String> {
    let mut cfg = Config {
        name: String::new(),
        script: String::new(),
        description: String::new(),
        args: Vec::new(),
    };
    for line in s.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, val)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let val = val.trim();
        match key {
            "name" => cfg.name = parse_string(val),
            "script" => cfg.script = parse_string(val),
            "description" => cfg.description = parse_string(val),
            "args" => cfg.args = parse_string_array(val),
            _ => {}
        }
    }
    Ok(cfg)
}

fn parse_string(s: &str) -> String {
    let s = s.trim();
    if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

fn parse_string_array(s: &str) -> Vec<String> {
    let s = s.trim();
    if !s.starts_with('[') || !s.ends_with(']') {
        return Vec::new();
    }
    let inner = s[1..s.len() - 1].trim();
    if inner.is_empty() {
        return Vec::new();
    }
    inner.split(',').map(parse_string).collect()
}
