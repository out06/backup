use std::io::{self, Read};
use std::process::{Command, Stdio};

use crate::config::Config;
use crate::env;
use crate::logger::Logger;

pub struct Result {
    pub exit_code: i32,
    pub log_path: String,
}

pub fn run(project: &str, cfg: &Config, log: &Logger) -> io::Result<Result> {
    let shell_dir = env::app_path().join(project);
    let script_path = shell_dir.join(&cfg.script);

    if !script_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("脚本不存在: {}", script_path.display()),
        ));
    }

    log.log(format_args!("开始执行任务: {} (项目: {})", cfg.name, project));
    log.log(format_args!("脚本路径: {}", script_path.display()));

    let mut cmd = Command::new("bash");
    cmd.arg(&script_path);
    for arg in &cfg.args {
        cmd.arg(arg);
    }
    cmd.current_dir(&shell_dir);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    inject_env(&mut cmd);
    cmd.env("DOCKER_PATH", env::docker_path());
    cmd.env("BACKUP_PATH", env::backup_path().to_string_lossy().to_string());
    cmd.env("APP_PATH", env::app_path().to_string_lossy().to_string());
    cmd.env("BACKUP_DB_PATH", env::backup_db_path().to_string_lossy().to_string());

    let mut child = cmd.spawn()?;
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let log_out = log.clone();
    let h1 = std::thread::spawn(move || {
        let mut reader = std::io::BufReader::new(stdout);
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let _ = log_out.write_bytes(&buf[..n]);
                }
                Err(_) => break,
            }
        }
    });

    let log_err = log.clone();
    let h2 = std::thread::spawn(move || {
        let mut reader = std::io::BufReader::new(stderr);
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let _ = log_err.write_bytes(&buf[..n]);
                }
                Err(_) => break,
            }
        }
    });

    let status = child.wait()?;
    h1.join().unwrap();
    h2.join().unwrap();

    let exit_code = status.code().unwrap_or(1);
    if exit_code == 0 {
        log.log(format_args!("任务执行成功"));
    } else {
        log.log(format_args!("任务执行失败，退出码: {}", exit_code));
    }

    Ok(Result {
        exit_code,
        log_path: log.path().to_string(),
    })
}

fn inject_env(cmd: &mut Command) {
    for (k, v) in env::all() {
        if k == "PATH" {
            let current_path = std::env::var("PATH").unwrap_or_default();
            let sep = if cfg!(windows) { ';' } else { ':' };
            let new_path = format!("{}{}{}", current_path, sep, v);
            cmd.env("PATH", new_path);
        } else if std::env::var(&k).unwrap_or_default().is_empty() {
            cmd.env(k, v);
        }
    }
}
