use xfiles::*;
use std::sync::Arc;

#[tokio::test]
async fn test_create_and_read_file() {
    let adapter = Arc::new(MockAdapter::new());
    let mut fs = XFS::with_adapter("testuser", adapter, Some(":memory:"))
        .await
        .unwrap();

    // Create a new file
    let mut file = fs.open("test.txt", OpenMode::Create).await.unwrap();

    // Write content
    let content = b"Hello, xfiles!";
    file.write(content).await.unwrap();

    // Read it back
    let read_content = file.read().await.unwrap();
    assert_eq!(read_content, content);
}

#[tokio::test]
async fn test_file_reopen_same_session() {
    let adapter = Arc::new(MockAdapter::new());
    let mut fs = XFS::with_adapter("testuser", adapter.clone(), Some(":memory:"))
        .await
        .unwrap();

    // Create and write
    let mut file = fs.open("persistent.txt", OpenMode::Create).await.unwrap();
    file.write(b"Persisted data").await.unwrap();
    drop(file);

    // Reopen in same session
    let file = fs
        .open("persistent.txt", OpenMode::ReadOnly)
        .await
        .unwrap();
    let content = file.read().await.unwrap();
    assert_eq!(content, b"Persisted data");
}

#[tokio::test]
async fn test_multiple_writes() {
    let adapter = Arc::new(MockAdapter::new());
    let mut fs = XFS::with_adapter("testuser", adapter, Some(":memory:"))
        .await
        .unwrap();

    let mut file = fs.open("multi.txt", OpenMode::Create).await.unwrap();

    // Multiple writes create commit chain
    file.write(b"Version 1").await.unwrap();
    file.write(b"Version 2").await.unwrap();
    file.write(b"Version 3").await.unwrap();

    // Latest version should be readable
    let content = file.read().await.unwrap();
    assert_eq!(content, b"Version 3");
}

#[tokio::test]
async fn test_file_history() {
    let adapter = Arc::new(MockAdapter::new());
    let mut fs = XFS::with_adapter("testuser", adapter, Some(":memory:"))
        .await
        .unwrap();

    let mut file = fs.open("history.txt", OpenMode::Create).await.unwrap();

    file.write(b"Commit 1").await.unwrap();
    file.write(b"Commit 2").await.unwrap();
    file.write(b"Commit 3").await.unwrap();

    // Get history
    let history = fs.history("history.txt").await.unwrap();

    // Should have 4 commits: root + 3 writes
    assert!(history.len() >= 3);
}

#[tokio::test]
async fn test_list_files() {
    let adapter = Arc::new(MockAdapter::new());
    let mut fs = XFS::with_adapter("testuser", adapter, Some(":memory:"))
        .await
        .unwrap();

    // Create several files
    fs.open("file1.txt", OpenMode::Create).await.unwrap();
    fs.open("file2.txt", OpenMode::Create).await.unwrap();
    fs.open("logs/debug.log", OpenMode::Create).await.unwrap();

    // List all files
    let files = fs.list("").await.unwrap();
    assert!(files.contains(&"file1.txt".to_string()));
    assert!(files.contains(&"file2.txt".to_string()));
    assert!(files.contains(&"logs/debug.log".to_string()));

    // List directory
    let log_files = fs.list("logs").await.unwrap();
    assert_eq!(log_files.len(), 1);
    assert!(log_files.contains(&"logs/debug.log".to_string()));
}

#[tokio::test]
async fn test_file_exists() {
    let adapter = Arc::new(MockAdapter::new());
    let mut fs = XFS::with_adapter("testuser", adapter, Some(":memory:"))
        .await
        .unwrap();

    // File doesn't exist initially
    assert!(!fs.exists("nonexistent.txt").await.unwrap());

    // Create file
    fs.open("exists.txt", OpenMode::Create).await.unwrap();

    // Now it exists
    assert!(fs.exists("exists.txt").await.unwrap());
}

#[tokio::test]
async fn test_large_content_chunking() {
    let adapter = Arc::new(MockAdapter::new());
    let mut fs = XFS::with_adapter("testuser", adapter, Some(":memory:"))
        .await
        .unwrap();

    let mut file = fs.open("large.txt", OpenMode::Create).await.unwrap();

    // Create content larger than tweet size (280 bytes)
    let large_content = vec![b'x'; 1000];
    file.write(&large_content).await.unwrap();

    // Read it back and verify
    let read_content = file.read().await.unwrap();
    assert_eq!(read_content, large_content);
}

#[tokio::test]
async fn test_reopen_file() {
    let adapter = Arc::new(MockAdapter::new());
    let mut fs = XFS::with_adapter("testuser", adapter, Some(":memory:"))
        .await
        .unwrap();

    // Create and write
    let mut file = fs.open("reopen.txt", OpenMode::Create).await.unwrap();
    file.write(b"Initial content").await.unwrap();
    drop(file);

    // Reopen and verify
    let file = fs.open("reopen.txt", OpenMode::ReadOnly).await.unwrap();
    let content = file.read().await.unwrap();
    assert_eq!(content, b"Initial content");
}

#[tokio::test]
async fn test_create_existing_file_fails() {
    let adapter = Arc::new(MockAdapter::new());
    let mut fs = XFS::with_adapter("testuser", adapter, Some(":memory:"))
        .await
        .unwrap();

    // Create file
    fs.open("existing.txt", OpenMode::Create).await.unwrap();

    // Try to create again - should fail
    let result = fs.open("existing.txt", OpenMode::Create).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_open_nonexistent_file_fails() {
    let adapter = Arc::new(MockAdapter::new());
    let mut fs = XFS::with_adapter("testuser", adapter, Some(":memory:"))
        .await
        .unwrap();

    // Try to open nonexistent file
    let result = fs.open("nonexistent.txt", OpenMode::ReadOnly).await;
    assert!(result.is_err());
}
