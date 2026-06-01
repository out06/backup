use std::fs;
use std::path::PathBuf;

pub fn run(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    if url.is_empty() {
        return Err("未配置升级地址，请在 .env 中设置 UPGRADE_URL".into());
    }

    println!("正在检查更新...");
    println!("下载地址: {}", url);

    let tmp_file = download(url)?;

    let exe_path = std::env::current_exe()?;
    let exe_path = fs::canonicalize(&exe_path).unwrap_or(exe_path);

    println!("当前程序: {}", exe_path.display());
    println!("替换中...");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&tmp_file)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&tmp_file, perms)?;
    }

    fs::rename(&tmp_file, &exe_path)?;

    println!("升级成功！请重新运行程序。");
    Ok(())
}

fn download(url: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let tmp_path = std::env::temp_dir().join(format!(
        "backup_upgrade_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs()
    ));

    let output = std::process::Command::new("curl")
        .args(["-fsSL", "-o", tmp_path.to_str().unwrap_or("/dev/null"), url])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("curl 下载失败: {}", stderr).into());
    }

    let meta = fs::metadata(&tmp_path)?;
    if meta.len() == 0 {
        return Err("下载文件为空".into());
    }

    println!("下载完成: {} bytes", meta.len());
    Ok(tmp_path)
}
