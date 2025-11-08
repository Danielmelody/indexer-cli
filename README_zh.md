# indexer-cli

[English](README.md) | [简体中文](README_zh.md)

> 一个生产就绪的命令行工具，用于自动化网站索引工作流程，支持 Google Indexing API 和 IndexNow 协议

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Build Status](https://img.shields.io/github/actions/workflow/status/your-username/indexer-cli/ci.yml?branch=master)](https://github.com/your-username/indexer-cli/actions)
[![Crates.io](https://img.shields.io/crates/v/indexer-cli.svg)](https://crates.io/crates/indexer-cli)
[![Downloads](https://img.shields.io/crates/d/indexer-cli.svg)](https://crates.io/crates/indexer-cli)

**indexer-cli** 是一个强大的命令行工具，可自动将 URL 提交到搜索引擎。它无缝集成了 Google Indexing API 和 IndexNow 协议，帮助您更快、更高效地让内容被索引。

## 特性

- **Google Indexing API 集成**
  - 使用服务账号的 OAuth2 身份验证
  - 支持 UPDATE/DELETE 操作的 URL 提交
  - 元数据检索和状态检查
  - 智能速率限制和配额管理
  - 指数退避重试逻辑

- **IndexNow API 支持**
  - 提交到多个搜索引擎（Bing、Yandex、Seznam、Naver）
  - 批量提交最多 10,000 个 URL
  - API 密钥生成和验证
  - 密钥文件托管验证

- **站点地图处理**
  - 解析 XML 站点地图和站点地图索引
  - 递归站点地图索引遍历
  - 支持 gzip 压缩的站点地图
  - 按模式、日期和优先级过滤 URL
  - 自动 URL 提取和去重

- **提交历史跟踪**
  - 使用 SQLite 数据库持久化存储
  - 防止重复提交
  - 查询和导出提交历史
  - 统计和报告

- **高级功能**
  - 带进度条的并发批处理
  - 可配置的重试策略
  - URL 验证和过滤
  - 监视模式用于持续监控
  - 带日志轮转的全面日志记录
  - 用于测试的模拟运行模式

## 目录

- [安装](#安装)
- [快速开始](#快速开始)
- [配置](#配置)
- [使用方法](#使用方法)
  - [初始化配置](#初始化配置)
  - [Google Indexing API](#google-indexing-api)
  - [IndexNow API](#indexnow-api)
  - [站点地图操作](#站点地图操作)
  - [提交历史](#提交历史)
  - [监视模式](#监视模式)
- [Google 设置指南](#google-设置指南)
- [IndexNow 设置指南](#indexnow-设置指南)
- [高级用法](#高级用法)
- [架构](#架构)
- [开发](#开发)
- [故障排除](#故障排除)
- [常见问题](#常见问题)
- [对比](#对比)
- [许可证](#许可证)

## 安装

### 前置要求

- **Rust 1.70 或更高版本** - 从 [rustup.rs](https://rustup.rs/) 安装
- **SQLite 3** - 大多数系统上通常预装

### 从源代码安装

克隆仓库并构建：

```bash
git clone https://github.com/your-username/indexer-cli.git
cd indexer-cli
cargo build --release
```

二进制文件将位于 `target/release/indexer-cli`。

### 全局安装

将二进制文件安装到系统：

```bash
cargo install --path .
```

### 从 crates.io 安装（即将推出）

```bash
cargo install indexer-cli
```

## 快速开始

### 1. 初始化配置

使用交互式向导创建配置文件：

```bash
indexer-cli init
```

这将在 `~/.indexer-cli/config.yaml` 创建默认设置。

### 2. 配置 Google 服务账号

设置 Google Indexing API 凭据：

```bash
indexer-cli google setup --service-account /path/to/service-account.json
```

详细说明请参阅 [Google 设置指南](#google-设置指南)。

### 3. 配置 IndexNow API 密钥

生成并配置 IndexNow API 密钥：

```bash
# 生成新密钥
indexer-cli indexnow generate-key --length 32 --save

# 或设置现有密钥
indexer-cli indexnow setup --key your-api-key-here
```

详细说明请参阅 [IndexNow 设置指南](#indexnow-设置指南)。

### 4. 提交您的第一个 URL

将 URL 提交到所有已配置的 API：

```bash
indexer-cli submit https://example.com/your-page
```

或提交到特定 API：

```bash
# 仅 Google
indexer-cli google submit https://example.com/your-page

# 仅 IndexNow
indexer-cli indexnow submit https://example.com/your-page
```

### 5. 从站点地图提交

从站点地图提取 URL 并提交：

```bash
indexer-cli sitemap parse https://example.com/sitemap.xml | \
  indexer-cli submit --file -
```

## 配置

### 配置文件位置

配置文件位于：
- **全局**：`~/.indexer-cli/config.yaml`
- **项目**：`./.indexer-cli/config.yaml`（覆盖全局配置）

您可以使用 `--config` 标志指定自定义位置：

```bash
indexer-cli --config /path/to/config.yaml <command>
```

### 配置格式

配置文件使用 YAML 格式：

```yaml
# Google Indexing API 配置
google:
  enabled: true
  service_account_file: ~/.indexer-cli/service-account.json
  quota:
    daily_limit: 200
    rate_limit: 380
  batch_size: 100

# IndexNow API 配置
indexnow:
  enabled: true
  api_key: your-32-character-api-key-here
  key_location: https://example.com/your-api-key.txt
  endpoints:
    - https://api.indexnow.org/indexnow
    - https://www.bing.com/indexnow
    - https://yandex.com/indexnow
  batch_size: 10000

# 站点地图配置
sitemap:
  url: https://example.com/sitemap.xml
  follow_index: true
  filters:
    url_pattern: ".*"
    lastmod_after: null
    priority_min: 0.0

# 历史跟踪
history:
  enabled: true
  database_path: ~/.indexer-cli/history.db
  retention_days: 365

# 日志配置
logging:
  level: info
  file: ~/.indexer-cli/indexer.log
  max_size_mb: 10
  max_backups: 5

# 重试配置
retry:
  enabled: true
  max_attempts: 3
  backoff_factor: 2
  max_wait_seconds: 60

# 输出配置
output:
  format: text
  color: true
  verbose: false
```

### 环境变量

也可以通过环境变量设置配置：

```bash
export INDEXER_CONFIG=/path/to/config.yaml
export INDEXER_GOOGLE_SERVICE_ACCOUNT=/path/to/service-account.json
export INDEXER_INDEXNOW_API_KEY=your-api-key
```

## 使用方法

### 初始化配置

创建新的配置文件：

```bash
# 交互式向导（默认）
indexer-cli init

# 创建全局配置
indexer-cli init --global

# 覆盖现有配置
indexer-cli init --force

# 使用默认值的非交互式模式
indexer-cli init --non-interactive
```

### Google Indexing API

#### 设置

配置 Google 服务账号凭据：

```bash
# 设置服务账号
indexer-cli google setup --service-account /path/to/service-account.json

# 保存到全局配置
indexer-cli google setup --service-account /path/to/service-account.json --global

# 验证配置
indexer-cli google verify
```

#### 提交 URL

提交一个或多个 URL：

```bash
# 单个 URL
indexer-cli google submit https://example.com/page1

# 多个 URL
indexer-cli google submit https://example.com/page1 https://example.com/page2

# 从文件读取（每行一个 URL）
indexer-cli google submit --file urls.txt

# 从站点地图
indexer-cli google submit --sitemap https://example.com/sitemap.xml

# 使用 DELETE 操作
indexer-cli google submit https://example.com/old-page --action url-deleted

# 使用过滤器
indexer-cli google submit --sitemap https://example.com/sitemap.xml \
  --filter "^https://example.com/blog/" \
  --since 2024-01-01

# 模拟运行（不实际提交）
indexer-cli google submit https://example.com/page1 --dry-run
```

#### 检查状态

检查 URL 的索引状态：

```bash
# 检查单个 URL
indexer-cli google status https://example.com/page1

# 检查多个 URL
indexer-cli google status --file urls.txt

# 输出为 JSON
indexer-cli google status https://example.com/page1 --format json
```

#### 配额管理

查看您的 API 配额使用情况：

```bash
indexer-cli google quota
```

### IndexNow API

#### 设置

配置 IndexNow API 密钥：

```bash
# 生成新密钥
indexer-cli indexnow generate-key --length 32

# 生成并保存到配置
indexer-cli indexnow generate-key --length 32 --save

# 生成并输出密钥文件
indexer-cli indexnow generate-key --length 32 --output /var/www/html/

# 设置现有密钥
indexer-cli indexnow setup --key your-api-key-here \
  --key-location https://example.com/your-api-key.txt

# 验证密钥文件是否可访问
indexer-cli indexnow verify
```

#### 提交 URL

将 URL 提交到 IndexNow：

```bash
# 单个 URL
indexer-cli indexnow submit https://example.com/page1

# 多个 URL
indexer-cli indexnow submit https://example.com/page1 https://example.com/page2

# 从文件
indexer-cli indexnow submit --file urls.txt

# 从站点地图
indexer-cli indexnow submit --sitemap https://example.com/sitemap.xml

# 仅提交到特定端点
indexer-cli indexnow submit https://example.com/page1 --endpoint bing

# 使用过滤器
indexer-cli indexnow submit --sitemap https://example.com/sitemap.xml \
  --filter "^https://example.com/products/" \
  --since 2024-01-01

# 批量大小控制
indexer-cli indexnow submit --file urls.txt --batch-size 1000
```

### 统一提交命令

一次性提交到所有已配置的 API：

```bash
# 提交到所有 API
indexer-cli submit https://example.com/page1

# 从文件
indexer-cli submit --file urls.txt

# 从站点地图
indexer-cli submit --sitemap https://example.com/sitemap.xml

# 提交到特定 API
indexer-cli submit https://example.com/page1 --api google
indexer-cli submit https://example.com/page1 --api indexnow

# 使用选项
indexer-cli submit --sitemap https://example.com/sitemap.xml \
  --filter "^https://example.com/" \
  --since 2024-01-01 \
  --batch-size 50 \
  --format json
```

### 站点地图操作

#### 解析站点地图

解析并显示站点地图内容：

```bash
# 解析站点地图
indexer-cli sitemap parse https://example.com/sitemap.xml

# 跟随站点地图索引
indexer-cli sitemap parse https://example.com/sitemap.xml --follow-index

# 输出为 JSON
indexer-cli sitemap parse https://example.com/sitemap.xml --format json
```

#### 列出 URL

列出站点地图中的所有 URL：

```bash
# 列出所有 URL
indexer-cli sitemap list https://example.com/sitemap.xml

# 使用过滤器
indexer-cli sitemap list https://example.com/sitemap.xml \
  --filter "^https://example.com/blog/"

# 修改日期之后
indexer-cli sitemap list https://example.com/sitemap.xml \
  --since 2024-01-01

# 限制结果
indexer-cli sitemap list https://example.com/sitemap.xml --limit 100
```

#### 导出 URL

将站点地图 URL 导出到文件：

```bash
# 导出到文本文件
indexer-cli sitemap export https://example.com/sitemap.xml --output urls.txt

# 使用过滤器
indexer-cli sitemap export https://example.com/sitemap.xml \
  --output urls.txt \
  --filter "^https://example.com/products/" \
  --since 2024-01-01
```

#### 站点地图统计

显示站点地图统计信息：

```bash
# 显示统计
indexer-cli sitemap stats https://example.com/sitemap.xml

# 输出为 JSON
indexer-cli sitemap stats https://example.com/sitemap.xml --format json
```

#### 验证站点地图

验证站点地图格式和结构：

```bash
indexer-cli sitemap validate https://example.com/sitemap.xml
```

### 提交历史

#### 列出历史

查看最近的提交历史：

```bash
# 列出最近 20 次提交
indexer-cli history list

# 列出最近 50 次提交
indexer-cli history list --limit 50

# 输出为 JSON
indexer-cli history list --format json
```

#### 搜索历史

使用过滤器搜索提交历史：

```bash
# 按 URL 模式搜索
indexer-cli history search --url "example.com/blog"

# 按 API 搜索
indexer-cli history search --api google

# 按状态搜索
indexer-cli history search --status success

# 按日期范围搜索
indexer-cli history search --since 2024-01-01 --until 2024-01-31

# 组合过滤器
indexer-cli history search \
  --url "example.com" \
  --api indexnow \
  --status success \
  --since 2024-01-01 \
  --limit 100
```

#### 历史统计

查看提交统计信息：

```bash
# 整体统计
indexer-cli history stats

# 日期范围统计
indexer-cli history stats --since 2024-01-01 --until 2024-01-31

# 输出为 JSON
indexer-cli history stats --format json
```

#### 导出历史

导出提交历史：

```bash
# 导出为 CSV
indexer-cli history export --output history.csv --format csv

# 导出为 JSON
indexer-cli history export --output history.json --format json

# 导出日期范围
indexer-cli history export --output history.csv \
  --since 2024-01-01 --until 2024-01-31
```

#### 清理历史

清理旧的历史记录：

```bash
# 删除 90 天前的记录
indexer-cli history clean --older-than 90

# 删除所有记录
indexer-cli history clean --all

# 跳过确认提示
indexer-cli history clean --older-than 90 --yes
```

### 监视模式

持续监控站点地图的变化并自动提交新 URL：

```bash
# 监视站点地图（每小时检查一次）
indexer-cli watch --sitemap https://example.com/sitemap.xml

# 自定义检查间隔（以秒为单位）
indexer-cli watch --sitemap https://example.com/sitemap.xml --interval 1800

# 提交到特定 API
indexer-cli watch --sitemap https://example.com/sitemap.xml --api google

# 作为守护进程运行
indexer-cli watch --sitemap https://example.com/sitemap.xml --daemon

# 使用 PID 文件
indexer-cli watch --sitemap https://example.com/sitemap.xml \
  --daemon --pid-file /var/run/indexer-cli.pid
```

### 配置管理

管理配置设置：

```bash
# 列出所有设置
indexer-cli config list

# 获取特定设置
indexer-cli config get google.enabled

# 设置一个设置
indexer-cli config set google.enabled true

# 在全局配置中设置
indexer-cli config set google.batch_size 50 --global

# 验证配置
indexer-cli config validate

# 显示配置文件路径
indexer-cli config path
```

### 验证

验证您的配置和设置：

```bash
# 验证所有内容
indexer-cli validate

# 仅验证 Google 配置
indexer-cli validate google

# 仅验证 IndexNow 配置
indexer-cli validate indexnow

# 检查 IndexNow 密钥文件可访问性
indexer-cli validate --check-key-file

# 输出为 JSON
indexer-cli validate --format json
```

## Google 设置指南

### 前置要求

1. 一个 Google Cloud Platform (GCP) 账号
2. 在 Google Search Console 中验证的网站
3. 添加为网站属性的 URL

### 步骤 1：创建 GCP 项目

1. 前往 [Google Cloud Console](https://console.cloud.google.com/)
2. 创建新项目或选择现有项目
3. 记下您的项目 ID

### 步骤 2：启用 Indexing API

1. 导航到 **APIs & Services** > **Library**
2. 搜索 "Web Search Indexing API" 或 "Indexing API"
3. 点击 **Enable**

### 步骤 3：创建服务账号

1. 前往 **APIs & Services** > **Credentials**
2. 点击 **Create Credentials** > **Service Account**
3. 输入名称（例如，"indexer-cli-service"）
4. 点击 **Create and Continue**
5. 跳过可选步骤并点击 **Done**

### 步骤 4：生成服务账号密钥

1. 在服务账号列表中，点击刚创建的账号
2. 前往 **Keys** 选项卡
3. 点击 **Add Key** > **Create new key**
4. 选择 **JSON** 格式
5. 点击 **Create** - 密钥文件将被下载
6. 将文件安全保存（例如，`~/.indexer-cli/service-account.json`）

### 步骤 5：授予 Search Console 访问权限

1. 前往 [Google Search Console](https://search.google.com/search-console/)
2. 选择您的资源
3. 前往 **Settings** > **Users and permissions**
4. 点击 **Add user**
5. 输入服务账号邮箱（格式：`name@project-id.iam.gserviceaccount.com`）
6. 选择 **Owner** 权限级别
7. 点击 **Add**

### 步骤 6：配置 indexer-cli

```bash
indexer-cli google setup --service-account ~/.indexer-cli/service-account.json
indexer-cli google verify
```

### 配额限制

- **每日发布限制**：每天 200 个 URL 通知
- **速率限制**：每分钟 380 个请求（总计）
- **元数据速率限制**：每分钟 180 个请求

该工具通过速率限制和配额跟踪自动遵守这些限制。

### 最佳实践

- 仅提交您拥有并在 Search Console 中验证的 URL
- 对新页面或修改的页面使用 UPDATE 操作
- 对删除的页面使用 DELETE 操作
- 不要不必要地重新提交 URL（使用历史跟踪）
- 定期监控您的配额使用情况

## IndexNow 设置指南

### 什么是 IndexNow？

IndexNow 是一个开放协议，允许网站所有者立即通知搜索引擎有关最新内容的更改。支持的搜索引擎包括：

- Microsoft Bing
- Yandex
- Seznam.cz
- Naver

### 步骤 1：生成 API 密钥

生成新的 API 密钥（建议 32 个字符）：

```bash
indexer-cli indexnow generate-key --length 32
```

示例输出：
```
Generated IndexNow API key: 3f8a9c2b1d7e6f5a4c3b2a1d7e6f5a4c
```

### 步骤 2：创建密钥文件

使用您的 API 密钥创建文本文件并将其托管在您网站的根目录：

1. 创建文件：`your-api-key.txt`
2. 内容：确切的 API 密钥（无额外空格或换行符）
3. 上传到：`https://yourdomain.com/your-api-key.txt`

示例：
```bash
echo -n "3f8a9c2b1d7e6f5a4c3b2a1d7e6f5a4c" > 3f8a9c2b1d7e6f5a4c3b2a1d7e6f5a4c.txt
# 将此文件上传到您的 Web 服务器的文档根目录
```

### 步骤 3：配置 indexer-cli

```bash
indexer-cli indexnow setup \
  --key 3f8a9c2b1d7e6f5a4c3b2a1d7e6f5a4c \
  --key-location https://yourdomain.com/3f8a9c2b1d7e6f5a4c3b2a1d7e6f5a4c.txt
```

### 步骤 4：验证设置

验证密钥文件是否可访问：

```bash
indexer-cli indexnow verify
```

### 密钥要求

- **长度**：8-128 个字符（建议 32 个）
- **字符**：仅字母数字（a-z、A-Z、0-9）和连字符（-）
- **文件位置**：必须通过 HTTPS 在域根目录可访问
- **文件内容**：必须与 API 密钥完全匹配

### 最佳实践

- 保持 API 密钥私密但密钥文件公开
- 对密钥文件位置使用 HTTPS
- 尽可能批量提交 URL（最多 10,000 个）
- 在一个请求中包含来自同一主机的所有 URL
- 不要在短时间内重复提交相同的 URL

### 支持的端点

该工具同时提交到多个端点：

- `https://api.indexnow.org/indexnow` - 主端点
- `https://www.bing.com/indexnow` - Bing 直连
- `https://yandex.com/indexnow` - Yandex 直连

您可以使用以下命令提交到特定端点：

```bash
indexer-cli indexnow submit URL --endpoint bing
```

## 高级用法

### 批处理

高效处理大量 URL：

```bash
# 从站点地图使用自定义批量大小
indexer-cli submit --sitemap https://example.com/sitemap.xml \
  --batch-size 100

# 多个站点地图
cat sitemap-urls.txt | while read sitemap; do
  indexer-cli submit --sitemap "$sitemap"
done
```

### URL 过滤

使用正则表达式模式过滤 URL：

```bash
# 仅提交博客文章
indexer-cli submit --sitemap https://example.com/sitemap.xml \
  --filter "^https://example.com/blog/\d{4}/\d{2}/"

# 排除某些模式
indexer-cli submit --sitemap https://example.com/sitemap.xml \
  --filter "^https://example.com/(?!admin|private)"

# 基于日期的过滤
indexer-cli submit --sitemap https://example.com/sitemap.xml \
  --since 2024-01-01
```

### 自定义重试策略

在 `config.yaml` 中配置重试行为：

```yaml
retry:
  enabled: true
  max_attempts: 5        # 最多尝试 5 次
  backoff_factor: 2      # 每次延迟加倍
  max_wait_seconds: 120  # 重试之间最多 2 分钟
```

### 日志配置

控制日志输出：

```bash
# 详细输出
indexer-cli --verbose submit https://example.com/page1

# 静默模式（仅错误）
indexer-cli --quiet submit https://example.com/page1

# 配置中的调试日志
logging:
  level: debug
  file: ~/.indexer-cli/debug.log
```

### 数据库管理

历史数据库默认存储在 `~/.indexer-cli/history.db`。

#### 备份数据库

```bash
cp ~/.indexer-cli/history.db ~/.indexer-cli/history-backup.db
```

#### 导出所有历史

```bash
indexer-cli history export --output all-history.csv
```

#### 清理旧记录

```bash
# 删除 180 天前的记录
indexer-cli history clean --older-than 180 --yes
```

### Cron 任务

设置自动提交：

```bash
# 添加到 crontab (crontab -e)

# 每天凌晨 3 点提交站点地图
0 3 * * * /usr/local/bin/indexer-cli submit --sitemap https://example.com/sitemap.xml

# 每月清理历史
0 0 1 * * /usr/local/bin/indexer-cli history clean --older-than 180 --yes
```

### CI/CD 集成

在 CI/CD 管道中使用：

```yaml
# GitHub Actions 示例
- name: 提交 URL 到搜索引擎
  run: |
    indexer-cli submit --file changed-urls.txt --format json
```

### 模拟运行模式

在不实际提交的情况下测试：

```bash
# 查看将要提交的内容
indexer-cli submit --sitemap https://example.com/sitemap.xml --dry-run
```

## 架构

### 高层概览

```
┌─────────────────┐
│   CLI 层        │  (args.rs, handler.rs)
│   基于 clap     │
└────────┬────────┘
         │
┌────────▼──────────────────────────────────────┐
│           命令层                               │
│  (init, config, google, indexnow, submit...)  │
└────────┬───────────────────────┬───────────────┘
         │                       │
┌────────▼─────────┐    ┌────────▼────────────┐
│  服务层          │    │   数据库层          │
│  - 批量提交      │    │   - 模式            │
│  - 站点地图解析  │    │   - 查询            │
│  - 历史管理      │    │   - 模型            │
│  - URL 处理器    │    │                     │
└────────┬─────────┘    └─────────────────────┘
         │
┌────────▼─────────────────────┐
│       API 客户端             │
│  - Google Indexing (OAuth2)  │
│  - IndexNow (HTTP)           │
└──────────────────────────────┘
```

### 模块组织

- **cli/**：命令行界面和参数解析
- **commands/**：命令实现
- **api/**：外部 API 客户端实现
- **services/**：业务逻辑和编排
- **database/**：SQLite 模式、模型和查询
- **config/**：配置加载和验证
- **types/**：共享类型和错误定义
- **utils/**：辅助工具（重试、日志记录、验证）
- **constants.rs**：应用程序范围的常量

### 数据流

1. **用户输入** → CLI 参数解析
2. **配置加载** → 合并配置文件 + 环境 + 默认值
3. **API 客户端初始化** → OAuth2 或 API 密钥身份验证
4. **URL 收集** → 从参数、文件或站点地图
5. **历史检查** → 过滤掉最近提交的 URL
6. **批处理** → 分批，并发提交
7. **结果记录** → 保存到 SQLite 数据库
8. **输出格式化** → 显示结果（文本、JSON、CSV）

### 错误处理

项目使用自定义错误类型（`IndexerError`），其变体包括：
- 配置错误
- API 错误（Google、IndexNow）
- 数据库错误
- HTTP 错误
- 验证错误

错误支持：
- 重试检测（`.is_retryable()`）
- 详细的上下文和错误链
- 用户友好的错误消息

### 异步/并发模型

- 基于 **tokio** 异步运行时构建
- 使用 **futures** 流进行并发批处理
- 使用令牌桶算法进行速率限制
- 可配置的并发级别

### 数据库模式

```sql
CREATE TABLE submission_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    url TEXT NOT NULL,
    api TEXT NOT NULL,           -- 'google' 或 'indexnow'
    action TEXT NOT NULL,         -- 'url-updated' 或 'url-deleted'
    status TEXT NOT NULL,         -- 'success' 或 'failed'
    response_code INTEGER,
    response_message TEXT,
    submitted_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    metadata TEXT
);

-- 高效查询的索引
CREATE INDEX idx_url ON submission_history(url);
CREATE INDEX idx_api ON submission_history(api);
CREATE INDEX idx_status ON submission_history(status);
CREATE INDEX idx_submitted_at ON submission_history(submitted_at);
```

有关更多详细信息，请参阅 [docs/ARCHITECTURE.md](/Users/danielhu/Projects/indexer-cli/docs/ARCHITECTURE.md)。

## 开发

### 设置开发环境

```bash
# 克隆仓库
git clone https://github.com/your-username/indexer-cli.git
cd indexer-cli

# 安装依赖
cargo build

# 运行测试
cargo test

# 使用调试日志运行
RUST_LOG=debug cargo run -- submit https://example.com
```

### 项目结构

```
indexer-cli/
├── src/
│   ├── main.rs              # 入口点
│   ├── lib.rs               # 库导出
│   ├── cli/                 # CLI 接口
│   ├── commands/            # 命令实现
│   ├── api/                 # API 客户端
│   ├── services/            # 业务逻辑
│   ├── database/            # 数据库层
│   ├── config/              # 配置
│   ├── types/               # 类型定义
│   ├── utils/               # 工具
│   └── constants.rs         # 常量
├── tests/                   # 集成测试
├── examples/                # 使用示例
└── docs/                    # 文档
```

### 运行测试

```bash
# 所有测试
cargo test

# 仅单元测试
cargo test --lib

# 仅集成测试
cargo test --test '*'

# 特定测试
cargo test test_sitemap_parser

# 带输出
cargo test -- --nocapture
```

### 构建文档

```bash
# 构建 API 文档
cargo doc --no-deps --open

# 构建所有功能
cargo doc --all-features --open
```

### 代码风格

项目遵循标准的 Rust 约定：

- 使用 `rustfmt` 进行格式化：`cargo fmt`
- 使用 `clippy` 进行 lint：`cargo clippy`
- 提交前运行检查：`cargo fmt && cargo clippy && cargo test`

### 贡献

有关详细的贡献指南，请参阅 [CONTRIBUTING.md](/Users/danielhu/Projects/indexer-cli/CONTRIBUTING.md)。

## 故障排除

### 常见问题

#### "Configuration file not found"（未找到配置文件）

创建配置文件：
```bash
indexer-cli init
```

#### "Google service account not found"（未找到 Google 服务账号）

设置您的 Google 凭据：
```bash
indexer-cli google setup --service-account /path/to/service-account.json
```

#### "IndexNow key file not accessible"（IndexNow 密钥文件无法访问）

验证您的密钥文件是否公开可访问：
```bash
curl https://yourdomain.com/your-api-key.txt
```

#### "Rate limit exceeded"（超出速率限制）

该工具会自动遵守 API 速率限制。如果看到此错误：
- 等待速率限制窗口重置
- 在配置中减少批量大小
- 检查您的配额使用情况：`indexer-cli google quota`

#### "Permission denied"（权限被拒绝）（Google API）

确保您的服务账号在 Google Search Console 中具有域的 Owner 权限。

#### "Database locked"（数据库已锁定）

可能有另一个实例正在运行。关闭其他实例或等待操作完成。

### 调试模式

启用详细日志记录：

```bash
# 详细 CLI 输出
indexer-cli --verbose submit URL

# 调试日志
RUST_LOG=debug indexer-cli submit URL

# 跟踪日志（非常详细）
RUST_LOG=trace indexer-cli submit URL
```

### 日志文件

检查日志文件以获取详细信息：

```bash
# 默认日志位置
tail -f ~/.indexer-cli/indexer.log

# 查看最近的错误
grep ERROR ~/.indexer-cli/indexer.log
```

### 验证配置

```bash
# 检查配置语法
indexer-cli config validate

# 显示当前配置
indexer-cli config list

# 测试 API 连接
indexer-cli validate
```

### 获取帮助

如果遇到此处未涵盖的问题：

1. 查看[文档](docs/)
2. 搜索[现有问题](https://github.com/your-username/indexer-cli/issues)
3. 打开[新问题](https://github.com/your-username/indexer-cli/issues/new)并提供：
   - 运行的命令
   - 错误消息
   - 配置（已清理敏感信息）
   - 调试日志输出

## 常见问题

### 一般问题

**问：Google Indexing API 和 IndexNow 有什么区别？**

答：Google Indexing API 专门用于 Google 搜索，需要使用服务账号进行 OAuth2 身份验证。它有严格的配额限制（每天 200 个 URL），但提供详细的状态信息。IndexNow 是一个开放协议，受多个搜索引擎（Bing、Yandex、Seznam、Naver）支持，限制更高（每批 10,000 个 URL），但只需要 API 密钥。

**问：我可以同时使用两个 API 吗？**

答：可以！`submit` 命令同时提交到所有已配置的 API。您也可以使用特定于 API 的命令（`google submit`、`indexnow submit`）来针对单个服务。

**问：我应该多久提交一次 URL？**

答：仅在 URL 是新的或有重大更新时提交。该工具会跟踪提交历史以防止重复提交。对于定期更新，请使用监视模式或通过 cron 安排提交。

**问：使用这些 API 需要付费吗？**

答：Google Indexing API 和 IndexNow 都可以免费使用。但是，Google 有配额限制（索引 API 每天 200 个 URL）。

**问：我可以向 Google Indexing API 提交什么类型的 URL？**

答：Google Indexing API 主要设计用于招聘信息和直播视频内容。对于一般网站 URL，请使用 Google Search Console 的站点地图提交功能。IndexNow 支持所有类型的内容。

### 技术问题

**问：我的数据存储在哪里？**

答：所有数据都本地存储在 `~/.indexer-cli/` 的 SQLite 数据库中（默认）。配置存储在 `config.yaml` 中，提交历史存储在 `history.db` 中，日志存储在 `indexer.log` 中。

**问：我可以在服务器上运行此工具吗？**

答：可以！该工具设计用于本地和服务器使用。您可以使用 `--daemon` 标志在监视模式下运行它，或通过 cron 任务安排它。确保设置适当的日志记录和监控。

**问：如何备份我的提交历史？**

答：使用 `indexer-cli history export --output backup.csv` 将历史导出为 CSV 或 JSON。您也可以直接复制 `~/.indexer-cli/history.db` 的 SQLite 数据库。

**问：我可以提交多个域吗？**

答：可以，但每个域需要单独的配置。对于 Google API，每个域必须在 Search Console 中验证，服务账号添加为所有者。对于 IndexNow，您需要在每个域上托管 API 密钥文件。

**问：重试机制如何工作？**

答：该工具使用指数退避进行重试。它将重试失败的请求最多 3 次（可配置），延迟递增（2x、4x、8x 等）。仅重试暂时性错误；永久性错误（如身份验证失败）不会重试。

**问：如果超出速率限制会发生什么？**

答：该工具会自动遵守速率限制，并在达到限制时暂停提交。对于 Google API，它会跟踪配额使用情况并防止超出每日限制。

**问：我可以在提交之前过滤 URL 吗？**

答：可以！使用 `--filter` 的正则表达式模式，使用 `--since` 的日期过滤器，或两者结合。站点地图解析器还支持按优先级和修改日期过滤。

**问：服务账号密钥是否安全？**

答：服务账号 JSON 文件包含凭据，应保密。设置适当的文件权限（`chmod 600`），切勿将其提交到版本控制。将其存储在安全位置，如 `~/.indexer-cli/`。

### 使用问题

**问：如何在不实际提交的情况下测试？**

答：对任何提交命令使用 `--dry-run` 标志，以查看将要提交的内容而不实际进行 API 调用。

**问：我可以一次从多个站点地图提交 URL 吗？**

答：可以，您可以通过工具管道多个站点地图或创建脚本来迭代它们：
```bash
cat sitemap-list.txt | while read sitemap; do
  indexer-cli submit --sitemap "$sitemap"
done
```

**问：如何监控长时间运行的提交？**

答：该工具为批量提交显示进度条。对于监视模式，检查 `~/.indexer-cli/indexer.log` 的日志文件或使用 `--verbose` 标志运行以获得详细输出。

**问：我的 URL 文件应该是什么格式？**

答：每行一个 URL，纯文本格式。空行和以 `#` 开头的行将被忽略。

**问：我可以自定义批量大小吗？**

答：可以，使用 `--batch-size N` 标志或在配置文件中的 `google.batch_size` 或 `indexnow.batch_size` 下设置。

## 对比

### 与类似工具对比

| 功能 | indexer-cli | Google Search Console | 手动提交 | 其他 CLI 工具 |
|------|-------------|----------------------|----------|---------------|
| **Google Indexing API** | ✅ 完全支持 | ✅ 通过 UI | ❌ 否 | ⚠️ 有限 |
| **IndexNow 协议** | ✅ 完全支持 | ❌ 否 | ✅ 手动 | ⚠️ 部分 |
| **批量提交** | ✅ 无限制* | ⚠️ 有限 | ❌ 逐个 | ⚠️ 有限 |
| **站点地图解析** | ✅ 高级 | ✅ 基础 | ❌ 否 | ✅ 基础 |
| **历史跟踪** | ✅ SQLite | ❌ 否 | ❌ 否 | ❌ 否 |
| **重试逻辑** | ✅ 指数退避 | ❌ 否 | ❌ 否 | ⚠️ 基础 |
| **速率限制** | ✅ 自动 | ✅ 强制 | ❌ 否 | ⚠️ 基础 |
| **进度跟踪** | ✅ 进度条 | ❌ 否 | ❌ 否 | ⚠️ 有限 |
| **监视模式** | ✅ 连续 | ❌ 否 | ❌ 否 | ❌ 否 |
| **CLI 自动化** | ✅ 完整 | ❌ 否 | ❌ 否 | ⚠️ 部分 |
| **多引擎** | ✅ Google + Bing + Yandex + 更多 | ❌ 仅 Google | ✅ 多个 | ⚠️ 有限 |
| **开源** | ✅ MIT | ❌ 否 | N/A | ⚠️ 不定 |
| **费用** | ✅ 免费 | ✅ 免费 | ✅ 免费 | ⚠️ 不定 |

*受 API 配额限制

### 为什么选择 indexer-cli？

**对于开发者：**
- 🚀 快速高效的 Rust 实现
- 🔧 通过配置文件完全可定制
- 📦 单个二进制文件，无依赖
- 🔄 轻松集成到 CI/CD 管道
- 🐳 Docker 友好（即将推出）

**对于 SEO 专业人士：**
- 📊 全面的提交跟踪和报告
- 🎯 高级过滤和定位功能
- ⏰ 带监视模式的自动化调度
- 📈 大型网站的批处理
- 🔍 站点地图分析和验证

**对于网站所有者：**
- ✨ 使用交互式向导简单设置
- 📝 清晰的文档和示例
- 🛡️ 内置安全功能（模拟运行、速率限制）
- 💾 本地存储，保护隐私
- 🆓 完全免费和开源

### 使用场景

1. **新内容发布**：发布后立即自动提交新的博客文章或产品
2. **网站迁移**：在网站重组期间快速通知搜索引擎 URL 更改
3. **电子商务**：在添加库存时提交新产品页面
4. **新闻网站**：为时效性内容快速索引
5. **开发者博客**：与静态站点生成器（Hugo、Jekyll 等）集成
6. **SEO 代理**：高效管理多个客户网站
7. **CI/CD 集成**：作为部署管道的一部分自动提交

## 许可证

本项目根据 MIT 许可证授权 - 详见 [LICENSE](LICENSE) 文件。

## 致谢

使用这些优秀的 Rust crate 构建：

- [clap](https://github.com/clap-rs/clap) - 命令行参数解析
- [tokio](https://github.com/tokio-rs/tokio) - 异步运行时
- [reqwest](https://github.com/seanmonstar/reqwest) - HTTP 客户端
- [rusqlite](https://github.com/rusqlite/rusqlite) - SQLite 绑定
- [yup-oauth2](https://github.com/dermesser/yup-oauth2) - OAuth2 身份验证
- [serde](https://github.com/serde-rs/serde) - 序列化框架
- [roxmltree](https://github.com/RazrFalcon/roxmltree) - XML 解析
- [indicatif](https://github.com/console-rs/indicatif) - 进度条
- [tracing](https://github.com/tokio-rs/tracing) - 应用程序日志记录

---

**使用 Rust 精心打造**

如有疑问或需要支持，请在 GitHub 上提出问题。
