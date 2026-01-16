//! # xfiles - Twitter as a filesystem for agents
//!
//! `xfiles` is a Rust crate that treats Twitter as a public, append-only,
//! log-structured filesystem. Tweets become "files", replies become "commits",
//! and a local SQLite index keeps traversal fast.
//!
//! ## Features
//!
//! - Tweet = file root
//! - Reply = commit
//! - Append-only versioning
//! - SQLite graph index
//! - History, read, write APIs
//! - Chunking for long content
//!
//! ## Example
//!
//! ```rust,no_run
//! use xfiles::{XFS, OpenMode};
//!
//! # async fn example() -> xfiles::error::Result<()> {
//! let mut fs = XFS::connect("@myagent", "api_key", "api_secret").await?;
//! let mut file = fs.open("memory.txt", OpenMode::Create).await?;
//! file.write(b"Day 1: Agent bootstrapped").await?;
//! # Ok(())
//! # }
//! ```

// Public modules
pub mod error;
pub mod fs;
pub mod dag;
pub mod store;
pub mod remote;
pub mod util;

// Re-export commonly used types
pub use error::{Result, XFilesError};
pub use fs::{XFile, chunk::TWEET_MAX_SIZE};
pub use dag::{Commit, TweetId};
pub use remote::RemoteAdapter;

use store::SqliteStore;
use remote::TwitterAdapter;
use std::sync::Arc;

/// File open mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenMode {
    /// Create a new file (fails if exists)
    Create,
    /// Open existing file for reading
    ReadOnly,
    /// Open existing file for reading and writing
    ReadWrite,
}

/// Main filesystem interface
pub struct XFS {
    /// Twitter username
    user: String,
    /// SQLite store
    store: Arc<SqliteStore>,
    /// Twitter API adapter
    adapter: Arc<TwitterAdapter>,
}

impl XFS {
    /// Connect to xfiles with Twitter credentials
    ///
    /// # Arguments
    ///
    /// * `user` - Twitter username (e.g., "@myagent")
    /// * `api_key` - Twitter API key
    /// * `api_secret` - Twitter API secret
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use xfiles::XFS;
    /// # async fn example() -> xfiles::error::Result<()> {
    /// let fs = XFS::connect("@myagent", "key", "secret").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect(user: &str, api_key: &str, api_secret: &str) -> Result<Self> {
        let user = user.trim_start_matches('@').to_string();

        // Initialize SQLite store
        let db_path = format!("xfiles_{}.db", user);
        let store = SqliteStore::new(&format!("sqlite://{}", db_path)).await?;
        store.init_schema().await?;

        // Initialize Twitter adapter
        let adapter = TwitterAdapter::new(api_key.to_string(), api_secret.to_string());

        Ok(Self {
            user,
            store: Arc::new(store),
            adapter: Arc::new(adapter),
        })
    }

    /// Open a file
    ///
    /// # Arguments
    ///
    /// * `path` - File path (e.g., "memory.txt" or "logs/agent.log")
    /// * `mode` - How to open the file
    pub async fn open(&mut self, _path: &str, _mode: OpenMode) -> Result<XFile> {
        todo!("Implement file open")
    }

    /// List files in a directory
    ///
    /// # Arguments
    ///
    /// * `path` - Directory path (use "" or "/" for root)
    pub async fn list(&self, _path: &str) -> Result<Vec<String>> {
        todo!("Implement directory listing")
    }

    /// Get the history of a file
    ///
    /// Returns all commits in chronological order
    pub async fn history(&self, path: &str) -> Result<Vec<Commit>> {
        fs::history::get_history(path).await
    }

    /// Check if a file exists
    pub async fn exists(&self, _path: &str) -> Result<bool> {
        todo!("Implement file existence check")
    }

    /// Get the current user
    pub fn user(&self) -> &str {
        &self.user
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_mode() {
        assert_eq!(OpenMode::Create, OpenMode::Create);
        assert_ne!(OpenMode::Create, OpenMode::ReadOnly);
    }
}
