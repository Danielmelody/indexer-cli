# IndexNow Setup Guide

IndexNow is an open protocol that allows website owners to instantly notify search engines about content changes.

## Supported Search Engines

- Microsoft Bing
- Yandex
- Seznam.cz
- Naver

## Step 1: Generate an API Key

Generate a new API key (32 characters recommended):

```bash
# Generate key only
indexer-cli indexnow generate-key --length 32

# Generate and save to configuration
indexer-cli indexnow generate-key --length 32 --save

# Generate and output key file
indexer-cli indexnow generate-key --length 32 --output /var/www/html/
```

Example output:
```
Generated IndexNow API key: 3f8a9c2b1d7e6f5a4c3b2a1d7e6f5a4c
```

## Step 2: Create Key File

Create a text file with your API key and host it at your website's root:

1. Create file: `your-api-key.txt`
2. Content: exactly your API key (no extra spaces or newlines)
3. Upload to: `https://yourdomain.com/your-api-key.txt`

Example:
```bash
echo -n "3f8a9c2b1d7e6f5a4c3b2a1d7e6f5a4c" > 3f8a9c2b1d7e6f5a4c3b2a1d7e6f5a4c.txt
# Upload this file to your web server's document root
```

## Step 3: Configure indexer-cli

```bash
indexer-cli indexnow setup \
  --key 3f8a9c2b1d7e6f5a4c3b2a1d7e6f5a4c \
  --key-location https://yourdomain.com/3f8a9c2b1d7e6f5a4c3b2a1d7e6f5a4c.txt
```

## Step 4: Verify Setup

Verify that the key file is accessible:

```bash
indexer-cli indexnow verify
```

This checks:
1. Key file is accessible via HTTPS
2. File content matches your API key

## Key Requirements

- **Length**: 8-128 characters (32 recommended)
- **Characters**: Alphanumeric (a-z, A-Z, 0-9) and hyphens (-) only
- **File location**: Must be accessible via HTTPS at domain root
- **File content**: Must exactly match the API key (no whitespace)

## Best Practices

- Keep API key secret, but key file public
- Use HTTPS for key file location
- Submit batches when possible (up to 10,000 URLs)
- Include all URLs from same host in one request
- Avoid submitting same URL repeatedly in short time

## Supported Endpoints

The tool submits to multiple endpoints simultaneously:

- `https://api.indexnow.org/indexnow` - Main endpoint
- `https://www.bing.com/indexnow` - Bing direct
- `https://yandex.com/indexnow` - Yandex direct

### Submit to Specific Endpoint

```bash
indexer-cli indexnow submit https://your-site.com/page --endpoint bing
```

Valid endpoints: `indexnow`, `bing`, `yandex`

## Troubleshooting

### "Key file not accessible"

Verify your key file is publicly accessible:
```bash
curl https://yourdomain.com/your-api-key.txt
```

### "Invalid key format"

- Check key length (8-128 characters)
- Verify only alphanumeric and hyphens are used
- Ensure no whitespace in key file

### "All endpoints failed"

- Verify your key file is accessible from all search engines
- Check network connectivity
- Use `indexer-cli validate` to check configuration

See [Troubleshooting Guide](TROUBLESHOOTING.md) for more help.
