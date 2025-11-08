# Indexer CLI - 需求文档

## 项目概述

Indexer CLI 是一个开源命令行工具，用于替代 IndexGuru 等商业索引服务。该工具集成 Google Indexing API 和 IndexNow API，帮助网站管理员自动化地将网站 URL 提交给搜索引擎，加速索引过程。

## 背景与动机

### IndexGuru 的问题
- 收费服务（起价 $9/月）
- API Key 生成存在问题（生成不符合规范的短 key）
- 功能有限，缺乏灵活性
- 闭源，无法自定义

### 本工具的优势
- 完全免费开源
- 直接使用官方 API，无中间层
- 灵活的命令行界面
- 支持自动化脚本集成
- 本地配置和历史记录管理

## 核心功能

### 1. Google Indexing API 集成

#### 功能描述
集成 Google Indexing API，支持提交 JobPosting 和 BroadcastEvent 类型的页面进行索引。

#### 技术细节
- **认证方式**: Service Account (JSON key file)
- **API 配额**:
  - 默认每天 200 个 publish 请求
  - 每分钟 180 个 getMetadata 请求
  - 每分钟 380 个请求（所有端点）
- **支持操作**:
  - `URL_UPDATED`: 通知 Google 某个 URL 已添加或更新
  - `URL_DELETED`: 通知 Google 某个 URL 已删除

#### API Endpoint
```
POST https://indexing.googleapis.com/v3/urlNotifications:publish
```

#### 请求格式
```json
{
  "url": "https://example.com/page.html",
  "type": "URL_UPDATED"
}
```

#### 响应格式
```json
{
  "urlNotificationMetadata": {
    "url": "https://example.com/page.html",
    "latestUpdate": {
      "url": "https://example.com/page.html",
      "type": "URL_UPDATED",
      "notifyTime": "2025-11-08T12:00:00Z"
    }
  }
}
```

#### 设置步骤
1. 创建 Google Cloud 项目
2. 启用 Indexing API
3. 创建 Service Account
4. 下载 JSON key 文件
5. 在 Google Search Console 中添加 Service Account 为所有者

### 2. IndexNow API 集成

#### 功能描述
集成 IndexNow API，一次提交即可通知 Bing, Yandex, Seznam, Naver 等多个搜索引擎。

#### 技术细节
- **认证方式**: API Key (8-128 字符)
- **API 配额**:
  - 完全免费
  - 单次最多提交 10,000 个 URLs
  - 无每日限制
- **支持的搜索引擎**:
  - Microsoft Bing
  - Yandex
  - Seznam.cz
  - Naver

#### API Endpoint
```
POST https://api.indexnow.org/indexnow
POST https://www.bing.com/indexnow
POST https://yandex.com/indexnow
```

#### 请求格式（单个 URL）
```
GET https://api.indexnow.org/indexnow?url={url}&key={key}
```

#### 请求格式（批量 URLs）
```json
{
  "host": "example.com",
  "key": "your-api-key",
  "keyLocation": "https://example.com/your-api-key.txt",
  "urlList": [
    "https://example.com/page1",
    "https://example.com/page2",
    "https://example.com/page3"
  ]
}
```

#### 响应码
- `200 OK`: URL 提交成功
- `202 Accepted`: URL 已接收，key 验证中
- `400 Bad Request`: 格式无效
- `403 Forbidden`: Key 无效
- `422 Unprocessable Entity`: URL 不属于该 host 或 key 不匹配
- `429 Too Many Requests`: 请求过多（可能是垃圾邮件）

#### Key 文件要求
- 在网站根目录创建 `{your-key}.txt` 文件
- 文件内容仅包含 API key（无额外空格或字符）
- UTF-8 编码

### 3. Sitemap 解析

#### 功能描述
解析 XML sitemap，提取所有 URLs，支持 sitemap index 文件。

#### 技术细节
- 支持标准 XML sitemap 格式
- 支持 sitemap index（最多 50,000 个 sitemap）
- 单个 sitemap 限制：50,000 URLs 或 50MB（未压缩）
- 支持 gzip 压缩的 sitemap
- 递归解析嵌套的 sitemap index

#### Sitemap 格式
```xml
<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url>
    <loc>https://example.com/page1</loc>
    <lastmod>2025-11-08</lastmod>
    <changefreq>weekly</changefreq>
    <priority>0.8</priority>
  </url>
</urlset>
```

#### Sitemap Index 格式
```xml
<?xml version="1.0" encoding="UTF-8"?>
<sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <sitemap>
    <loc>https://example.com/sitemap1.xml</loc>
    <lastmod>2025-11-08</lastmod>
  </sitemap>
</sitemapindex>
```

#### 过滤功能
- 按 URL 模式过滤（支持正则表达式）
- 按 lastmod 日期过滤（只提交最近更新的）
- 按优先级过滤
- 去重

### 4. 批量提交管理

#### 功能描述
支持批量提交 URLs，自动分批处理，遵守 API 配额限制。

#### 技术细节
- **Google Indexing API**:
  - 支持批量请求（batch request）
  - 自动限速以遵守配额
  - 失败重试机制
- **IndexNow API**:
  - 自动分批（每批最多 10,000 个 URLs）
  - 并发提交到多个搜索引擎
- **进度显示**:
  - 实时显示提交进度
  - 显示成功/失败统计
  - 显示剩余配额

### 5. 历史记录追踪

#### 功能描述
提供一份轻量级的本地提交缓存，用于在提交前进行去重检查，并允许用户快速查看最近的提交结果。目标是实现与 IndexGuru 相同的“避免重复提交”体验，而非完整的数据仓库。

#### 数据存储
- MVP 仅要求存储最近提交过的 URL、API、动作和时间戳，可选择使用 JSON 文件或精简 SQLite 表；高级统计和长时间保留数据移至后续版本。

#### 记录字段（MVP）
```json
{
  "url": "https://example.com/page1",
  "api": "google|indexnow",
  "action": "URL_UPDATED|URL_DELETED",
  "status": "success|failed",
  "submitted_at": "2025-11-08T12:00:00Z"
}
```

#### 查询功能
- MVP：按 URL 或时间范围列出最近提交记录，用于人工确认是否已提交。
- 进阶功能（统计报表、失败原因聚合等）移至 v1.1+ 路线图。

#### 导出功能
- MVP：支持导出最近记录为 CSV/JSON（二选一即可）。
- Excel 导出与批量报表在 v1.1+ 再引入。

### 6. 配置管理

#### 功能描述
支持灵活的配置管理，包括全局配置、项目配置和命令行参数。

#### 配置文件位置
- **全局配置**: `~/.indexer-cli/config.yaml`
- **项目配置**: `./indexer.yaml` 或 `./.indexer.yaml`
- **环境变量**: `INDEXER_*`

#### 配置优先级
```
命令行参数 > 环境变量 > 项目配置 > 全局配置 > 默认值
```

#### 配置文件格式 (YAML)
```yaml
# Google Indexing API 配置
google:
  enabled: true
  service_account_file: "/path/to/service-account.json"
  quota:
    daily_limit: 200
    rate_limit: 380  # 每分钟请求数
  batch_size: 100  # 批量请求大小

# IndexNow API 配置
indexnow:
  enabled: true
  api_key: "your-32-character-api-key"
  key_location: "https://example.com/your-key.txt"
  endpoints:
    - "https://api.indexnow.org/indexnow"
    - "https://www.bing.com/indexnow"
    - "https://yandex.com/indexnow"
  batch_size: 10000

# Sitemap 配置
sitemap:
  url: "https://example.com/sitemap.xml"
  follow_index: true  # 是否跟踪 sitemap index
  filters:
    url_pattern: ".*"  # 正则表达式
    lastmod_after: "2025-01-01"  # 只提交此日期后的 URLs
    priority_min: 0.5  # 最小优先级

# 历史记录配置
history:
  enabled: true
  database_path: "~/.indexer-cli/history.db"
  retention_days: 365  # 保留天数

# 日志配置
logging:
  level: "info"  # debug, info, warn, error
  file: "~/.indexer-cli/indexer.log"
  max_size_mb: 10
  max_backups: 5

# 重试配置
retry:
  enabled: true
  max_attempts: 3
  backoff_factor: 2  # 指数退避因子
  max_wait_seconds: 60

# 输出配置
output:
  format: "text"  # text, json, csv
  color: true
  verbose: false
```

## 命令行接口设计

### 命令结构
```
indexer [global-options] <command> [command-options] [arguments]
```

### 全局选项
```
-c, --config <file>       指定配置文件
-v, --verbose             详细输出
-q, --quiet               静默模式
--no-color                禁用彩色输出
-h, --help                显示帮助
--version                 显示版本
```

### 命令列表

#### 1. `init` - 初始化配置
```bash
indexer init

# 交互式配置向导
# - 选择要启用的 API (Google/IndexNow/Both)
# - 配置 Google Service Account
# - 生成/配置 IndexNow API key
# - 设置 sitemap URL
# - 生成配置文件
```

**选项**:
```
--global              创建全局配置
--force               覆盖现有配置
```

#### 2. `config` - 管理配置
```bash
# 查看当前配置
indexer config list

# 设置配置项
indexer config set google.enabled true
indexer config set indexnow.api_key "your-key"

# 获取配置项
indexer config get google.service_account_file

# 验证配置
indexer config validate

# 查看配置文件位置
indexer config path
```

#### 3. `google` - Google Indexing API 操作

##### 3.1 设置
```bash
# 设置 Service Account
indexer google setup --service-account /path/to/key.json

# 验证配置
indexer google verify

# 查看配额
indexer google quota
```

##### 3.2 提交 URL
```bash
# 提交单个 URL
indexer google submit https://example.com/page1

# 提交多个 URLs
indexer google submit https://example.com/page1 https://example.com/page2

# 从文件读取 URLs
indexer google submit --file urls.txt

# 从 sitemap 提取并提交
indexer google submit --sitemap https://example.com/sitemap.xml

# 指定操作类型
indexer google submit https://example.com/page1 --type URL_UPDATED
indexer google submit https://example.com/page1 --type URL_DELETED
```

**选项**:
```
--type <type>         操作类型 (URL_UPDATED|URL_DELETED)
--file <file>         从文件读取 URLs
--sitemap <url>       从 sitemap 读取 URLs
--filter <pattern>    URL 过滤模式（正则）
--since <date>        只提交此日期后更新的 URLs
--dry-run             模拟运行，不实际提交
--batch-size <n>      批量大小（默认 100）
--skip-history        跳过历史记录检查
```

##### 3.3 查询状态
```bash
# 查询单个 URL 的索引状态
indexer google status https://example.com/page1

# 批量查询
indexer google status --file urls.txt
```

#### 4. `indexnow` - IndexNow API 操作

##### 4.1 设置
```bash
# 生成 API key
indexer indexnow generate-key

# 生成 key 文件
indexer indexnow generate-key --output ./public/

# 设置 API key
indexer indexnow setup --key your-api-key

# 验证配置
indexer indexnow verify
```

**选项**:
```
--key <key>           API key
--key-location <url>  Key 文件位置
--length <n>          生成的 key 长度（8-128，推荐 32）
--output <dir>        Key 文件输出目录
```

##### 4.2 提交 URL
```bash
# 提交单个 URL
indexer indexnow submit https://example.com/page1

# 提交多个 URLs
indexer indexnow submit https://example.com/page1 https://example.com/page2

# 从文件读取 URLs
indexer indexnow submit --file urls.txt

# 从 sitemap 提取并提交
indexer indexnow submit --sitemap https://example.com/sitemap.xml

# 指定搜索引擎
indexer indexnow submit --endpoint bing https://example.com/page1
indexer indexnow submit --endpoint yandex https://example.com/page1
indexer indexnow submit --endpoint all https://example.com/page1
```

**选项**:
```
--file <file>         从文件读取 URLs
--sitemap <url>       从 sitemap 读取 URLs
--filter <pattern>    URL 过滤模式（正则）
--since <date>        只提交此日期后更新的 URLs
--dry-run             模拟运行，不实际提交
--batch-size <n>      批量大小（默认 10000）
--endpoint <name>     搜索引擎 (bing|yandex|seznam|naver|all)
--skip-history        跳过历史记录检查
```

#### 5. `submit` - 统一提交（同时使用 Google 和 IndexNow）
```bash
# 提交到所有启用的 API
indexer submit https://example.com/page1

# 从 sitemap 提交
indexer submit --sitemap https://example.com/sitemap.xml

# 指定 API
indexer submit --api google https://example.com/page1
indexer submit --api indexnow https://example.com/page1
indexer submit --api all https://example.com/page1
```

**选项**:
```
--api <name>          使用的 API (google|indexnow|all)
--file <file>         从文件读取 URLs
--sitemap <url>       从 sitemap 读取 URLs
--filter <pattern>    URL 过滤模式（正则）
--since <date>        只提交此日期后更新的 URLs
--dry-run             模拟运行，不实际提交
--skip-history        跳过历史记录检查
```

#### 6. `sitemap` - Sitemap 操作
```bash
# 解析 sitemap
indexer sitemap parse https://example.com/sitemap.xml

# 列出所有 URLs
indexer sitemap list https://example.com/sitemap.xml

# 导出 URLs
indexer sitemap export https://example.com/sitemap.xml --output urls.txt

# 统计信息
indexer sitemap stats https://example.com/sitemap.xml

# 验证 sitemap
indexer sitemap validate https://example.com/sitemap.xml
```

**选项**:
```
--output <file>       输出文件
--format <fmt>        输出格式 (text|json|csv)
--filter <pattern>    URL 过滤模式（正则）
--since <date>        只显示此日期后更新的 URLs
--follow-index        跟踪 sitemap index
```

#### 7. `history` - 历史记录管理
```bash
# 查看历史记录
indexer history list

# 按 URL 查询
indexer history search --url https://example.com/page1

# 按日期范围查询
indexer history search --since 2025-11-01 --until 2025-11-08

# 按状态查询
indexer history search --status success
indexer history search --status failed

# 按 API 查询
indexer history search --api google
indexer history search --api indexnow

# 统计报告
indexer history stats

# 导出历史记录
indexer history export --output history.csv
indexer history export --format json --output history.json

# 清理历史记录
indexer history clean --older-than 365  # 删除 365 天前的记录
indexer history clean --all  # 清空所有记录
```

**选项**:
```
--url <url>           按 URL 过滤
--since <date>        开始日期
--until <date>        结束日期
--status <status>     状态 (success|failed)
--api <api>           API 类型 (google|indexnow)
--limit <n>           限制结果数量
--output <file>       输出文件
--format <fmt>        输出格式 (text|json|csv|excel)
--older-than <days>   删除 N 天前的记录
--all                 删除所有记录
```

#### 8. `watch` - 监控模式（自动提交）
```bash
# 监控 sitemap，自动提交新 URLs
indexer watch --sitemap https://example.com/sitemap.xml

# 指定检查间隔（秒）
indexer watch --sitemap https://example.com/sitemap.xml --interval 3600

# 指定 API
indexer watch --sitemap https://example.com/sitemap.xml --api all
```

**选项**:
```
--sitemap <url>       Sitemap URL
--interval <seconds>  检查间隔（默认 3600 秒 = 1 小时）
--api <name>          使用的 API (google|indexnow|all)
--daemon              后台运行
--pid-file <file>     PID 文件路径
```

#### 9. `validate` - 验证工具
```bash
# 验证整体配置
indexer validate

# 验证 Google 配置
indexer validate google

# 验证 IndexNow 配置
indexer validate indexnow

# 验证 IndexNow key 文件可访问性
indexer validate indexnow --check-key-file
```

## 技术栈建议

### 编程语言
- **主要选择**: Rust
- **理由**:
  - 极致的性能和内存安全
  - 编译后的单一二进制文件，无需运行时
  - 优秀的并发和异步支持
  - 强大的类型系统和错误处理
  - Cargo 生态系统完善
  - 轻松实现跨平台编译

### Rust 技术栈 (Cargo.toml)
```toml
[package]
name = "indexer-cli"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <you@example.com>"]
description = "A CLI tool for submitting URLs to Google Indexing API and IndexNow"
license = "MIT"
repository = "https://github.com/yourusername/indexer-cli"

[dependencies]
# CLI 框架
clap = { version = "4.5", features = ["derive", "cargo"] }
dialoguer = "0.11"  # 交互式提示

# HTTP 客户端
reqwest = { version = "0.12", features = ["json", "gzip"] }
tokio = { version = "1.40", features = ["full"] }

# 序列化
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"

# XML 解析
quick-xml = { version = "0.36", features = ["serialize"] }
roxmltree = "0.20"

# 数据库
rusqlite = { version = "0.32", features = ["bundled"] }

# Google API 认证
yup-oauth2 = "11.0"
google-indexing3 = "6.0"

# 日期时间
chrono = { version = "0.4", features = ["serde"] }

# 错误处理
anyhow = "1.0"
thiserror = "1.0"

# 日志
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# 工具库
url = "2.5"
regex = "1.11"
rand = "0.8"
sha2 = "0.10"

# 进度条和美化输出
indicatif = { version = "0.17", features = ["rayon"] }
console = "0.15"
colored = "2.1"

# 配置和环境变量
config = "0.14"
dotenvy = "0.15"

# 并发控制
futures = "0.3"
async-trait = "0.1"

[dev-dependencies]
# 测试框架
mockito = "1.5"
tempfile = "3.13"
wiremock = "0.6"

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true  # 减小二进制文件大小
```

## 项目结构

### Rust 标准项目结构
```
indexer-cli/
├── src/
│   ├── main.rs                 # 程序入口
│   ├── lib.rs                  # 库根模块
│   │
│   ├── cli/                    # CLI 相关
│   │   ├── mod.rs
│   │   ├── args.rs             # 命令行参数定义
│   │   └── handler.rs          # 命令处理器
│   │
│   ├── commands/               # CLI 命令实现
│   │   ├── mod.rs
│   │   ├── init.rs
│   │   ├── config.rs
│   │   ├── google.rs
│   │   ├── indexnow.rs
│   │   ├── submit.rs
│   │   ├── sitemap.rs
│   │   ├── history.rs
│   │   ├── watch.rs
│   │   └── validate.rs
│   │
│   ├── api/                    # API 客户端
│   │   ├── mod.rs
│   │   ├── google_indexing.rs  # Google Indexing API
│   │   └── indexnow.rs         # IndexNow API
│   │
│   ├── services/               # 业务逻辑服务
│   │   ├── mod.rs
│   │   ├── sitemap_parser.rs   # Sitemap 解析
│   │   ├── url_processor.rs    # URL 处理
│   │   ├── batch_submitter.rs  # 批量提交
│   │   └── history_manager.rs  # 历史记录管理
│   │
│   ├── database/               # 数据库
│   │   ├── mod.rs
│   │   ├── schema.rs           # 数据库 schema
│   │   ├── models.rs           # 数据模型
│   │   └── queries.rs          # SQL 查询
│   │
│   ├── config/                 # 配置管理
│   │   ├── mod.rs
│   │   ├── loader.rs           # 配置加载
│   │   ├── settings.rs         # 配置结构
│   │   └── validation.rs       # 配置验证
│   │
│   ├── utils/                  # 工具函数
│   │   ├── mod.rs
│   │   ├── logger.rs           # 日志工具
│   │   ├── retry.rs            # 重试逻辑
│   │   ├── validators.rs       # 验证器
│   │   └── file.rs             # 文件操作
│   │
│   ├── types/                  # 类型定义
│   │   ├── mod.rs
│   │   ├── error.rs            # 错误类型
│   │   └── result.rs           # Result 类型别名
│   │
│   └── constants.rs            # 常量定义
│
├── tests/                      # 集成测试
│   ├── integration_test.rs
│   ├── api_tests.rs
│   └── fixtures/
│       ├── mod.rs
│       └── sample_sitemap.xml
│
├── benches/                    # 性能基准测试
│   └── benchmark.rs
│
├── examples/                   # 示例代码
│   ├── basic_usage.rs
│   └── advanced_config.rs
│
├── docs/                       # 文档
│   ├── getting-started.md
│   ├── configuration.md
│   ├── api-reference.md
│   └── examples.md
│
├── .cargo/
│   └── config.toml            # Cargo 配置
│
├── Cargo.toml                  # 项目依赖和配置
├── Cargo.lock                  # 依赖锁定文件
├── .env.example                # 环境变量示例
├── .gitignore
├── LICENSE
├── README.md
├── CHANGELOG.md
└── rust-toolchain.toml         # Rust 工具链版本
```

## 数据库设计

### SQLite Schema
```sql
-- 历史记录表
CREATE TABLE submission_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    url TEXT NOT NULL,
    api TEXT NOT NULL,           -- 'google' or 'indexnow'
    action TEXT NOT NULL,        -- 'URL_UPDATED', 'URL_DELETED'
    status TEXT NOT NULL,        -- 'success', 'failed'
    response_code INTEGER,
    response_message TEXT,
    submitted_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    metadata JSON,
    INDEX idx_url (url),
    INDEX idx_api (api),
    INDEX idx_status (status),
    INDEX idx_submitted_at (submitted_at)
);

-- 配置表（可选，用于存储敏感信息）
CREATE TABLE config_store (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    encrypted BOOLEAN DEFAULT 0,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 队列表（用于 watch 模式）
CREATE TABLE submission_queue (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    url TEXT NOT NULL,
    api TEXT NOT NULL,
    action TEXT NOT NULL,
    priority INTEGER DEFAULT 0,
    attempts INTEGER DEFAULT 0,
    max_attempts INTEGER DEFAULT 3,
    next_retry TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_next_retry (next_retry),
    INDEX idx_priority (priority)
);
```

## 技术实现

详细的技术实现文档（包括代码示例）请参考：`indexer-cli-implementation.md`

核心模块包括：
1. **Google Indexing API 客户端** - OAuth2 认证和 API 调用
2. **IndexNow API 客户端** - 多端点并发提交
3. **Sitemap 解析器** - XML 解析和递归处理
4. **批量提交管理器** - 并发控制和进度追踪
5. **历史记录管理器** - SQLite 数据库操作
6. **配置管理器** - YAML/环境变量/命令行参数处理
7. **CLI 框架** - 基于 clap 的命令行界面

## 使用示例

### 示例 1: 初始化配置
```bash
# 交互式初始化
indexer init

# 输出:
# ? Enable Google Indexing API? Yes
# ? Path to Google Service Account JSON: /path/to/service-account.json
# ? Enable IndexNow API? Yes
# ? IndexNow API Key (leave empty to generate):
# Generated API key: a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6
# ? Sitemap URL: https://example.com/sitemap.xml
#
# Configuration saved to: /Users/danielhu/.indexer-cli/config.yaml
#
# Next steps:
# 1. Upload the IndexNow key file to your website:
#    File: a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6.txt
#    Content: a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6
#    Location: https://example.com/a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6.txt
#
# 2. Add the Google Service Account as owner in Google Search Console:
#    Email: indexer@project-123456.iam.gserviceaccount.com
#
# Run 'indexer validate' to verify your configuration.
```

### 示例 2: 从 Sitemap 提交
```bash
# 提交到所有 API
indexer submit --sitemap https://example.com/sitemap.xml

# 输出:
# ⠋ Parsing sitemap...
# ✓ Found 1,234 URLs in sitemap
# ⠋ Checking submission history...
# ✓ 234 URLs already submitted (skipped)
# ⠋ Submitting 1,000 URLs to Google Indexing API...
# ⠋ Progress: 100/1000
# ⠋ Progress: 200/1000
# ...
# ✓ Google Indexing API: 995 success, 5 failed
# ⠋ Submitting 1,000 URLs to IndexNow...
# ✓ IndexNow: 1,000 success, 0 failed
#
# Summary:
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Total URLs:        1,234
# Skipped:             234
# Submitted:         1,000
#
# Google API:
#   Success:           995
#   Failed:              5
#   Success Rate:    99.5%
#
# IndexNow:
#   Success:         1,000
#   Failed:              0
#   Success Rate:      100%
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### 示例 3: 监控模式
```bash
# 后台监控 sitemap，自动提交新 URLs
indexer watch --sitemap https://example.com/sitemap.xml --daemon

# 输出:
# ✓ Started monitoring sitemap: https://example.com/sitemap.xml
# ✓ Check interval: 3600 seconds (1 hour)
# ✓ PID: 12345
# ✓ Log file: /Users/danielhu/.indexer-cli/indexer.log
#
# Run 'tail -f /Users/danielhu/.indexer-cli/indexer.log' to view logs
# Run 'kill 12345' to stop monitoring
```

### 示例 4: 查看历史记录
```bash
# 查看统计
indexer history stats

# 输出:
# Submission Statistics
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Total Submissions:     5,234
# Successful:            5,189
# Failed:                   45
# Success Rate:          99.14%
#
# By API:
#   Google:              2,500
#   IndexNow:            2,734
#
# Last 7 days:           1,234
# Last 30 days:          3,456
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

# 查询失败的提交
indexer history search --status failed --limit 10

# 输出:
# URL                                    API      Status  Code  Message                   Date
# ─────────────────────────────────────────────────────────────────────────────────────────────
# https://example.com/page1              google   failed  403   Forbidden                 2025-11-08 10:30:15
# https://example.com/page2              google   failed  429   Too Many Requests         2025-11-08 10:31:20
# ...
```

### 示例 5: 生成 IndexNow Key
```bash
# 生成 key 并创建文件
indexer indexnow generate-key --output ./public/

# 输出:
# Generated IndexNow API Key: a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6
#
# ✓ Created key file: ./public/a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6.txt
#
# Next steps:
# 1. Upload this file to your website root directory
# 2. Verify the file is accessible at:
#    https://yoursite.com/a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6.txt
# 3. Run: indexer config set indexnow.api_key a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6
# 4. Run: indexer validate indexnow
```

## 部署与分发

### Cargo 发布到 crates.io
```bash
# 检查包是否可以发布
cargo publish --dry-run

# 发布到 crates.io
cargo publish

# 用户安装
cargo install indexer-cli

# 从 Git 仓库安装
cargo install --git https://github.com/yourusername/indexer-cli
```

### 预编译二进制发布
```bash
# 为当前平台构建 release 版本
cargo build --release

# 交叉编译到多个平台（使用 cross）
cargo install cross

# Linux x86_64
cross build --release --target x86_64-unknown-linux-gnu

# Linux ARM64
cross build --release --target aarch64-unknown-linux-gnu

# macOS x86_64
cross build --release --target x86_64-apple-darwin

# macOS ARM64 (M1/M2)
cross build --release --target aarch64-apple-darwin

# Windows x86_64
cross build --release --target x86_64-pc-windows-gnu

# 发布到 GitHub Releases
gh release create v1.0.0 \
  ./target/x86_64-unknown-linux-gnu/release/indexer-cli \
  ./target/aarch64-unknown-linux-gnu/release/indexer-cli \
  ./target/x86_64-apple-darwin/release/indexer-cli \
  ./target/aarch64-apple-darwin/release/indexer-cli \
  ./target/x86_64-pc-windows-gnu/release/indexer-cli.exe
```

### 使用 cargo-binstall 支持
```bash
# 添加 binstall 元数据到 Cargo.toml
[package.metadata.binstall]
pkg-url = "{ repo }/releases/download/v{ version }/{ name }-{ target }{ archive-suffix }"
bin-dir = "{ name }-{ target }/{ bin }{ binary-ext }"

# 用户可以使用 binstall 快速安装
cargo binstall indexer-cli
```

### Homebrew 发布（macOS/Linux）
```ruby
# 创建 Homebrew formula: indexer-cli.rb
class IndexerCli < Formula
  desc "A CLI tool for submitting URLs to Google Indexing API and IndexNow"
  homepage "https://github.com/yourusername/indexer-cli"
  url "https://github.com/yourusername/indexer-cli/archive/v1.0.0.tar.gz"
  sha256 "..."

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    system "#{bin}/indexer", "--version"
  end
end

# 用户安装
brew tap yourusername/tap
brew install indexer-cli
```

### Docker 镜像
```dockerfile
# Dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/indexer-cli /usr/local/bin/
ENTRYPOINT ["indexer-cli"]

# 构建和发布
docker build -t indexer-cli:latest .
docker tag indexer-cli:latest yourusername/indexer-cli:v1.0.0
docker push yourusername/indexer-cli:v1.0.0

# 用户使用
docker run --rm -v ~/.indexer-cli:/root/.indexer-cli \
  yourusername/indexer-cli:v1.0.0 submit --sitemap https://example.com/sitemap.xml
```

## 测试策略

### 单元测试
- API 客户端测试（使用 mock）
- Sitemap 解析器测试
- 配置加载器测试
- 历史记录管理器测试

### 集成测试
- Google Indexing API 集成测试（需要 test service account）
- IndexNow API 集成测试（需要 test key）
- Sitemap 解析集成测试（使用真实 sitemap）
- 数据库操作测试

### E2E 测试
- 完整的提交流程测试
- CLI 命令测试
- 配置管理测试

## 文档计划

### README.md
- 项目简介
- 快速开始
- 安装指南
- 基本用法示例

### docs/getting-started.md
- 详细安装步骤
- 配置 Google Indexing API
- 配置 IndexNow API
- 第一次提交

### docs/configuration.md
- 配置文件结构
- 所有配置选项详解
- 环境变量
- 配置优先级

### docs/api-reference.md
- 所有命令详细说明
- 选项和参数
- 示例

### docs/examples.md
- 常见使用场景
- 自动化脚本示例
- 与 CI/CD 集成
- cron job 设置

### docs/troubleshooting.md
- 常见问题
- 错误码说明
- 调试技巧

## 开发路线图

### v1.0.0 - MVP（核心功能）
- [x] 需求分析
- [ ] 项目初始化
- [ ] Google Indexing API 客户端
- [ ] IndexNow API 客户端
- [ ] Sitemap 解析器
- [ ] 基本 CLI 命令（init, submit, config）
- [ ] 历史记录管理
- [ ] 基本文档
- [ ] 单元测试

### v1.1.0 - 增强功能
- [ ] 批量提交优化
- [ ] 进度条和美化输出
- [ ] 重试机制
- [ ] watch 监控模式
- [ ] 历史记录查询和导出
- [ ] 完整文档

### v1.2.0 - 高级功能
- [ ] 配置加密（敏感信息）
- [ ] 多站点管理
- [ ] 插件系统
- [ ] Web UI（可选）
- [ ] 性能优化
- [ ] 国际化支持

### v2.0.0 - 生态系统
- [ ] GitHub Action
- [ ] Docker 支持
- [ ] WordPress 插件
- [ ] Netlify/Vercel 插件
- [ ] 云函数部署
- [ ] 第三方集成

## 许可证

建议使用 MIT License，允许商业和个人使用。

## 贡献指南

- 欢迎贡献代码、文档和建议
- 提交 PR 前请先创建 Issue
- 遵循代码风格指南
- 添加测试
- 更新文档

## 总结

这个 CLI 工具将提供比 IndexGuru 更强大、更灵活的功能：

**核心优势**:
1. 完全免费开源
2. 直接集成官方 API
3. 本地历史记录和配置管理
4. 支持自动化和批量操作
5. 灵活的 CLI 设计
6. 跨平台支持

**技术亮点**:
1. Rust 实现，极致性能和内存安全
2. 单一二进制文件，无需运行时
3. SQLite 本地存储
4. 异步并发和智能限速
5. 重试机制和错误处理
6. 详细的日志和历史记录
7. 跨平台编译和分发
8. 可扩展的模块化架构

**Rust 的优势**:
- 编译后的二进制文件小巧且高效
- 无需安装运行时环境
- 出色的并发性能
- 强大的类型系统确保代码安全
- 丰富的生态系统（crates.io）

这个工具将成为网站管理员和 SEO 专业人员的得力助手，帮助他们更高效地管理搜索引擎索引。
