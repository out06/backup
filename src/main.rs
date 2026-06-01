mod config;
mod env;
mod logger;
mod runner;
mod tui;
mod upgrade;
mod version;

use std::env::args;
use std::process;

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() < 2 {
        if let Err(e) = tui::run_interactive() {
            eprintln!("错误: {}", e);
            process::exit(1);
        }
        return;
    }

    let cmd = &args[1];
    match cmd.as_str() {
        "-h" | "--help" => print_help(),
        "-v" | "--version" | "version" | "-version" => println!("{}", version::string()),
        "all" => {
            if let Err(e) = tui::run_all() {
                eprintln!("错误: {}", e);
                process::exit(1);
            }
        }
        "upgrade" => {
            if let Err(e) = upgrade::run(&env::upgrade_url()) {
                eprintln!("升级失败: {}", e);
                process::exit(1);
            }
        }
        _ => {
            if let Err(e) = tui::run_direct(cmd) {
                eprintln!("错误: {}", e);
                process::exit(1);
            }
        }
    }
}

fn print_help() {
    println!(
        r#"backup - 备份工具 {}

USAGE:
    backup [COMMAND]

COMMANDS:
    <project>          执行单个备份项目
    all                顺序执行所有备份项目
    upgrade            自动升级程序（需配置 UPGRADE_URL）
    -h, --help         显示帮助
    -v, --version      显示版本

    （无参数）          启动 TUI 交互界面

DIRECTORIES:
    {}/               存放所有备份项目，每个项目一个子目录
    {}/<project>/     单个项目的脚本和配置
    {}/<project>/     单个项目的执行日志

CONFIGURATION:
    每个项目目录下必须包含 {}，格式：

        name = "备份名称"
        script = "backup.sh"
        description = "描述"
        args = ["--full"]

ENVIRONMENT:
    以下变量可在 .env 或 .env.local 中配置：

        APP_DIR          应用目录，默认 "application"
        CONFIG_NAME      配置文件名，默认 "default.toml"
        LOG_DIR          日志根目录，默认 "logs"
        BACKUP_PATH      backup 根目录（默认自动推导）
        APP_PATH         application 绝对路径（默认 BACKUP_PATH/APP_DIR）
        BACKUP_DB_PATH   数据库备份目录（默认 BACKUP_PATH/db）
        DOCKER_PATH      docker 命令路径，默认 "docker"
        UPGRADE_URL      升级下载地址，默认空

    优先级：系统环境变量 > .env.local > .env > 默认值

EXAMPLES:
    # 执行 ls 项目
    backup ls

    # 执行所有项目
    backup all

    # 启动 TUI
    backup

    # 自动升级
    backup upgrade

    # 使用环境变量覆盖配置
    APP_DIR=jobs LOG_DIR=var/log backup all

升级说明:
    1. 在 .env 中配置 UPGRADE_URL，例如：
       UPGRADE_URL=https://example.com/backup

    2. 运行 backup upgrade

    3. 升级完成后，请重新运行程序验证
"#,
        version::short(),
        env::app_dir(),
        env::app_dir(),
        env::log_dir(),
        env::config_name()
    );
}
