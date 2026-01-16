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

## Step 3: Get Your Bearer Token

1. Navigate to your app in the Developer Portal
2. Go to the "Keys and tokens" tab
3. Under "Authentication Tokens", find "Bearer Token"
4. Click "Generate" if it doesn't exist
5. **IMPORTANT**: Copy and save the Bearer Token immediately - you won't be able to see it again!

## Step 4: Configure xfiles

### Option A: Environment Variables (Recommended)

```bash
export TWITTER_BEARER_TOKEN="your_bearer_token_here"
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
   TWITTER_BEARER_TOKEN=your_actual_bearer_token
   TWITTER_USERNAME=your_username
   ```

3. Load the .env file in your application (requires `dotenv` crate)

## Step 5: Test Your Setup

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
