# Google OAuth 2.0 Setup Guide

This guide will walk you through setting up OAuth 2.0 credentials for Google Indexing API in `indexer-cli`.

## Why Do I Need This?

The `indexer-cli` tool uses Google's OAuth 2.0 for user authentication to access the Google Indexing API. Unlike service accounts, OAuth allows you to authenticate as yourself and manage URLs for properties you own in Google Search Console.

**Important:** You need to create your own OAuth 2.0 client credentials. The default credentials in the tool are placeholders and will not work.

## Prerequisites

- A Google account
- Access to [Google Cloud Console](https://console.cloud.google.com)
- Properties verified in [Google Search Console](https://search.google.com/search-console)

## Step-by-Step Setup

### Step 1: Create or Select a Google Cloud Project

1. Visit the [Google Cloud Console](https://console.cloud.google.com)
2. Sign in with your Google account
3. Click on the project dropdown at the top of the page
4. Either:
   - Click "New Project" to create a new project
   - Select an existing project

**Note:** If creating a new project, give it a meaningful name like "Indexer CLI" or "My SEO Tools"

### Step 2: Enable Google Indexing API

1. In the Google Cloud Console, navigate to **APIs & Services** > **Library**
2. In the search bar, type "Indexing API"
3. Click on **Indexing API** in the search results
4. Click the **Enable** button
5. Wait for the API to be enabled (usually takes a few seconds)

### Step 3: Create OAuth 2.0 Credentials

1. Navigate to **APIs & Services** > **Credentials**
2. Click **+ Create Credentials** at the top
3. Select **OAuth client ID** from the dropdown

#### Configure OAuth Consent Screen (First Time Only)

If this is your first time creating OAuth credentials, you'll be prompted to configure the OAuth consent screen:

1. Click **Configure Consent Screen**
2. Choose **External** user type (unless you have a Google Workspace account and want Internal)
3. Click **Create**
4. Fill in the required fields:
   - **App name:** "Indexer CLI" (or your preferred name)
   - **User support email:** Your email address
   - **Developer contact information:** Your email address
5. Click **Save and Continue**
6. On the "Scopes" page, click **Save and Continue** (we'll add scopes programmatically)
7. On the "Test users" page, you can add your email if the app is in testing mode
8. Click **Save and Continue**, then **Back to Dashboard**

#### Create the OAuth Client

1. Back in **Credentials**, click **+ Create Credentials** > **OAuth client ID**
2. For **Application type**, choose one of:
   - **Desktop app** (recommended for CLI tools)
   - **Web application** (also works)
3. Give it a name, e.g., "Indexer CLI OAuth Client"
4. If you chose "Web application":
   - Under **Authorized redirect URIs**, click **+ Add URI**
   - Add: `http://localhost:8080/oauth/callback`
5. Click **Create**
6. A dialog will appear with your credentials:
   - **Client ID:** Something like `123456789-abc123xyz.apps.googleusercontent.com`
   - **Client Secret:** A random string like `GOCSPX-abc123...`
7. **Important:** Copy both the Client ID and Client Secret - you'll need them next

### Step 4: Configure Credentials in indexer-cli

You have three options to provide your credentials to `indexer-cli`:

#### Option A: Environment Variables (Recommended)

This is the most secure and convenient method for regular use:

```bash
# Add to your shell profile (~/.bashrc, ~/.zshrc, etc.)
export GOOGLE_OAUTH_CLIENT_ID="YOUR_CLIENT_ID.apps.googleusercontent.com"
export GOOGLE_OAUTH_CLIENT_SECRET="YOUR_CLIENT_SECRET"

# Then run auth
indexer-cli google auth
```

**Advantages:**
- Credentials are not stored in files
- Works across all projects
- Easy to update

#### Option B: Configuration File

Store credentials in your project's `indexer.yaml` or global config:

```yaml
# indexer.yaml or ~/.indexer-cli/config.yaml
google:
  enabled: true
  auth:
    method: oauth
    oauth_client_id: "YOUR_CLIENT_ID.apps.googleusercontent.com"
    oauth_client_secret: "YOUR_CLIENT_SECRET"
  quota:
    daily_limit: 200
    rate_limit: 380
  batch_size: 100
```

Then run:

```bash
indexer-cli google auth
```

**Advantages:**
- Easy to manage different credentials per project
- Configuration is version-controllable (but add to .gitignore!)

**Security Warning:** Never commit your `oauth_client_secret` to a public repository! Add `indexer.yaml` to your `.gitignore`.

#### Option C: Command Line Arguments (Quick Testing)

Provide credentials directly via command line:

```bash
indexer-cli google auth \
  --client-id "YOUR_CLIENT_ID.apps.googleusercontent.com" \
  --client-secret "YOUR_CLIENT_SECRET"
```

**Advantages:**
- Quick for one-off testing
- No configuration needed

**Disadvantages:**
- Credentials may be visible in shell history
- Must provide every time

### Step 5: Authenticate

Run the authentication command:

```bash
indexer-cli google auth
```

The tool will:
1. Validate your credentials
2. Open your default browser
3. Prompt you to sign in with Google
4. Ask for permission to access Google Indexing API
5. Save the access token locally

**During the OAuth flow:**
- You may see a warning "This app isn't verified" if your app is in testing mode
  - Click "Advanced" → "Go to [App Name] (unsafe)" to proceed
  - This is safe for apps you created yourself
- Review the permissions being requested
- Click "Allow" to grant access

After successful authentication, you'll see:
```
✓ Authorization Successful!
Token saved to: ~/.indexer-cli/tokens/google_oauth_token.json

Next steps:
  • Run 'indexer-cli google verify' to verify the setup
  • Run 'indexer-cli google submit <url>' to submit URLs
  • Run 'indexer-cli google quota' to check your quota
```

### Step 6: Verify Setup

Test your configuration:

```bash
indexer-cli google verify
```

This will:
- Check that credentials are configured
- Verify authentication is working
- Test API connectivity

## Managing Your Credentials

### Viewing Token Location

Your OAuth access token is stored locally at:
```
~/.indexer-cli/tokens/google_oauth_token.json
```

This token is automatically refreshed when it expires.

### Logging Out

To revoke your credentials and logout:

```bash
indexer-cli google logout
```

This will:
1. Revoke the token with Google
2. Delete the local token file

### Re-authenticating

To force re-authentication (e.g., after changing accounts):

```bash
indexer-cli google auth --force
```

## Troubleshooting

### Error: "OAuth client credentials not configured"

**Cause:** You haven't provided custom credentials, and the tool is using placeholder values.

**Solution:** Follow Steps 3-4 above to create and configure your credentials.

### Error: "The server cannot process the request because it is malformed"

**Cause:** The Client ID or Secret is incorrect or still using placeholders.

**Solution:**
1. Double-check your Client ID and Secret from Google Cloud Console
2. Ensure there are no extra spaces or quotes
3. Make sure you're using the full Client ID (includes `.apps.googleusercontent.com`)

### Error: "redirect_uri_mismatch"

**Cause:** The redirect URI in your OAuth client doesn't match what the tool expects.

**Solution:**
1. Go to Google Cloud Console > APIs & Services > Credentials
2. Click on your OAuth client
3. Add `http://localhost:8080/oauth/callback` to Authorized redirect URIs
4. Save and try again

### Error: "Port 8080 may be in use"

**Cause:** Another application is using port 8080.

**Solution:**
1. Stop the application using port 8080
2. Or wait a moment and try again

### Warning: "This app isn't verified"

**Cause:** Your app is in testing mode and hasn't been verified by Google.

**Solution:**
- This is normal for personal/testing apps
- Click "Advanced" → "Go to [App Name] (unsafe)" to proceed
- If you want to verify your app (for production use with many users):
  - Go to OAuth consent screen in Google Cloud Console
  - Click "Publish App" (requires verification if accessing sensitive scopes)

### Permission Denied Errors When Submitting URLs

**Cause:** You don't have access to the property in Google Search Console.

**Solution:**
1. Go to [Google Search Console](https://search.google.com/search-console)
2. Add and verify ownership of the website
3. Make sure you're authenticated with the same Google account

## Security Best Practices

1. **Never share your Client Secret publicly**
   - Add config files containing secrets to `.gitignore`
   - Don't commit credentials to version control

2. **Use environment variables for production**
   - More secure than files
   - Easier to rotate credentials

3. **Restrict OAuth client usage**
   - In Google Cloud Console, you can restrict which domains can use your client
   - Consider using separate clients for development and production

4. **Rotate credentials periodically**
   - Delete old OAuth clients you're not using
   - Create new credentials if you suspect they've been compromised

5. **Monitor API usage**
   - Check the Google Cloud Console for unexpected API calls
   - Set up budget alerts if using paid APIs

## Alternative: Service Account Authentication

If you prefer not to use OAuth (for server/automation use cases), you can use Service Account authentication instead:

```bash
indexer-cli google setup --service-account /path/to/service-account.json
```

**Pros:**
- No browser interaction needed
- Better for automation/CI/CD
- No user consent required

**Cons:**
- More setup steps (need to create service account, download JSON, add to Search Console)
- Service account email must be added as owner for each property in Search Console

See the main documentation for service account setup instructions.

## Additional Resources

- [Google OAuth 2.0 Documentation](https://developers.google.com/identity/protocols/oauth2)
- [Google Indexing API Documentation](https://developers.google.com/search/apis/indexing-api/v3/quickstart)
- [Google Search Console Help](https://support.google.com/webmasters/)

## Getting Help

If you encounter issues not covered in this guide:

1. Check the [GitHub Issues](https://github.com/yourusername/indexer-cli/issues)
2. Run with verbose logging: `indexer-cli --verbose google auth`
3. Open a new issue with the error message and steps to reproduce

## Summary

Here's a quick checklist:

- [ ] Create Google Cloud Project
- [ ] Enable Google Indexing API
- [ ] Create OAuth 2.0 Client
- [ ] Configure OAuth Consent Screen
- [ ] Add redirect URI: `http://localhost:8080/oauth/callback`
- [ ] Copy Client ID and Secret
- [ ] Configure credentials (environment variables, config file, or CLI args)
- [ ] Run `indexer-cli google auth`
- [ ] Grant permissions in browser
- [ ] Verify with `indexer-cli google verify`
- [ ] Start submitting URLs!

Happy indexing!
