# Google Setup Guide

This guide will help you set up Google Indexing API authentication for indexer-cli.

## Prerequisites

1. A Google Cloud Platform (GCP) project
2. Website verified in Google Search Console
3. Service account added as an **Owner** on the Search Console property

## Step 1: Create a GCP Project

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project or select an existing one
3. Note your project ID

## Step 2: Enable the Indexing API

1. Navigate to **APIs & Services** > **Library**
2. Search for "Web Search Indexing API" or "Indexing API"
3. Click **Enable**

## Step 3: Create a Service Account

1. Go to **APIs & Services** > **Credentials**
2. Click **Create Credentials** > **Service Account**
3. Enter a name (e.g., "indexer-cli-service")
4. Click **Create and Continue**
5. Skip optional steps and click **Done**

## Step 4: Generate Service Account Key

1. In the service accounts list, click on your new account
2. Go to the **Keys** tab
3. Click **Add Key** > **Create new key**
4. Select **JSON** format
5. Click **Create** - the key file will download
6. Save securely (e.g., `~/.indexer-cli/service-account.json`)

## Step 5: Grant Search Console Access

1. Go to [Google Search Console](https://search.google.com/search-console/)
2. Select your property
3. Go to **Settings** > **Users and permissions**
4. Click **Add user**
5. Enter the service account email (`name@project-id.iam.gserviceaccount.com`)
6. Select **Owner** permission level
7. Click **Add**

## Step 6: Configure indexer-cli

```bash
# Set up service account
indexer-cli google setup --service-account /path/to/service-account.json

# Verify configuration
indexer-cli google verify
```

## Quota Limits

- **Daily publish limit**: 200 URL notifications per day
- **Rate limit**: 380 requests per minute (total)
- **Metadata rate limit**: 180 requests per minute

The tool automatically respects these limits with rate limiting and quota tracking.

## Best Practices

- Only submit URLs you own and have verified in Search Console
- Use UPDATE action for new or modified pages
- Use DELETE action for removed pages
- Don't resubmit URLs unnecessarily (use history tracking)
- Monitor quota usage regularly: `indexer-cli google quota`

## Troubleshooting

### "Permission denied"

Ensure your service account has **Owner** permission in Google Search Console for the domain.

### "Service account not found"

Verify the JSON key file path is correct and the file is readable.

### "API not enabled"

Make sure the Indexing API is enabled in your GCP project.

See [Troubleshooting Guide](TROUBLESHOOTING.md) for more common issues.
