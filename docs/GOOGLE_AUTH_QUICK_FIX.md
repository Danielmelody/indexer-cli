# Google Service Account 认证错误快速修复指南

## 问题描述

运行 `indexer-cli google verify` 时遇到类似错误：
```
✓ Service account file exists
  Testing authentication...
Error: Google API authentication failed: Not enough private keys in PEM
```

常见原因：
- JSON 文件格式问题（私钥字段缺失或格式错误）
- 下载了 P12 格式而不是 JSON
- JSON 文件损坏或不完整

## 快速修复步骤

### 1. 重新下载 JSON 密钥（推荐）

```bash
# 步骤概览
1. 访问：https://console.cloud.google.com/iam-admin/serviceaccounts
2. 选择正确的项目和你的服务账户
3. 进入 "Keys" 标签页
4. 点击 "ADD KEY" → "Create new key"
5. 重要：选择 "JSON" 格式（不是 P12）
6. 点击 "Create" 下载
```

### 2. 验证下载的文件

```bash
# 检查文件大小（通常 2-3 KB）
ls -lh ~/Downloads/your-key.json

# 验证 JSON 格式
cat ~/Downloads/your-key.json | jq .

# 检查私钥格式（应显示：-----BEGIN PRIVATE KEY-----）
cat ~/Downloads/your-key.json | jq -r '.private_key' | head -1
```

### 3. 重新配置

```bash
# 删除旧配置（可选）
mv indexer.yaml indexer.yaml.backup

# 使用新密钥配置
indexer-cli google setup --service-account ~/Downloads/your-key.json

# 验证配置
indexer-cli google verify
```

## 预期输出

成功时应该看到：
```
✓ Configuration found
✓ Google configuration enabled
✓ Service account file exists
✓ JSON format valid
✓ API authentication successful
✓ API connection successful
```

## 常见错误诊断

### 错误：`Not enough private keys in PEM`
- **原因**：私钥格式错误（PKCS#1 而非 PKCS#8）
- **解决**：重新下载 JSON 密钥，确保选择 JSON 格式

### 错误：`Service account JSON is missing 'client_email' field`
- **原因**：JSON 文件损坏或被编辑
- **解决**：重新下载完整的 JSON 文件，不要手动编辑

### 错误：`Permission denied` (HTTP 403)
- **原因**：服务账户未添加到 Search Console 或权限不足
- **解决**：
  1. 访问：https://search.google.com/search-console
  2. 选择网站属性 → Settings → Users and permissions
  3. 添加服务账户邮箱，权限设为 "Owner"

## 调试技巧

### 启用详细日志
```bash
RUST_LOG=debug indexer-cli google verify
```

### 检查 JSON 必需字段
```bash
# 检查所有字段
jq 'keys' /path/to/service-account.json

# 检查私钥长度（应 > 1000 字符）
jq '.private_key | length' /path/to/service-account.json
```

### 验证服务账户邮箱
```bash
# 应该是 something@project-id.iam.gserviceaccount.com
jq -r '.client_email' /path/to/service-account.json
```

## 完整的 JSON 文件示例

```json
{
  "type": "service_account",
  "project_id": "my-project-123456",
  "private_key_id": "abcdef123456789",
  "private_key": "-----BEGIN PRIVATE KEY-----\nMIIEvQIBADANBgkqhkiG9w0BAQE...\n-----END PRIVATE KEY-----\n",
  "client_email": "indexer-cli@my-project-123456.iam.gserviceaccount.com",
  "client_id": "123456789",
  "auth_uri": "https://accounts.google.com/o/oauth2/auth",
  "token_uri": "https://oauth2.googleapis.com/token"
}
```

**关键点**：
- `private_key` 必须以 `-----BEGIN PRIVATE KEY-----` 开头
- 所有字段都必须存在且不能为空
- 不要手动编辑 JSON 文件

## 手动转换密钥格式（最后手段）

如果实在无法从 Google 重新获取密钥：

```bash
# 提取私钥
cat your-file.json | jq -r '.private_key' > old_key.pem

# 转换 PKCS#1 到 PKCS#8
openssl pkcs8 -topk8 -inform PEM -outform PEM \
  -in old_key.pem -out new_key.pem -nocrypt

# 手动更新 JSON 文件中的 private_key 字段
# 注意：需要将换行符转义为 \n
```

**警告**：手动转换容易出错，强烈建议重新下载新密钥。

## 检查清单

在运行 `verify` 前确认：

- [ ] JSON 文件存在且可读
- [ ] JSON 格式有效（使用 `jq` 验证）
- [ ] 密钥格式为 JSON，不是 P12
- [ ] `private_key` 字段存在且完整
- [ ] `client_email` 字段存在且正确
- [ ] 服务账户已添加到 Search Console
- [ ] Search Console 权限设置为 "Owner"
- [ ] Indexing API 已启用（可能需要等待 3-5 分钟）

## 获取更多帮助

如果问题仍未解决：

1. 运行 `indexer-cli google verify` 获取详细诊断
2. 检查错误消息中的具体提示
3. 在 Google Cloud Console 验证服务账户状态
4. 在 Search Console 验证用户权限
5. 考虑创建全新的服务账户和密钥

## 相关链接

- Google Cloud Console: https://console.cloud.google.com
- Service Accounts: https://console.cloud.google.com/iam-admin/serviceaccounts
- Indexing API: https://console.cloud.google.com/apis/library/indexing.googleapis.com
- Google Search Console: https://search.google.com/search-console
