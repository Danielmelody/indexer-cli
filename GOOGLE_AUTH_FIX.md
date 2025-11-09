# Google Service Account 认证错误修复

## 问题描述

用户在运行 `indexer-cli google verify` 时出现错误：
```
✓ Service account file exists
  Testing authentication...
Error: Google API authentication failed: Not enough private keys in PEM
```

这个错误通常是由以下原因引起的：
1. **JSON 文件格式问题** - 私钥字段缺失或格式错误
2. **下载错误的密钥格式** - 下载了 P12 格式而不是 JSON
3. **JSON 文件损坏** - 文件不完整或被截断

## 修复方案

已对以下文件进行了改进，以提供更好的错误诊断和用户指导：

### 1. `/src/api/google_indexing.rs` - 增强验证和错误处理

**改进点：**

#### 更详细的错误消息
```rust
let service_account_key = yup_oauth2::read_service_account_key(&service_account_path)
    .await
    .map_err(|e| {
        let error_msg = e.to_string();
        error!("Failed to read service account key: {}", error_msg);

        // 检测 PEM 解析错误
        if error_msg.contains("Not enough private keys in PEM")
            || error_msg.contains("private_key")
            || error_msg.contains("key") {
            IndexerError::GoogleServiceAccountInvalid {
                message: format!(
                    "Invalid service account JSON: {}. \
                    Ensure the file contains a valid 'private_key' field. \
                    Download the key as JSON (not P12) from Google Cloud Console.",
                    error_msg
                ),
            }
        } else {
            IndexerError::GoogleServiceAccountInvalid {
                message: error_msg,
            }
        }
    })?;
```

#### 字段验证
```rust
// 验证必需字段
if service_account_key.client_email.is_empty() {
    error!("Service account key is missing client_email");
    return Err(IndexerError::GoogleServiceAccountInvalid {
        message: "Service account JSON is missing 'client_email' field".to_string(),
    });
}
```

#### 详细的日志信息
```rust
debug!("Reading service account key from: {:?}", service_account_path);
debug!("Service account email: {}", service_account_key.client_email);
debug!("Project ID: {:?}", service_account_key.project_id);
info!("Successfully created service account authenticator");
```

### 2. `/src/commands/google.rs` - 改进用户界面

#### 在 `setup` 命令中增强 JSON 验证

用户现在会看到友好的错误消息和解决步骤：

```
✗ Failed to read JSON file
  Error: Not enough private keys in PEM

This error usually means:
  1. The JSON file is corrupted or incomplete
  2. The 'private_key' field is missing or malformed
  3. You downloaded P12 format instead of JSON

Solution:
  1. Visit: https://console.cloud.google.com/iam-admin/serviceaccounts
  2. Select your service account
  3. Keys → Add Key → Create new key
  4. IMPORTANT: Choose JSON format (NOT P12!)
  5. Click Create
```

#### 在 `verify` 命令中预验证 JSON 格式

在测试认证前，现在会验证 JSON 文件的格式：

```rust
// 验证 JSON 文件格式
print!("{}", "  Validating JSON format... ".dimmed());
match yup_oauth2::read_service_account_key(&service_account_file).await {
    Ok(key) => {
        println!("{}", "✓".green());
        println!("    Service Account: {}", key.client_email.cyan());
    }
    Err(e) => {
        println!("{}", "✗".red());
        // 显示详细的错误和修复步骤
        if e.to_string().contains("Not enough private keys in PEM") {
            println!("{}", "Common causes:".yellow());
            println!("  • Private key field is missing or corrupted");
            println!("  • Downloaded P12 format instead of JSON");
            println!("  • JSON file is truncated or damaged");
            println!();
            println!("{}", "Fix:".green());
            println!("  1. Delete the current key from Google Cloud Console");
            println!("  2. Create a new key and download as JSON format");
            println!("  3. Run: indexer-cli google setup --service-account <path>");
        }
        return Err(...);
    }
}
```

## 改进的工作流程

### 使用 `setup` 命令时
1. 用户输入 JSON 文件路径
2. 系统立即验证 JSON 格式
3. 如果格式错误，显示具体原因和解决步骤
4. 如果格式正确，继续配置和测试

### 使用 `verify` 命令时
1. 检查配置文件
2. 检查服务账户文件存在
3. **新增**：验证 JSON 格式 ← 在这里捕获大多数错误
4. 测试认证
5. 测试 API 连接

## 错误检测逻辑

新增的错误检测会查找以下关键字来识别 PEM 相关错误：
- `"Not enough private keys in PEM"`
- `"private_key"`
- `"key"`

当检测到这些错误时，会提供特定的修复指导。

## 测试验证

编译成功，所有修改都是向后兼容的：
```
Compiling indexer-cli v0.1.0
Finished `dev` profile [unoptimized + debuginfo] target(s) in 8.14s
```

## 使用建议

### 对于用户
1. **如果收到 PEM 错误**：
   - 删除当前的服务账户密钥
   - 在 Google Cloud Console 中创建新密钥
   - **重要**：选择 JSON 格式（不是 P12）
   - 重新运行 `setup` 命令

2. **如果仍有问题**：
   - 运行 `verify` 命令检查详细诊断
   - 检查 JSON 文件内容是否完整
   - 查看日志输出：`RUST_LOG=debug indexer-cli google verify`

### 对于开发者
1. 所有改进都包含详细注释
2. 添加了适当的日志级别（debug, info, error）
3. 向后兼容 - 不破坏现有代码
4. 错误处理遵循现有的 `IndexerError` 模式

## 文件修改总结

| 文件 | 修改内容 | 行数 |
|------|--------|------|
| `src/api/google_indexing.rs` | 增强 `create_service_account_authenticator` 方法 | +60 |
| `src/commands/google.rs` | 改进 `setup` 和 `verify` 命令中的错误处理 | +50 |

总计：约 110 行代码改进

## 相关资源

- Google Cloud Console: https://console.cloud.google.com/iam-admin/serviceaccounts
- Indexing API: https://console.cloud.google.com/apis/library/indexing.googleapis.com
- yup-oauth2 文档: https://docs.rs/yup-oauth2/
