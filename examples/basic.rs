//! Basic example of using xfiles with MockAdapter
//!
//! Run with: cargo run --example basic

use std::sync::Arc;
use xfiles::*;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== xfiles Basic Example ===\n");

    // Create a mock adapter (for testing without real Twitter API)
    let adapter = Arc::new(MockAdapter::new());

    // Initialize xfiles with mock adapter
    let mut fs = XFS::with_adapter("agent_smith", adapter, Some(":memory:")).await?;

    println!("1. Creating a new file...");
    let mut memory_file = fs.open("memory.txt", OpenMode::Create).await?;
    println!("   ✓ Created file: {}", memory_file.path());

    println!("\n2. Writing to file...");
    memory_file.write(b"Day 1: Agent initialized").await?;
    println!("   ✓ Wrote initial content");

    println!("\n3. Reading file content...");
    let content = memory_file.read().await?;
    println!("   Content: {}", String::from_utf8_lossy(&content));

    println!("\n4. Writing multiple updates...");
    memory_file.write(b"Day 2: Learned about xfiles").await?;
    memory_file.write(b"Day 3: Successfully stored memory").await?;
    println!("   ✓ Created commit chain");

    println!("\n5. Reading latest version...");
    let latest = memory_file.read().await?;
    println!("   Latest: {}", String::from_utf8_lossy(&latest));

    println!("\n6. Getting file history...");
    let history = fs.history("memory.txt").await?;
    println!("   Total commits: {}", history.len());
    for (i, commit) in history.iter().enumerate() {
        println!("   Commit {}: {} ({})", i + 1, commit.id, commit.timestamp);
    }

    println!("\n7. Creating more files...");
    fs.open("logs/debug.log", OpenMode::Create).await?;
    fs.open("logs/error.log", OpenMode::Create).await?;
    fs.open("state.json", OpenMode::Create).await?;
    println!("   ✓ Created multiple files");

    println!("\n8. Listing all files...");
    let all_files = fs.list("").await?;
    println!("   Files: {:?}", all_files);

    println!("\n9. Listing logs directory...");
    let logs = fs.list("logs").await?;
    println!("   Log files: {:?}", logs);

    println!("\n10. Checking file existence...");
    println!("   memory.txt exists: {}", fs.exists("memory.txt").await?);
    println!("   nonexistent.txt exists: {}", fs.exists("nonexistent.txt").await?);

    println!("\n11. Reopening existing file...");
    drop(memory_file);
    let file = fs.open("memory.txt", OpenMode::ReadOnly).await?;
    let content = file.read().await?;
    println!("   Content: {}", String::from_utf8_lossy(&content));

    println!("\n12. Testing large content (chunking)...");
    let mut large_file = fs.open("large.txt", OpenMode::Create).await?;
    let large_content = vec![b'x'; 1000]; // Larger than tweet size
    large_file.write(&large_content).await?;
    println!("   ✓ Wrote {} bytes (auto-chunked)", large_content.len());

    let read_back = large_file.read().await?;
    println!("   ✓ Read back {} bytes", read_back.len());
    assert_eq!(read_back, large_content);

    println!("\n=== Example Complete ===");
    println!("\nKey features demonstrated:");
    println!("  • File creation and writing");
    println!("  • Commit chain (version history)");
    println!("  • File listing and directory navigation");
    println!("  • Content caching");
    println!("  • Automatic chunking for large content");
    println!("  • Reopening files");

    Ok(())
}
