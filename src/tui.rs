use std::io::{self, Read, Write};

use crate::config;
use crate::logger;
use crate::runner;

const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const GRAY: &str = "\x1b[90m";
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const CLEAR_SCREEN: &str = "\x1b[2J\x1b[H";
const HIDE_CURSOR: &str = "\x1b[?25l";
const SHOW_CURSOR: &str = "\x1b[?25h";

struct RawModeGuard;

impl RawModeGuard {
    fn new() -> io::Result<Self> {
        set_raw_mode()?;
        Ok(Self)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = restore_mode();
        let _ = io::stdout().write_all(SHOW_CURSOR.as_bytes());
        let _ = io::stdout().flush();
    }
}

pub fn run_interactive() -> Result<(), Box<dyn std::error::Error>> {
    let names = config::list()?;
    if names.is_empty() {
        println!("application/ 目录下没有找到任何项目");
        return Ok(());
    }

    let mut items: Vec<(String, String)> = Vec::new();
    for name in &names {
        let desc = config::load(name).map(|c| c.description).unwrap_or_default();
        items.push((name.clone(), desc));
    }

    print!("{}{}", HIDE_CURSOR, CLEAR_SCREEN);
    io::stdout().flush()?;

    let _guard = RawModeGuard::new();

    let mut selected = 0usize;
    render(&items, selected)?;

    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut buf = [0u8; 1];

    loop {
        reader.read_exact(&mut buf)?;
        let b = buf[0];

        if b == 27 {
            let mut seq = [0u8; 2];
            if reader.read_exact(&mut seq).is_ok()
                && seq[0] == b'[' {
                    match seq[1] {
                        b'A' => {
                            selected = selected.saturating_sub(1);
                            render(&items, selected)?;
                        }
                        b'B' => {
                            if selected < items.len() - 1 {
                                selected += 1;
                            }
                            render(&items, selected)?;
                        }
                        _ => {}
                    }
                }
            continue;
        }

        match b {
            b'q' | b'Q' => {
                print!("{}", CLEAR_SCREEN);
                io::stdout().flush()?;
                break;
            }
            b'a' | b'A' => {
                print!("{}{}", CLEAR_SCREEN, SHOW_CURSOR);
                io::stdout().flush()?;
                return run_all();
            }
            b'\r' | b'\n' => {
                print!("{}{}", CLEAR_SCREEN, SHOW_CURSOR);
                io::stdout().flush()?;
                return run_direct(&items[selected].0);
            }
            _ => {}
        }
    }

    Ok(())
}

fn render(items: &[(String, String)], selected: usize) -> io::Result<()> {
    let mut stdout = io::stdout();
    write!(stdout, "{}", CLEAR_SCREEN)?;
    writeln!(stdout, "{}Backup Tool - 选择要执行的任务{}\n", BOLD, RESET)?;
    for (i, (name, desc)) in items.iter().enumerate() {
        if i == selected {
            write!(stdout, "  {}➜ {}{}", GREEN, name, RESET)?;
        } else {
            write!(stdout, "    {}", name)?;
        }
        if !desc.is_empty() {
            write!(stdout, " {}- {}{}", GRAY, desc, RESET)?;
        }
        writeln!(stdout)?;
    }
    writeln!(stdout, "\n{}↑↓ 选择 | Enter 执行 | a 执行全部 | q 退出{}", GRAY, RESET)?;
    stdout.flush()
}

pub fn run_direct(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let cfg = config::load(name).map_err(|e| e.to_string())?;
    let log = logger::Logger::new(name)?;

    println!("执行任务: {}{}{}", BOLD, cfg.name, RESET);
    println!("日志文件: {}\n", log.path());

    let result = runner::run(name, &cfg, &log)?;
    if result.exit_code != 0 {
        println!("\n{}任务退出码: {}{}", RED, result.exit_code, RESET);
        std::process::exit(result.exit_code);
    }

    println!("\n{}任务执行成功{}", GREEN, RESET);
    println!("日志: {}", result.log_path);
    Ok(())
}

pub fn run_all() -> Result<(), Box<dyn std::error::Error>> {
    let names = config::list()?;
    if names.is_empty() {
        println!("application/ 目录下没有找到任何项目");
        return Ok(());
    }

    println!("\n{}开始批量备份，共 {} 个项目{}\n", BOLD, names.len(), RESET);

    let mut failed = Vec::new();
    for name in &names {
        let cfg = match config::load(name) {
            Ok(c) => c,
            Err(e) => {
                println!("{}[{}] 配置加载失败: {}{}", RED, name, e, RESET);
                failed.push(name.clone());
                continue;
            }
        };

        let log = match logger::Logger::new(name) {
            Ok(l) => l,
            Err(e) => {
                println!("{}[{}] 日志创建失败: {}{}", RED, name, e, RESET);
                failed.push(name.clone());
                continue;
            }
        };

        println!("{}[{}] {}{}", BOLD, name, cfg.name, RESET);
        println!("  日志: {}", log.path());

        match runner::run(name, &cfg, &log) {
            Ok(result) => {
                if result.exit_code != 0 {
                    println!("  {}退出码: {}{}", RED, result.exit_code, RESET);
                    failed.push(name.clone());
                } else {
                    println!("  {}成功{}", GREEN, RESET);
                }
            }
            Err(e) => {
                println!("  {}执行失败: {}{}", RED, e, RESET);
                failed.push(name.clone());
            }
        }
    }

    println!("\n{}======== 批量备份完成 ========{}", BOLD, RESET);
    println!(
        "总计: {} | 成功: {} | 失败: {}",
        names.len(),
        names.len() - failed.len(),
        failed.len()
    );
    if !failed.is_empty() {
        println!("{}失败项目: {:?}{}", RED, failed, RESET);
        std::process::exit(1);
    }
    Ok(())
}

fn tty_flag() -> &'static str {
    if cfg!(target_os = "macos") {
        "-f"
    } else {
        "-F"
    }
}

fn set_raw_mode() -> io::Result<()> {
    let flag = tty_flag();
    let _ = std::process::Command::new("stty")
        .arg(flag)
        .arg("/dev/tty")
        .arg("cbreak")
        .arg("min")
        .arg("1")
        .stderr(std::process::Stdio::null())
        .status();
    let _ = std::process::Command::new("stty")
        .arg(flag)
        .arg("/dev/tty")
        .arg("-echo")
        .stderr(std::process::Stdio::null())
        .status();
    Ok(())
}

fn restore_mode() -> io::Result<()> {
    let flag = tty_flag();
    let _ = std::process::Command::new("stty")
        .arg(flag)
        .arg("/dev/tty")
        .arg("sane")
        .stderr(std::process::Stdio::null())
        .status();
    Ok(())
}
