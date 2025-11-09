# Google Service Account 认证故障排查指南

## 快速诊断流程

### 第 1 步：验证配置
```bash
indexer-cli google verify
```

系统会逐步验证：
- ✓ 配置文件是否存在
- ✓ Google 配置是否启用
- ✓ 认证方式是否为 Service Account
- ✓ 服务账户文件是否存在
- ✓ **JSON 格式是否有效** ← 大多数错误会在这里捕获
- ✓ 认证是否成功
- ✓ API 连接是否正常

## 常见错误和解决方案

### 错误：`Not enough private keys in PEM`

**原因分析：**
这个错误表示 `yup-oauth2` 库无法解析 JSON 文件中的 `private_key` 字段。可能的原因：

#### 1. 下载了 P12 格式而不是 JSON
P12 是二进制密钥格式，不能直接用作 JSON。

**解决方案：**
```bash
# 步骤 1：访问 Google Cloud Console
# https://console.cloud.google.com/iam-admin/serviceaccounts

# 步骤 2：删除旧密钥
#   1. 找到你的服务账户
#   2. 点击账户名称
#   3. 选择 "Keys" 标签页
#   4. 找到旧密钥（可能显示为 ".p12" 或 ".json"）
#   5. 点击三点菜单 → Delete

# 步骤 3：创建新的 JSON 密钥
#   1. 点击 "Add Key" → "Create new key"
#   2. 选择 "Key type" → JSON ← 重要！
#   3. 点击 "Create"
#   4. 文件会自动下载为 .json 格式

# 步骤 4：重新配置
indexer-cli google setup --service-account /path/to/new-key.json
```

#### 2. JSON 文件被截断或损坏
如果下载过程中断或文件被修改，可能导致格式错误。

**诊断方法：**
```bash
# 检查文件大小（通常 2-3 KB）
ls -lh /path/to/service-account.json

# 检查文件内容（应该是有效的 JSON）
cat /path/to/service-account.json | jq .

# 检查 private_key 字段是否存在和完整
cat /path/to/service-account.json | jq .private_key | head -c 100
```

**修复方法：**
- 重新下载密钥（确保完整下载）
- 验证 JSON 有效性：使用在线 JSON 验证工具或 `jq`
- 重新运行 setup 命令

#### 3. 使用了错误的服务账户密钥
可能混淆了多个服务账户或项目的密钥。

**验证方法：**
```bash
# 检查 JSON 中的 client_email
cat /path/to/service-account.json | jq .client_email

# 应该是 something@project-id.iam.gserviceaccount.com 的格式
# 例如：indexer-cli@my-project-123.iam.gserviceaccount.com
```

**修复方法：**
- 确认密钥来自正确的项目
- 确认密钥来自正确的服务账户
- 在 Google Cloud Console 中验证

### 错误：`Service account JSON is missing 'client_email' field`

**原因：**
JSON 文件格式不正确或被编辑了。

**解决方案：**
1. 重新下载完整的 JSON 密钥
2. 不要手动编辑 JSON 文件
3. 确保所有必需字段存在：
   - `type`
   - `project_id`
   - `private_key_id`
   - `private_key`
   - `client_email`
   - `client_id`
   - `auth_uri`
   - `token_uri`

### 错误：`Permission denied` (HTTP 403)

**原因：**
服务账户未被添加到 Google Search Console，或权限不足。

**解决方案：**
```
步骤 1：访问 Google Search Console
https://search.google.com/search-console

步骤 2：选择你的网站属性

步骤 3：导航到 Settings → Users and permissions

步骤 4：点击 "Add user"

步骤 5：粘贴服务账户邮箱地址
格式：something@project-id.iam.gserviceaccount.com

步骤 6：设置权限为 "Owner"
其他权限级别会导致 API 错误！

步骤 7：点击 "Add"

步骤 8：等待 5-10 分钟生效，然后运行 verify
```

**重要：** 权限必须是 `Owner`，其他级别（Editor, Viewer）不起作用。

## 完整的 JSON 文件示例

下载的 JSON 文件应该类似于：
```json
{
  "type": "service_account",
  "project_id": "my-project-123456",
  "private_key_id": "abcdef123456789",
  "private_key": "-----BEGIN PRIVATE KEY-----\nMIIEvQIBADANBgkqhkiG9w0BAQE...\n-----END PRIVATE KEY-----\n",
  "client_email": "indexer-cli@my-project-123456.iam.gserviceaccount.com",
  "client_id": "123456789",
  "auth_uri": "https://accounts.google.com/o/oauth2/auth",
  "token_uri": "https://oauth2.googleapis.com/token",
  "auth_provider_x509_cert_url": "https://www.googleapis.com/oauth2/v1/certs",
  "client_x509_cert_url": "https://www.googleapis.com/robot/v1/metadata/x509/indexer-cli%40my-project-123456.iam.gserviceaccount.com"
}
```

**关键点：**
- `private_key` 必须以 `-----BEGIN PRIVATE KEY-----` 开头
- `private_key` 必须以 `-----END PRIVATE KEY-----\n` 结尾
- 所有字段都必须存在且不能为空

## 调试技巧

### 启用详细日志
```bash
# 使用 debug 日志级别
RUST_LOG=debug indexer-cli google verify

# 输出示例会显示：
# 2024-01-15 10:30:45 DEBUG google_indexing: Reading service account key from: "/path/to/file.json"
# 2024-01-15 10:30:45 DEBUG google_indexing: Service account email: test@project.iam.gserviceaccount.com
```

### 手动验证 JSON
```bash
# 使用 jq 验证格式
jq . /path/to/service-account.json

# 检查必需字段
jq 'keys' /path/to/service-account.json

# 检查 private_key 长度（应该 > 1000 字符）
jq '.private_key | length' /path/to/service-account.json
```

### 检查文件权限
```bash
# 确保文件可读
ls -la /path/to/service-account.json

# 应该显示：
# -rw-r--r-- 1 user group 2500 Jan 15 10:00 service-account.json
```

## 逐步排查清单

- [ ] 确认 JSON 文件存在且可读
- [ ] 确认 JSON 格式有效（使用 `jq` 或在线工具验证）
- [ ] 确认密钥格式为 JSON，不是 P12
- [ ] 确认 `private_key` 字段存在且完整
- [ ] 确认 `client_email` 字段存在且正确
- [ ] 确认服务账户已添加到 Search Console
- [ ] 确认 Search Console 中的权限设置为 "Owner"
- [ ] 运行 `indexer-cli google verify` 测试配置
- [ ] 等待 3-5 分钟后重试（新密钥可能需要时间激活）

## 获取帮助

如果问题仍未解决，请检查：
1. 查看 debug 日志：`RUST_LOG=debug indexer-cli google verify`
2. 在 Google Cloud Console 中检查服务账户状态
3. 在 Google Search Console 中验证用户权限
4. 检查 Indexing API 是否已启用（通常需要 3-5 分钟激活）

## 相关链接

- Google Cloud Console: https://console.cloud.google.com
- Service Accounts: https://console.cloud.google.com/iam-admin/serviceaccounts
- Indexing API: https://console.cloud.google.com/apis/library/indexing.googleapis.com
- Google Search Console: https://search.google.com/search-console
- yup-oauth2 库: https://crates.io/crates/yup-oauth2
