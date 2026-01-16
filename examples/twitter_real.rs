//! Example using real Twitter API
//!
//! Prerequisites:
//! 1. Set up Twitter Developer account at https://developer.twitter.com
//! 2. Create a project and app
//! 3. Get your Bearer Token
//! 4. Set TWITTER_BEARER_TOKEN environment variable
//!
//! Run with:
//! ```
//! export TWITTER_BEARER_TOKEN="your_token_here"
//! cargo run --example twitter_real
//! ```

use xfiles::*;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== xfiles Twitter API Example ===\n");

    // Get credentials from environment
    let bearer_token = std::env::var("TWITTER_BEARER_TOKEN")
        .expect("TWITTER_BEARER_TOKEN environment variable not set");

    let username = std::env::var("TWITTER_USERNAME")
        .unwrap_or_else(|_| "xfiles_agent".to_string());

    println!("Connecting to Twitter API as @{}...", username);

    // Connect with real Twitter API
    let mut fs = XFS::connect(&username, &bearer_token).await?;
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
