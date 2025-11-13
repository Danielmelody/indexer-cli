# Troubleshooting Guide

Common issues and solutions for indexer-cli.

## Table of Contents

- [Configuration Issues](#configuration-issues)
- [Google API Issues](#google-api-issues)
- [IndexNow Issues](#indexnow-issues)
- [Sitemap Issues](#sitemap-issues)
- [Database Issues](#database-issues)
- [Rate Limiting](#rate-limiting)
- [Debug Mode](#debug-mode)
- [Log Files](#log-files)
- [Getting Help](#getting-help)

## Configuration Issues

### "Configuration file not found"

**Error:**
```
Error: Configuration file not found at ~/.indexer-cli/config.yaml
```

**Solution:**
Create a configuration file:
```bash
indexer-cli init
```

Or create a minimal config manually:
```bash
mkdir -p ~/.indexer-cli
cat > ~/.indexer-cli/config.yaml <<EOF
google:
  enabled: false
indexnow:
  enabled: false
history:
  enabled: true
  database_path: ~/.indexer-cli/history.db
EOF
```

### "Invalid configuration format"

**Error:**
```
Error: Failed to parse config.yaml: invalid YAML format
```

**Solution:**
Validate your YAML syntax:
```bash
# Install yamllint if needed
yamllint ~/.indexer-cli/config.yaml

# Or use online YAML validator
```

Common issues:
- Incorrect indentation (use spaces, not tabs)
- Missing colons after keys
- Unquoted special characters

**Fix:**
```yaml
# Wrong
google:
enabled: true  # Missing indentation

# Correct
google:
  enabled: true
```

### "Setting not found"

**Error:**
```
Error: Configuration key 'google.service_account' not found
```

**Solution:**
Check available settings:
```bash
indexer-cli config list
```

Verify correct key path:
```bash
# Wrong
indexer-cli config get google.service_account

# Correct
indexer-cli config get google.service_account_file
```

## Google API Issues

### "Service account not found"

**Error:**
```
Error: Service account file not found: /path/to/service-account.json
```

**Solution:**
1. Verify file path is correct
2. Check file permissions:
   ```bash
   ls -la /path/to/service-account.json
   chmod 600 /path/to/service-account.json
   ```
3. Set correct path in config:
   ```bash
   indexer-cli google setup --service-account /correct/path/service-account.json
   ```

### "Permission denied (Google API)"

**Error:**
```
Error: Permission denied: The caller does not have permission
```

**Solution:**
1. Verify service account email format:
   ```
   name@project-id.iam.gserviceaccount.com
   ```

2. Check Search Console permissions:
   - Go to Google Search Console
   - Select your property
   - Settings > Users and permissions
   - Verify service account has "Owner" role

3. Re-add service account if needed:
   - Remove existing entry
   - Add again with "Owner" permission
   - Wait 5-10 minutes for propagation

4. Verify using:
   ```bash
   indexer-cli google verify
   ```

### "API not enabled"

**Error:**
```
Error: Indexing API has not been used in project before or it is disabled
```

**Solution:**
Enable the Indexing API:
1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Select your project
3. Navigate to **APIs & Services** > **Library**
4. Search for "Web Search Indexing API"
5. Click **Enable**

Wait a few minutes for activation, then retry.

### "Quota exceeded"

**Error:**
```
Error: Rate limit exceeded. Quota exceeded for quota metric
```

**Solution:**
1. Check current quota usage:
   ```bash
   indexer-cli google quota
   ```

2. Wait for quota reset (daily at midnight Pacific Time)

3. Reduce batch size in config:
   ```yaml
   google:
     batch_size: 50  # Reduced from default
   ```

4. Enable more aggressive rate limiting:
   ```yaml
   google:
     quota:
       rate_limit: 300  # Reduced from 380
   ```

### "Invalid URL for Google API"

**Error:**
```
Error: URL is not valid for Google Indexing API
```

**Cause:**
Google Indexing API is primarily for:
- Job postings
- Livestream videos

Not for general web pages.

**Solution:**
- Use IndexNow for general pages
- Or use Google Search Console sitemap submission
- For job/video content, ensure proper schema markup

See: [Google Indexing API documentation](https://developers.google.com/search/apis/indexing-api/v3)

## IndexNow Issues

### "Key file not accessible"

**Error:**
```
Error: IndexNow key file is not accessible: HTTP 404
```

**Solution:**
1. Verify key file URL:
   ```bash
   curl -v https://yourdomain.com/your-api-key.txt
   ```

2. Check file location:
   - Must be at domain root or subdirectory
   - Must be accessible via HTTPS
   - Should not redirect

3. Verify file content:
   ```bash
   # File should contain ONLY the key, no whitespace
   cat your-api-key.txt
   # Output: 3f8a9c2b1d7e6f5a4c3b2a1d7e6f5a4c
   ```

4. Check web server configuration:
   - Verify .txt files are served
   - Check for hotlink protection
   - Verify no IP blocking

5. Recreate key file:
   ```bash
   echo -n "YOUR-API-KEY" > your-api-key.txt
   # Upload to your server
   ```

### "Key mismatch"

**Error:**
```
Error: Key file content does not match configured API key
```

**Solution:**
1. Get current configured key:
   ```bash
   indexer-cli config get indexnow.api_key
   ```

2. Verify key file content:
   ```bash
   curl -s https://yourdomain.com/key.txt
   ```

3. Fix mismatch:
   - Option A: Update config
     ```bash
     indexer-cli indexnow setup --key "CORRECT-KEY" --key-location "https://yourdomain.com/key.txt"
     ```
   - Option B: Update key file
     ```bash
     echo -n "CONFIGURED-KEY" > key.txt
     # Upload to server
     ```

### "All endpoints failed"

**Error:**
```
Error: All IndexNow endpoints failed
```

**Solution:**
1. Test individual endpoints:
   ```bash
   curl -v https://www.bing.com/indexnow
   curl -v https://yandex.com/indexnow
   ```

2. Check network connectivity:
   ```bash
   ping www.bing.com
   ping yandex.com
   ```

3. Verify no firewall blocking:
   - Ensure outbound HTTPS (port 443) is allowed
   - Check corporate proxy settings

4. Test with minimal example:
   ```bash
   indexer-cli validate indexnow
   indexer-cli indexnow verify
   ```

5. Check configuration:
   ```bash
   indexer-cli config list
   ```

### "Invalid key format"

**Error:**
```
Error: IndexNow API key format is invalid
```

**Solution:**
Key requirements:
- Length: 8-128 characters
- Characters: a-z, A-Z, 0-9, hyphens (-)
- No spaces or special characters

**Generate valid key:**
```bash
indexer-cli indexnow generate-key --length 32
```

## Sitemap Issues

### "Failed to fetch sitemap"

**Error:**
```
Error: Failed to fetch sitemap: HTTP 404
```

**Solution:**
1. Verify sitemap URL:
   ```bash
   curl -I https://your-site.com/sitemap.xml
   ```

2. Check common sitemap locations:
   - `/sitemap.xml`
   - `/sitemap_index.xml`
   - `/sitemap/sitemap.xml`

3. Check robots.txt:
   ```bash
   curl https://your-site.com/robots.txt
   ```

4. Test sitemap access:
   ```bash
   indexer-cli sitemap validate https://your-site.com/sitemap.xml
   ```

### "Invalid XML format"

**Error:**
```
Error: Sitemap XML parsing failed: invalid XML
```

**Solution:**
1. Validate XML syntax:
   ```bash
   curl -s https://your-site.com/sitemap.xml | xmllint --noout -
   ```

2. Common issues:
   - Unescaped characters in URLs (`&` should be `&amp;`)
   - Missing closing tags
   - Incorrect namespace
   - Encoding issues

3. Fix your sitemap generator to produce valid XML

4. Test with sample:
   ```bash
   # Download and examine
   curl -s https://your-site.com/sitemap.xml | head -50
   ```

### "Gzip decompression failed"

**Error:**
```
Error: Failed to decompress gzipped sitemap
```

**Solution:**
1. Verify gzip format:
   ```bash
   curl -s https://your-site.com/sitemap.xml.gz | file -
   ```

2. Try manual decompression:
   ```bash
   curl -s https://your-site.com/sitemap.xml.gz | gunzip -t
   ```

3. Check file integrity:
   ```bash
   curl -s https://your-site.com/sitemap.xml.gz | gzip -t
   ```

4. If file is corrupted, regenerate sitemap

### "Too many URLs in sitemap"

**Error:**
```
Warning: Sitemap contains more than 50,000 URLs
```

**Solution:**
Google recommends max 50,000 URLs per sitemap.

1. Split into multiple sitemaps:
   - By category: `sitemap-products.xml`, `sitemap-blog.xml`
   - By date: `sitemap-2024-01.xml`, `sitemap-2024-02.xml`

2. Create sitemap index:
   ```xml
   <?xml version="1.0" encoding="UTF-8"?>
   <sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
     <sitemap>
       <loc>https://your-site.com/sitemap-products.xml</loc>
     </sitemap>
     <sitemap>
       <loc>https://your-site.com/sitemap-blog.xml</loc>
     </sitemap>
   </sitemapindex>
   ```

3. Process sitemap index:
   ```bash
   indexer-cli sitemap parse https://your-site.com/sitemap-index.xml --follow-index
   ```

## Database Issues

### "Database locked"

**Error:**
```
Error: Database is locked
```

**Cause:**
Multiple indexer-cli instances accessing database simultaneously.

**Solution:**
1. Check running processes:
   ```bash
   ps aux | grep indexer-cli
   ```

2. Kill duplicate processes:
   ```bash
   pkill -f indexer-cli
   ```

3. Wait for operations to complete

4. Use different databases for parallel operations:
   ```yaml
   history:
     database_path: ~/.indexer-cli/history-site1.db
   ```

5. For cron jobs, add locking:
   ```bash
   # Using flock
   flock -n /tmp/indexer-cli.lock indexer-cli submit --sitemap https://site.com/sitemap.xml
   ```

### "Database corrupted"

**Error:**
```
Error: Database disk image is malformed
```

**Solution:**
1. Backup corrupted database:
   ```bash
   cp ~/.indexer-cli/history.db ~/.indexer-cli/history-corrupted.db
   ```

2. Try recovery:
   ```bash
   sqlite3 ~/.indexer-cli/history.db ".recover" | sqlite3 ~/.indexer-cli/history-recovered.db
   ```

3. If recovery fails, create new database:
   ```bash
   mv ~/.indexer-cli/history.db ~/.indexer-cli/history-old.db
   indexer-cli submit https://example.com  # Creates new DB
   ```

4. Export/import data if needed:
   ```bash
   # Export from old (if possible)
   indexer-cli history export --output backup.csv
   ```

### "No such table"

**Error:**
```
Error: no such table: submission_history
```

**Solution:**
Database schema needs initialization:

```bash
# Trigger schema creation
indexer-cli submit https://example.com --dry-run

# Or manually create
cat > ~/.indexer-cli/history.db <<'EOF'
CREATE TABLE submission_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    url TEXT NOT NULL,
    api TEXT NOT NULL,
    action TEXT NOT NULL,
    status TEXT NOT NULL,
    response_code INTEGER,
    response_message TEXT,
    submitted_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    metadata TEXT
);
CREATE INDEX idx_url ON submission_history(url);
CREATE INDEX idx_api ON submission_history(api);
CREATE INDEX idx_status ON submission_history(status);
CREATE INDEX idx_submitted_at ON submission_history(submitted_at);
EOF
```

## Rate Limiting

### Understanding Rate Limits

**Google Indexing API:**
- 200 URLs per day (hard limit)
- 380 requests per minute
- 180 metadata requests per minute

**IndexNow:**
- 10,000 URLs per batch
- Recommended: 1 request per second
- No strict daily limit

### "Rate limit exceeded" Prevention

1. **Configure conservative limits:**
   ```yaml
   google:
     quota:
       daily_limit: 200      # Your actual limit
       rate_limit: 300       # Below Google's 380

   indexnow:
     batch_size: 1000        # Smaller batches
   ```

2. **Monitor usage:**
   ```bash
   # In your scripts
   indexer-cli google quota
   ```

3. **Implement backoff:**
   ```yaml
   retry:
     enabled: true
     backoff_factor: 2
     max_wait_seconds: 120
   ```

4. **Schedule submissions:**
   ```bash
   # Spread across day
   0 3 * * * indexer-cli submit --sitemap https://site.com/sitemap.xml
   ```

### Handling 429 Errors

The tool automatically retries 429 (rate limit) errors:

```bash
# View retry behavior with debug logging
RUST_LOG=debug indexer-cli submit https://site.com/page
```

If you hit limits frequently:
1. Reduce `batch_size`
2. Increase `rate_limit` wait time
3. Schedule submissions during off-peak hours

## Debug Mode

### Enable Debug Logging

```bash
# Debug logging
RUST_LOG=debug indexer-cli submit https://your-site.com/page

# Trace logging (very detailed)
RUST_LOG=trace indexer-cli submit https://your-site.com/page
```

### Verbose CLI Output

```bash
indexer-cli --verbose submit https://your-site.com/page
```

Shows:
- Configuration loading
- API request details
- Response headers
- Timing information

### Debug Specific Module

```bash
# Debug API client only
RUST_LOG=indexer_cli::api=debug indexer-cli submit https://your-site.com/page

# Debug database queries
RUST_LOG=indexer_cli::database=debug indexer-cli history list
```

### Common Debug Patterns

**Test configuration loading:**
```bash
RUST_LOG=indexer_cli::config=debug indexer-cli config list
```

**Debug Google auth:**
```bash
RUST_LOG=indexer_cli::api::google=debug indexer-cli google verify
```

**Debug IndexNow submission:**
```bash
RUST_LOG=indexer_cli::api::indexnow=debug indexer-cli indexnow submit https://site.com/page
```

## Log Files

### Log File Location

Default: `~/.indexer-cli/indexer.log`

### Configure Logging

```yaml
logging:
  level: debug
  file: ~/.indexer-cli/debug.log
  max_size_mb: 10
  max_backups: 5
```

### View Logs

```bash
# Follow log in real-time
tail -f ~/.indexer-cli/indexer.log

# View recent errors
grep ERROR ~/.indexer-cli/indexer.log

# View last 100 lines
tail -n 100 ~/.indexer-cli/indexer.log
```

### Log Format

Each log entry includes:
- Timestamp
- Log level (ERROR, WARN, INFO, DEBUG, TRACE)
- Module path
- Message
- Context (when available)

Example:
```
2024-01-15T10:30:45.123Z INFO indexer_cli::commands::google - Submitting URL: https://example.com/page
2024-01-15T10:30:45.456Z DEBUG indexer_cli::api::google - Request: POST https://indexing.googleapis.com/v3/urlNotifications:publish
2024-01-15T10:30:46.789Z INFO indexer_cli::commands::google - Successfully submitted https://example.com/page
```

## Getting Help

### Before Reporting Issues

1. **Check version:**
   ```bash
   indexer-cli --version
   ```

2. **Verify installation:**
   ```bash
   indexer-cli validate
   ```

3. **Enable debug logging:**
   ```bash
   RUST_LOG=debug indexer-cli submit https://your-site.com/page
   ```

4. **Check logs:**
   ```bash
   tail -n 50 ~/.indexer-cli/indexer.log
   ```

### Information to Provide

When reporting issues, include:

1. **Command and output:**
   ```bash
   indexer-cli submit https://your-site.com/page --verbose
   ```

2. **Configuration** (sanitized):
   ```bash
   indexer-cli config list
   # Remove sensitive values
   ```

3. **Debug log:**
   ```bash
   RUST_LOG=debug indexer-cli submit https://your-site.com/page 2>&1 | tee debug.log
   ```

4. **Version info:**
   ```bash
   indexer-cli --version
   rustc --version
   ```

5. **System info:**
   ```bash
   uname -a
   ```

### Common Issues Checklist

- [ ] Configuration file exists
- [ ] Service account file is valid JSON
- [ ] IndexNow key file is accessible
- [ ] APIs are enabled in config
- [ ] Network connectivity to APIs
- [ ] No firewall blocking outbound HTTPS
- [ ] Sufficient quota remaining
- [ ] Database not locked
- [ ] Running latest version

### Where to Get Help

1. **Check documentation:**
   - Review this troubleshooting guide
   - Check relevant setup guides
   - Read command documentation

2. **Search existing issues:**
   ```bash
   # GitHub issue search
   https://github.com/your-username/indexer-cli/issues
   ```

3. **Ask questions:**
   - GitHub Discussions
   - Stack Overflow with tag `indexer-cli`

4. **Report bugs:**
   - GitHub Issues with template
   - Include all diagnostic information
   - Provide minimal reproduction

### Quick Diagnostics Script

```bash
#!/bin/bash
# diagnostics.sh

echo "=== indexer-cli Diagnostics ==="
echo
echo "Version:"
indexer-cli --version
echo
echo "System:"
uname -a
echo
echo "Config path:"
indexer-cli config path
echo
echo "Config validation:"
indexer-cli config validate
echo
echo "Validate Google:"
indexer-cli validate google 2>&1 | head -5
echo
echo "Validate IndexNow:"
indexer-cli validate indexnow 2>&1 | head -5
echo
echo "Recent log errors:"
grep -A2 ERROR ~/.indexer-cli/indexer.log 2>/dev/null | tail -10 || echo "No logs found"
echo
echo "Diagnostics complete!"
```

Save as `diagnostics.sh`, run:
```bash
chmod +x diagnostics.sh
./diagnostics.sh
```

See also:
- [Google Setup Guide](GOOGLE_SETUP.md)
- [IndexNow Setup Guide](INDEXNOW_SETUP.md)
- [Configuration Guide](CONFIGURATION.md)
- [FAQ](FAQ.md)
