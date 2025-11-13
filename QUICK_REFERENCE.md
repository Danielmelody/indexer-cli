# Google Service Account 认证错误 - 快速参考

## 快速诊断

遇到 `Not enough private keys in PEM` 错误？按以下步骤快速解决：

### 第一步：运行诊断命令
```bash
indexer-cli google verify
```

系统会自动检查并显示详细的诊断信息。

### 第二步：查看错误提示
如果看到类似这样的消息：
```
Validating JSON format... ✗
Common causes:
  • Private key field is missing or corrupted
  • Downloaded P12 format instead of JSON
  • JSON file is truncated or damaged
```

### 第三步：按照修复步骤
```
Fix:
  1. Delete the current key from Google Cloud Console
  2. Create a new key and download as JSON format
  3. Run: indexer-cli google setup --service-account <path>
```

---

## 最常见的 3 个错误和解决方案

### 错误 1：下载了 P12 格式而不是 JSON

**症状：** "Not enough private keys in PEM"

**解决方案：**
1. 访问 https://console.cloud.google.com/iam-admin/serviceaccounts
2. 删除旧的 P12 密钥
3. 点击 "Add Key" → "Create new key"
4. **关键：** 选择 "JSON" 格式（不是 P12）
5. 重新运行 `indexer-cli google setup --service-account /path/to/new-key.json`

---

### 错误 2：JSON 文件不完整或损坏

**症状：** "private_key field is missing" 或 PEM 错误

**解决方案：**
1. 删除当前的 JSON 文件（可能是下载过程中断）
2. 重新从 Google Cloud Console 下载
3. 验证文件大小通常是 2-3 KB
4. 确保文件内容以 `{` 开头，以 `}` 结尾

---

### 错误 3：权限不足

**症状：** "Permission denied (403)" 或无法提交 URLs

**解决方案：**
1. 访问 https://search.google.com/search-console
2. 选择你的网站属性
3. Settings → Users and permissions → Add user
4. 粘贴服务账户邮箱（格式：something@project-id.iam.gserviceaccount.com）
5. **重要：** 设置权限为 "Owner"（其他权限不起作用）
6. 等待 5-10 分钟生效后重试

---

## 调试技巧

### 启用详细日志
```bash
RUST_LOG=debug indexer-cli google verify
```

会显示：
- 文件读取过程
- 字段验证结果
- 认证器创建状态
- 错误详细信息

### 验证 JSON 文件
```bash
# 检查是否为有效 JSON
jq . /path/to/service-account.json

# 检查必需字段
jq 'keys' /path/to/service-account.json

# 检查 private_key 是否存在且完整
jq '.private_key | length' /path/to/service-account.json
```

---

## 常用命令

```bash
# 设置服务账户
indexer-cli google setup --service-account /path/to/key.json

# 验证配置
indexer-cli google verify

# 查看配额
indexer-cli google quota

# 检查 URL 索引状态
indexer-cli google status --url https://your-site.com/page

# 提交单个 URL
indexer-cli submit --url https://your-site.com/page

# 提交多个 URLs
indexer-cli submit --file urls.txt
```

---

## 完整文档导航

| 需要什么？ | 查看文件 |
|-----------|--------|
| 快速入门 | [GOOGLE_AUTH_FIX.md](./GOOGLE_AUTH_FIX.md) |
| 故障排查指南 | [GOOGLE_AUTH_TROUBLESHOOTING.md](./GOOGLE_AUTH_TROUBLESHOOTING.md) |
| 技术细节 | [CODE_CHANGES_DETAILED.md](./CODE_CHANGES_DETAILED.md) |
| 完整报告 | [GOOGLE_AUTH_COMPLETE_REPORT.md](./GOOGLE_AUTH_COMPLETE_REPORT.md) |

---

## 关键链接

- Google Cloud Console: https://console.cloud.google.com
- Service Accounts: https://console.cloud.google.com/iam-admin/serviceaccounts
- Search Console: https://search.google.com/search-console
- Indexing API: https://console.cloud.google.com/apis/library/indexing.googleapis.com

---

## 需要更多帮助？

1. **检查日志：** 使用 `RUST_LOG=debug` 启用详细日志
2. **验证 JSON：** 使用 `jq` 工具验证 JSON 格式
3. **查看 Google Console：** 确认服务账户配置
4. **查看完整指南：** 阅读 GOOGLE_AUTH_TROUBLESHOOTING.md

---

**记住：** 大多数问题都是由于下载了 P12 格式而不是 JSON。确保始终选择 JSON 格式！
