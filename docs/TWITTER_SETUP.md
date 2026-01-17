# Twitter API Setup Guide

This guide walks you through setting up Twitter API access for xfiles.

## Prerequisites

- A Twitter account
- Twitter Developer account (free)

## Step 1: Create Twitter Developer Account

1. Go to https://developer.twitter.com/en/portal/dashboard
2. Sign in with your Twitter account
3. Click "Sign up for Free Account" if you don't have developer access yet
4. Fill out the application form (basic info about your use case)
5. Verify your email address

## Step 2: Create a Project and App

1. In the Developer Portal dashboard, click "Create Project"
2. Give your project a name (e.g., "xfiles")
3. Select use case (e.g., "Making a bot" or "Exploring the API")
4. Provide project description
5. Create an App within the project
6. Give your app a name (e.g., "xfiles-bot")

## Step 3: Configure App Permissions

1. Navigate to your app in the Developer Portal
2. Go to "Settings" tab
3. Under "User authentication settings", click "Set up"
4. Enable "Read and Write" permissions (required for posting tweets)
5. Save your changes

**IMPORTANT**: If you change permissions, you must regenerate your access tokens!

## Step 4: Get Your OAuth 1.0a Credentials

You need **four** credentials for xfiles (OAuth 1.0a):

1. Navigate to your app's "Keys and tokens" tab
2. **API Key & API Secret (Consumer Keys)**:
   - Find "Consumer Keys" section
   - Copy your "API Key" (also called Consumer Key)
   - Copy your "API Secret" (also called Consumer Secret)

3. **Access Token & Access Token Secret**:
   - Find "Authentication Tokens" section
   - Click "Generate" if tokens don't exist
   - Copy your "Access Token"
   - Copy your "Access Token Secret"
   - **IMPORTANT**: You won't be able to see these again!

All four credentials are required for posting tweets.

## Step 5: Configure xfiles

### Option A: Environment Variables (Recommended)

```bash
export TWITTER_API_KEY="your_api_key_here"
export TWITTER_API_SECRET="your_api_secret_here"
export TWITTER_ACCESS_TOKEN="your_access_token_here"
export TWITTER_ACCESS_TOKEN_SECRET="your_access_token_secret_here"
export TWITTER_USERNAME="your_twitter_username"
```

Add these to your `~/.bashrc`, `~/.zshrc`, or equivalent.

### Option B: .env File

1. Copy the example environment file:
   ```bash
   cp .env.example .env
   ```

2. Edit `.env` and add your credentials:
   ```
   TWITTER_API_KEY=your_actual_api_key
   TWITTER_API_SECRET=your_actual_api_secret
   TWITTER_ACCESS_TOKEN=your_actual_access_token
   TWITTER_ACCESS_TOKEN_SECRET=your_actual_access_token_secret
   TWITTER_USERNAME=your_username
   ```

3. Load the .env file in your application (requires `dotenv` crate)

## Step 6: Test Your Setup

Run the example:

```bash
cargo run --example twitter_real
```

This will:
- Connect to Twitter API
- Create a root tweet
- Post a reply
- Read the content back
- Display the tweet URLs

## API Limits

Twitter API v2 Free Tier limits:
- **Tweets**: 50 requests per 15 minutes (read)
- **Post tweets**: 300 requests per 15 minutes (write)
- **Search tweets**: 180 requests per 15 minutes

xfiles automatically handles rate limiting with exponential backoff.

## Troubleshooting

### "Authentication failed"
- Verify your Bearer Token is correct
- Check that the token hasn't expired
- Ensure you're using Twitter API v2 (not v1.1)

### "App is read-only"
- In Developer Portal, go to your app settings
- Under "User authentication settings", enable "Read and Write" permissions
- Regenerate your Bearer Token after changing permissions

### "Rate limit exceeded"
- Wait for the rate limit window to reset (15 minutes)
- Consider upgrading to a higher API tier if needed
- xfiles will automatically retry after rate limit resets

## Security Best Practices

1. **Never commit credentials**: Always use environment variables or .env files
2. **Keep .env in .gitignore**: The .gitignore is already configured
3. **Rotate tokens regularly**: Generate new tokens periodically
4. **Use separate tokens for dev/prod**: Create different apps for testing
5. **Monitor API usage**: Check the Developer Portal for usage stats

## Additional Resources

- [Twitter API v2 Documentation](https://developer.twitter.com/en/docs/twitter-api)
- [Rate Limits](https://developer.twitter.com/en/docs/twitter-api/rate-limits)
- [Authentication](https://developer.twitter.com/en/docs/authentication/oauth-2-0)
- [API Reference](https://developer.twitter.com/en/docs/api-reference-index)

## Need Help?

- Check existing issues: https://github.com/cryptopatrick/xfiles/issues
- Create a new issue if you encounter problems
- Join discussions in the repository
