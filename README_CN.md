# indexer-cli

[English](README.md) | [简体中文](README_CN.md)

> 一个生产就绪的命令行工具，用于自动化网站索引工作流程，支持 Google Indexing API 和 IndexNow 协议

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Build Status](https://img.shields.io/github/actions/workflow/status/your-username/indexer-cli/ci.yml?branch=master)](https://github.com/your-username/indexer-cli/actions)
[![Crates.io](https://img.shields.io/crates/v/indexer-cli.svg)](https://crates.io/crates/indexer-cli)
[![Downloads](https://img.shields.io/crates/d/indexer-cli.svg)](https://crates.io/crates/indexer-cli)

**indexer-cli** 是一个强大的命令行工具，可自动将 URL 提交到搜索引擎。它无缝集成了 Google Indexing API 和 IndexNow 协议，帮助您更快、更高效地让内容被索引。

## 特性

- **Google Indexing API**：服务账号认证、URL 提交、状态检查、速率限制
- **IndexNow 协议**：多搜索引擎支持（Bing、Yandex、Seznam、Naver）、批量提交
- **站点地图处理**：解析 XML 站点地图、递归遍历、URL 过滤
- **提交历史跟踪**：SQLite 数据库用于提交记录和去重
- **高级功能**：并发批处理、重试逻辑、监视模式、模拟运行测试

## 安装

### 从源代码安装

```bash
git clone https://github.com/your-username/indexer-cli.git
cd indexer-cli
cargo build --release
```

二进制文件将位于 `target/release/indexer-cli`。

### 全局安装

```bash
cargo install --path .
```

## 快速开始

### 1. 初始化配置

```bash
indexer-cli init
```

在 `~/.indexer-cli/config.yaml` 创建默认设置。

### 2. 配置 API

**Google**（服务账号 JSON 密钥）：
```bash
indexer-cli google setup --service-account /path/to/service-account.json
```

**IndexNow**（生成 API 密钥）：
```bash
indexer-cli indexnow generate-key --length 32 --save
```

详细指南：[Google 设置](docs/GOOGLE_SETUP_CN.md) | [IndexNow 设置](docs/INDEXNOW_SETUP_CN.md)

### 3. 提交 URL

```bash
# 提交到所有已配置的 API
indexer-cli submit https://your-site.com/page

# 提交到特定 API
indexer-cli google submit https://your-site.com/page
indexer-cli indexnow submit https://your-site.com/page
```

### 4. 从站点地图提交

```bash
indexer-cli sitemap parse https://your-site.com/sitemap.xml | \
  indexer-cli submit --file -
```

## 文档

- [Google 设置指南](docs/GOOGLE_SETUP_CN.md) - 完整的 Google 服务账号设置
- [IndexNow 设置指南](docs/INDEXNOW_SETUP_CN.md) - IndexNow API 密钥配置
- [配置](docs/CONFIGURATION_CN.md) - 配置文件格式和选项
- [使用指南](docs/USAGE_CN.md) - 详细的命令使用示例
- [高级用法](docs/ADVANCED_USAGE_CN.md) - 批处理、过滤、自动化
- [架构](docs/ARCHITECTURE.md) - 技术架构详情
- [故障排除](docs/TROUBLESHOOTING_CN.md) - 常见问题和解决方案
- [FAQ](docs/FAQ_CN.md) - 常见问题解答
- [对比](docs/COMPARISON_CN.md) - 与其他工具对比
- [开发](docs/DEVELOPMENT_CN.md) - 开发环境搭建和贡献

## 配置

配置文件位置：
- **全局**：`~/.indexer-cli/config.yaml`
- **项目**：`./.indexer-cli/config.yaml`
- **自定义**：使用 `--config /path/to/config.yaml`

基本配置：
```yaml
google:
  enabled: true
  service_account_file: ~/.indexer-cli/service-account.json

indexnow:
  enabled: true
  api_key: your-api-key-here
  key_location: https://your-site.com/api-key.txt

history:
  enabled: true
  database_path: ~/.indexer-cli/history.db
```

查看[配置指南](docs/CONFIGURATION_CN.md)了解所有选项。

## 命令概览

```bash
# 初始化配置
indexer-cli init

# Google API
indexer-cli google submit <url>
indexer-cli google status <url>
indexer-cli google quota

# IndexNow API
indexer-cli indexnow submit <url>
indexer-cli indexnow verify

# 统一提交
indexer-cli submit <url>
indexer-cli submit --sitemap <sitemap.xml>

# 站点地图操作
indexer-cli sitemap parse <sitemap.xml>
indexer-cli sitemap list <sitemap.xml>

# 历史管理
indexer-cli history list
indexer-cli history stats

# 验证
indexer-cli validate

# 监视模式
indexer-cli watch --sitemap <sitemap.xml>
```

运行 `indexer-cli --help` 或 `indexer-cli <command> --help` 查看详细信息。

## 示例

### 带过滤的站点地图提交

```bash
# 仅提交最近 30 天的博客文章
indexer-cli submit --sitemap https://your-site.com/sitemap.xml \
  --filter "^https://your-site.com/blog/" \
  --since $(date -d "30 days ago" +%Y-%m-%d)
```

### 使用 Cron 自动化

```bash
# 每天凌晨 3 点提交站点地图
0 3 * * * /usr/local/bin/indexer-cli submit --sitemap https://your-site.com/sitemap.xml

# 每月清理历史记录
0 0 1 * * /usr/local/bin/indexer-cli history clean --older-than 180 --yes
```

### CI/CD 集成

```yaml
# GitHub Actions 示例
- name: 提交 URL 到搜索引擎
  run: |
    indexer-cli submit --file changed-urls.txt --format json
```

查看[使用指南](docs/USAGE_CN.md)获取更多示例。

## 许可证

本项目根据 MIT 许可证授权 - 详见 [LICENSE](LICENSE) 文件。
