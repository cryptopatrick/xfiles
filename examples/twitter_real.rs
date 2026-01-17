//! Example using real Twitter API
//!
//! Prerequisites:
//! 1. Set up Twitter Developer account at https://developer.twitter.com
//! 2. Create a project and app with "Read and Write" permissions
//! 3. Get your OAuth 1.0a credentials (4 values)
//! 4. Set environment variables
//!
//! Run with:
//! ```
//! export TWITTER_API_KEY="your_api_key"
//! export TWITTER_API_SECRET="your_api_secret"
//! export TWITTER_ACCESS_TOKEN="your_access_token"
//! export TWITTER_ACCESS_TOKEN_SECRET="your_access_token_secret"
//! export TWITTER_USERNAME="your_username"
//! cargo run --example twitter_real
//! ```

use xfiles::*;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== xfiles Twitter API Example ===\n");

    // Get OAuth credentials from environment
    let api_key = std::env::var("TWITTER_API_KEY")
        .expect("TWITTER_API_KEY environment variable not set");
    let api_secret = std::env::var("TWITTER_API_SECRET")
        .expect("TWITTER_API_SECRET environment variable not set");
    let access_token = std::env::var("TWITTER_ACCESS_TOKEN")
        .expect("TWITTER_ACCESS_TOKEN environment variable not set");
    let access_token_secret = std::env::var("TWITTER_ACCESS_TOKEN_SECRET")
        .expect("TWITTER_ACCESS_TOKEN_SECRET environment variable not set");

    let username = std::env::var("TWITTER_USERNAME")
        .unwrap_or_else(|_| "xfiles_agent".to_string());

    println!("Connecting to Twitter API as @{}...", username);

    // Connect with real Twitter API using OAuth 1.0a
    let mut fs = XFS::connect(&username, &api_key, &api_secret,
                               &access_token, &access_token_secret).await?;
    println!("✓ Connected successfully\n");

    println!("Creating a file (will post a root tweet)...");
    let mut file = fs.open("agent_memory.txt", OpenMode::Create).await?;
    println!("✓ Root tweet created: {}\n", file.head());

    println!("Writing content (will post reply tweet)...");
    file.write(b"xfiles v0.1 - Twitter as a filesystem").await?;
    println!("✓ Content written to tweet: {}\n", file.head());

    println!("Reading content back...");
    let content = file.read().await?;
    println!("Content: {}\n", String::from_utf8_lossy(&content));

    println!("Adding another update...");
    file.write(b"Successfully tested with real Twitter API!").await?;
    println!("✓ Update posted: {}\n", file.head());

    println!("Getting file history...");
    let history = fs.history("agent_memory.txt").await?;
    println!("Total commits: {}", history.len());
    for (i, commit) in history.iter().enumerate() {
        println!("  Commit {}: https://twitter.com/i/web/status/{}", i + 1, commit.id);
    }

    println!("\n=== Success ===");
    println!("Your tweets are now part of xfiles!");
    println!("Visit https://twitter.com/{} to see them.\n", username);

    Ok(())
}
