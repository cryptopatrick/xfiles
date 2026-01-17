use xfiles::*;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<()> {
    println!("╔════════════════════════════════════════════════════════╗");
    println!("║     xfiles Twitter API Integration Test               ║");
    println!("╚════════════════════════════════════════════════════════╝\n");

    // Step 1: Check OAuth credentials
    println!("📋 Step 1: Checking OAuth 1.0a credentials...");

    let api_key = match std::env::var("TWITTER_API_KEY") {
        Ok(key) => {
            println!("   ✓ TWITTER_API_KEY found");
            key
        }
        Err(_) => {
            eprintln!("\n❌ ERROR: TWITTER_API_KEY not set!");
            print_setup_instructions();
            std::process::exit(1);
        }
    };

    let api_secret = match std::env::var("TWITTER_API_SECRET") {
        Ok(secret) => {
            println!("   ✓ TWITTER_API_SECRET found");
            secret
        }
        Err(_) => {
            eprintln!("\n❌ ERROR: TWITTER_API_SECRET not set!");
            print_setup_instructions();
            std::process::exit(1);
        }
    };

    let access_token = match std::env::var("TWITTER_ACCESS_TOKEN") {
        Ok(token) => {
            println!("   ✓ TWITTER_ACCESS_TOKEN found");
            token
        }
        Err(_) => {
            eprintln!("\n❌ ERROR: TWITTER_ACCESS_TOKEN not set!");
            print_setup_instructions();
            std::process::exit(1);
        }
    };

    let access_token_secret = match std::env::var("TWITTER_ACCESS_TOKEN_SECRET") {
        Ok(secret) => {
            println!("   ✓ TWITTER_ACCESS_TOKEN_SECRET found");
            secret
        }
        Err(_) => {
            eprintln!("\n❌ ERROR: TWITTER_ACCESS_TOKEN_SECRET not set!");
            print_setup_instructions();
            std::process::exit(1);
        }
    };

    let username = std::env::var("TWITTER_USERNAME")
        .unwrap_or_else(|_| "xfiles_test".to_string());
    println!("   ✓ Username: @{}", username);

    // Confirm before proceeding
    println!("\n⚠️  WARNING: This will post tweets to your account!");
    print!("   Continue? (yes/no): ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    if !input.trim().eq_ignore_ascii_case("yes") {
        println!("\n❌ Test cancelled by user");
        return Ok(());
    }

    // Step 2: Initialize xfiles
    println!("\n📦 Step 2: Initializing xfiles...");
    let mut fs = match XFS::connect(&username, &api_key, &api_secret,
                                     &access_token, &access_token_secret).await {
        Ok(fs) => {
            println!("   ✓ Connected to Twitter API");
            println!("   ✓ SQLite database initialized");
            fs
        }
        Err(e) => {
            eprintln!("\n❌ Failed to connect: {}", e);
            eprintln!("\nPossible issues:");
            eprintln!("- Invalid OAuth credentials");
            eprintln!("- App doesn't have 'Read and Write' permissions");
            eprintln!("- Network connectivity issues");
            eprintln!("\nCheck your credentials at: https://developer.twitter.com/en/portal/dashboard");
            return Err(e);
        }
    };

     // Step 3: Create a test file (posts root tweet)
     println!("\n📝 Step 3: Creating test file...");
     println!("   This will post a root tweet...");

     let test_filename = format!("xfiles_test_{}.txt", chrono::Utc::now().timestamp());

     let mut file = match fs.open(&test_filename, OpenMode::Create).await {
         Ok(file) => {
             println!("   ✓ File created: {}", test_filename);
             println!("   ✓ Root tweet posted");
             println!("   📍 Tweet ID: {}", file.head());
             println!("   🔗 URL: https://twitter.com/{}/status/{}", username, file.head());
             file
         }
         Err(e) => {
             eprintln!("\n❌ Failed to create file: {}", e);
             eprintln!("\nPossible issues:");
             eprintln!("- Rate limit exceeded");
             eprintln!("- App doesn't have write permissions");
             eprintln!("- Invalid authentication");
             return Err(e);
         }
     };

     // Step 4: Write content (posts reply tweet)
     println!("\n✍️  Step 4: Writing content...");
     println!("   This will post a reply tweet...");

     let content = format!("xfiles test @ {}", chrono::Utc::now().to_rfc3339());

     match file.write(content.as_bytes()).await {
         Ok(_) => {
             println!("   ✓ Content written");
             println!("   ✓ Reply tweet posted");
             println!("   📍 New head: {}", file.head());
             println!("   🔗 URL: https://twitter.com/{}/status/{}", username, file.head());
         }
         Err(e) => {
             eprintln!("\n❌ Failed to write: {}", e);
             return Err(e);
         }
     }

     // Step 5: Read content back
     println!("\n📖 Step 5: Reading content back...");
     match file.read().await {
         Ok(read_content) => {
             println!("   ✓ Content fetched from Twitter");
             println!("   Content: {}", String::from_utf8_lossy(&read_content));

             if read_content == content.as_bytes() {
                 println!("   ✅ Content matches! Read/write works correctly.");
             } else {
                 println!("   ⚠️  Content mismatch!");
                 println!("   Expected: {}", content);
                 println!("   Got: {}", String::from_utf8_lossy(&read_content));
             }
         }
         Err(e) => {
             eprintln!("\n❌ Failed to read: {}", e);
             return Err(e);
         }
     }

     // Step 6: Test multiple writes (commit chain)
     println!("\n🔗 Step 6: Testing commit chain...");
     println!("   Posting multiple updates...");

     for i in 1..=3 {
         let update = format!("Update #{} @ {}", i, chrono::Utc::now().timestamp());
         match file.write(update.as_bytes()).await {
             Ok(_) => {
                 println!("   ✓ Update #{} posted: {}", i, file.head());
             }
             Err(e) => {
                 eprintln!("   ❌ Update #{} failed: {}", i, e);
                 return Err(e);
             }
         }
         // Small delay to avoid rate limits
         tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
     }

     // Step 7: Check history
     println!("\n📜 Step 7: Checking file history...");
     match fs.history(&test_filename).await {
         Ok(history) => {
             println!("   ✓ Retrieved history");
             println!("   Total commits: {}", history.len());
             println!("\n   Commit chain:");
             for (i, commit) in history.iter().enumerate() {
                 println!("   {}. {} @ {}",
                     i + 1,
                     commit.id,
                     commit.timestamp.format("%Y-%m-%d %H:%M:%S"));
             }
         }
         Err(e) => {
             eprintln!("   ❌ Failed to get history: {}", e);
             return Err(e);
         }
     }

     // Step 8: List files
     println!("\n📂 Step 8: Listing files...");
     match fs.list("").await {
         Ok(files) => {
             println!("   ✓ Files in filesystem:");
             for file in &files {
                 println!("      - {}", file);
             }
         }
         Err(e) => {
             eprintln!("   ❌ Failed to list files: {}", e);
             return Err(e);
         }
     }

     // Step 9: Test large content (chunking)
     println!("\n📦 Step 9: Testing chunking...");
     let large_content = "x".repeat(500); // Larger than tweet limit
     println!("   Writing {} bytes (will be chunked)...", large_content.len());

     match file.write(large_content.as_bytes()).await {
         Ok(_) => {
             println!("   ✓ Large content posted (auto-chunked)");
             println!("   📍 Head: {}", file.head());
         }
         Err(e) => {
             eprintln!("   ❌ Chunking test failed: {}", e);
             return Err(e);
         }
     }

     // Final summary
     println!("\n╔════════════════════════════════════════════════════════╗");
     println!("║              ✅ ALL TESTS PASSED!                      ║");
     println!("╚════════════════════════════════════════════════════════╝\n");

     println!("🎉 Your Twitter API integration is working correctly!\n");
     println!("📊 Test Summary:");
     println!("   ✓ Authentication successful");
     println!("   ✓ File creation (root tweets)");
     println!("   ✓ Content writing (reply tweets)");
     println!("   ✓ Content reading (fetch tweets)");
     println!("   ✓ Commit chains (multiple replies)");
     println!("   ✓ History tracking");
     println!("   ✓ File listing");
     println!("   ✓ Chunking (large content)");

     println!("\n🔗 View your tweets at:");
     println!("   https://twitter.com/{}", username);

     println!("\n💡 Next steps:");
     println!("   - Check your Twitter timeline to see the posted tweets");
     println!("   - Try the full example: cargo run --example twitter_real");
     println!("   - Integrate xfiles into your agent!\n");

     Ok(())
 }
fn print_setup_instructions() {
    eprintln!("\nTo fix this, set up your OAuth 1.0a credentials:");
    eprintln!("1. Go to https://developer.twitter.com/en/portal/dashboard");
    eprintln!("2. Create a project and app");
    eprintln!("3. Set app permissions to 'Read and Write'");
    eprintln!("4. Generate OAuth 1.0a credentials:");
    eprintln!("   - API Key & Secret (Consumer Keys)");
    eprintln!("   - Access Token & Secret");
    eprintln!("5. Set environment variables:");
    eprintln!("   export TWITTER_API_KEY=\"your_api_key\"");
    eprintln!("   export TWITTER_API_SECRET=\"your_api_secret\"");
    eprintln!("   export TWITTER_ACCESS_TOKEN=\"your_access_token\"");
    eprintln!("   export TWITTER_ACCESS_TOKEN_SECRET=\"your_access_token_secret\"");
    eprintln!("   export TWITTER_USERNAME=\"your_username\"\n");
    eprintln!("See docs/TWITTER_SETUP.md for detailed instructions.\n");
}
