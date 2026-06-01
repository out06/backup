# backup - 极简备份工具

让备份像按一个键一样简单。

## 为什么需要它？

日常备份最烦人的不是写脚本，而是**反复输入一长串命令**：

```bash
# 每天都要记住并输入这么长一串...
mysqldump -h 192.168.1.100 -u backup_user -p'xxx' --single-transaction mydb | gzip > /backups/db/$(date +%Y%m%d).sql.gz
rsync -avz --delete --exclude='*.log' /data/www/ /backups/www/
docker exec postgres pg_dumpall -U admin | gzip > /backups/postgres/all.sql.gz
```

**backup 帮你把这些"长串信息"变成"一个名字"**：

```bash
# 第一次：把命令写成脚本，配个名字
# 之后每天：
backup db          # 执行单个备份
backup all         # 全部执行一遍
backup             # 打开交互界面，上下键选择，回车执行
```

## 安装

### 从源码编译

```bash
cargo build --release
# 二进制在 target/release/backup
```

### 放置位置

建议把 `backup` 二进制放在你的备份工作目录根目录，例如：

```
~/backups/
  ├── backup              ← 可执行文件
  ├── .env                ← 环境配置（可选）
  ├── application/        ← 所有备份项目
  │   ├── mysql/
  │   ├── www/
  │   └── postgres/
  └── logs/               ← 自动生成的执行日志
```

## 快速开始

### 1. 创建一个备份项目

```bash
mkdir -p application/mydb
```

在 `application/mydb/default.toml` 写入配置：

```toml
name = "MySQL 每日备份"
script = "backup.sh"
description = "备份生产库并压缩"
args = ["--full"]
```

在 `application/mydb/backup.sh` 写脚本：

```bash
#!/bin/bash
echo "开始备份 MySQL..."
mysqldump -h "$DB_HOST" -u "$DB_USER" -p"$DB_PASS" mydb | gzip > "$BACKUP_DB_PATH"/mydb-$(date +%Y%m%d).sql.gz
echo "备份完成"
```

```bash
chmod +x application/mydb/backup.sh
```

### 2. 执行

```bash
# 方式一：TUI 交互界面（推荐日常使用）
backup

# 方式二：直接执行某个项目
backup mydb

# 方式三：顺序执行所有项目
backup all
```

## 使用方式

### TUI 交互模式（无参数）

```bash
backup
```

启动终端交互界面：

- `↑` `↓` 选择项目
- `Enter` 执行选中的项目
- `a` 执行全部项目
- `q` 退出

> 最适合日常手动备份，不用记项目名，看一眼就执行。

### 直接执行单个项目

```bash
backup <project>
```

示例：

```bash
backup mydb
backup www
backup postgres
```

### 批量执行全部项目

```bash
backup all
```

按 `application/` 下目录顺序，依次执行每个项目。任何一个失败会在最后汇总。

### 其他命令

```bash
backup -v              # 查看版本
backup -h              # 查看帮助
backup upgrade         # 自动升级（需配置 UPGRADE_URL）
```

## 目录结构

```
backup-workspace/
  ├── backup                  # 本程序
  ├── .env                    # 全局环境变量（可选）
  ├── .env.local              # 本地覆盖配置（建议忽略到 git）
  ├── application/            # 所有备份项目
  │   └── <project>/
  │       ├── default.toml    # 项目配置
  │       └── backup.sh       # 备份脚本（名字可自定义）
  └── logs/                   # 执行日志
      └── <project>/
          ├── 20260531120000.log
          └── 20260531130000.log
```

## 项目配置（default.toml）

每个项目目录下必须有一个 `default.toml`：

```toml
name = "显示名称"
script = "backup.sh"           # 要执行的脚本文件名
description = "简短描述"       # TUI 中显示
args = ["--full", "--quiet"]   # 传给脚本的参数（可选）
```

## 环境变量

可以在 `.env` 或 `.env.local` 中配置：

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `APP_DIR` | `application` | 备份项目存放目录 |
| `CONFIG_NAME` | `default.toml` | 配置文件名 |
| `LOG_DIR` | `logs` | 日志根目录 |
| `BACKUP_PATH` | 自动推导 | backup 程序所在目录 |
| `APP_PATH` | `BACKUP_PATH/APP_DIR` | 项目绝对路径 |
| `BACKUP_DB_PATH` | `BACKUP_PATH/db` | 数据库备份存放目录 |
| `DOCKER_PATH` | `docker` | docker 命令路径 |
| `UPGRADE_URL` | 空 | 自动升级下载地址 |

**优先级**：系统环境变量 > `.env.local` > `.env` > 默认值

示例 `.env`：

```bash
APP_DIR=jobs
LOG_DIR=var/log
DOCKER_PATH=/usr/local/bin/docker
UPGRADE_URL=https://example.com/backup
```

### 脚本内可用的环境变量

backup 会自动把以下变量注入到脚本环境中：

```bash
DOCKER_PATH       # docker 命令路径
BACKUP_PATH       # backup 根目录
APP_PATH          # application 目录
BACKUP_DB_PATH    # 数据库备份目录
```

所以脚本里可以直接用：

```bash
$DOCKER_PATH exec mysql mysqldump ... > "$BACKUP_DB_PATH"/xxx.sql
```

## 日志

每次执行都会自动生成日志，保存在 `logs/<project>/<时间>.log`，包含：

- 执行开始/结束时间
- 脚本路径
- stdout / stderr 完整输出
- 退出码

```
[2026-05-31 12:00:00] 开始执行任务: MySQL 每日备份 (项目: mydb)
[2026-05-31 12:00:00] 脚本路径: /backups/application/mydb/backup.sh
开始备份 MySQL...
备份完成
[2026-05-31 12:00:05] 任务执行成功
```

## 自动升级

1. 在 `.env` 中配置下载地址：

```bash
UPGRADE_URL=https://your-server.com/backup-darwin-arm64
```

2. 执行升级：

```bash
backup upgrade
```

3. 重新运行程序验证

## 实际场景示例

### 场景 1：备份多个网站

```
application/
  ├── blog/
  │   ├── default.toml
  │   └── backup.sh          # rsync /var/www/blog /backups/blog
  └── shop/
      ├── default.toml
      └── backup.sh          # rsync /var/www/shop /backups/shop
```

每天：`backup all`，一键备份所有站点。

### 场景 2：数据库 + 文件混合备份

```
application/
  ├── mysql/
  │   └── backup.sh          # mysqldump ...
  ├── redis/
  │   └── backup.sh          # redis-cli SAVE + cp
  └── upload/
      └── backup.sh          # tar czf uploads.tar.gz /data/uploads
```

### 场景 3：配合 cron 定时执行

```bash
# crontab -e
0 2 * * * cd /backups && ./backup all
```

凌晨 2 点自动执行所有备份，日志自动落盘，第二天看日志即可。

## 总结

| 以前 | 现在 |
|------|------|
| 记住并输入长命令 | `backup mydb` |
| 多个备份手忙脚乱 | `backup all` |
| 不知道上次备份成没成功 | 看 `logs/` 自动归档 |
| 换台机器要重新配环境 | 复制 `application/` 目录即可 |

**把复杂的命令藏起来，把简单的名字留下来。**
