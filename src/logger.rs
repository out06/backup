use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::env;

#[derive(Clone)]
pub struct Logger {
    file: Arc<Mutex<File>>,
    path: PathBuf,
}

impl Logger {
    pub fn new(project: &str) -> io::Result<Self> {
        let logs_dir = env::backup_path().join(env::log_dir()).join(project);
        fs::create_dir_all(&logs_dir)?;
        let name = format!("{}.log", now_compact());
        let path = logs_dir.join(&name);
        let file = File::create(&path)?;
        Ok(Self {
            file: Arc::new(Mutex::new(file)),
            path,
        })
    }

    pub fn path(&self) -> &str {
        self.path.to_str().unwrap_or("")
    }

    pub fn log(&self, args: std::fmt::Arguments<'_>) {
        let timestamp = now_fmt();
        let line = format!("[{}] {}\n", timestamp, std::fmt::format(args));
        let _ = self.file.lock().unwrap().write_all(line.as_bytes());
    }

    pub fn write_bytes(&self, buf: &[u8]) -> io::Result<()> {
        self.file.lock().unwrap().write_all(buf)
    }
}

impl Write for Logger {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.file.lock().unwrap().write_all(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.lock().unwrap().flush()
    }
}

fn now_fmt() -> String {
    let output = std::process::Command::new("date")
        .arg("+%Y-%m-%d %H:%M:%S")
        .output();
    match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => {
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            format!("{}", ts)
        }
    }
}

fn now_compact() -> String {
    let output = std::process::Command::new("date")
        .arg("+%Y%m%d%H%M%S")
        .output();
    match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => {
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            format!("{}", ts)
        }
    }
}
