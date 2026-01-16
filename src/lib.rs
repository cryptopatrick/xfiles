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
pub use remote::{RemoteAdapter, MockAdapter};

use store::{SqliteStore, ContentCache};
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
    /// Remote API adapter
    adapter: Arc<dyn RemoteAdapter>,
    /// Content cache
    cache: Arc<ContentCache>,
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

        // Initialize content cache
        let cache = ContentCache::new();

        Ok(Self {
            user,
            store: Arc::new(store),
            adapter: Arc::new(adapter),
            cache: Arc::new(cache),
        })
    }

    /// Create XFS with a custom adapter (useful for testing)
    ///
    /// # Arguments
    ///
    /// * `user` - Username
    /// * `adapter` - Custom RemoteAdapter implementation
    /// * `db_path` - Optional custom database path
    pub async fn with_adapter(
        user: &str,
        adapter: Arc<dyn RemoteAdapter>,
        db_path: Option<&str>,
    ) -> Result<Self> {
        let user = user.trim_start_matches('@').to_string();

        // Initialize SQLite store
        let default_db_path = format!("xfiles_{}.db", user);
        let db_path = db_path.unwrap_or(&default_db_path);
        let store = SqliteStore::new(&format!("sqlite://{}", db_path)).await?;
        store.init_schema().await?;

        // Initialize content cache
        let cache = ContentCache::new();

        Ok(Self {
            user,
            store: Arc::new(store),
            adapter,
            cache: Arc::new(cache),
        })
    }

    /// Open a file
    ///
    /// # Arguments
    ///
    /// * `path` - File path (e.g., "memory.txt" or "logs/agent.log")
    /// * `mode` - How to open the file
    pub async fn open(&mut self, path: &str, mode: OpenMode) -> Result<XFile> {
        // Check if file exists
        let root = self.store.get_file_root(path).await?;

        match (root, mode) {
            (Some(root_id), OpenMode::Create) => {
                // File already exists
                Err(XFilesError::Other(format!("File already exists: {}", path)))
            }
            (None, OpenMode::Create) => {
                // Create new file - post root tweet
                let initial_content = b"";
                let root_id = self.adapter.store(initial_content).await?;

                // Create root commit
                let commit = Commit::new(
                    root_id.clone(),
                    Vec::new(), // No parents for root
                    self.user.clone(),
                    util::hash::compute_hash(initial_content),
                    "text/plain".to_string(),
                    0,
                );

                self.store.store_commit(&commit).await?;
                self.store.register_file(path, &root_id).await?;
                self.store.set_head(&root_id).await?;

                Ok(XFile::new(
                    path.to_string(),
                    root_id,
                    self.store.clone(),
                    self.adapter.clone(),
                    self.cache.clone(),
                    self.user.clone(),
                ))
            }
            (Some(root_id), OpenMode::ReadOnly) | (Some(root_id), OpenMode::ReadWrite) => {
                // Open existing file - find current head
                let head = self.find_head(&root_id).await?;

                Ok(XFile::new(
                    path.to_string(),
                    head,
                    self.store.clone(),
                    self.adapter.clone(),
                    self.cache.clone(),
                    self.user.clone(),
                ))
            }
            (None, OpenMode::ReadOnly) | (None, OpenMode::ReadWrite) => {
                // File doesn't exist
                Err(XFilesError::FileNotFound(path.to_string()))
            }
        }
    }

    /// Find the current head commit for a file
    async fn find_head(&self, root_id: &TweetId) -> Result<TweetId> {
        // Get all replies to find the head
        let replies = self.adapter.fetch_replies(root_id).await?;

        if replies.is_empty() {
            // Root is the head
            return Ok(root_id.clone());
        }

        // Build commit graph and find head
        let mut graph = dag::CommitGraph::new();

        // Add root
        if let Some(root_commit) = self.store.get_commit(root_id).await? {
            graph.add_commit(root_commit);
        }

        // Add all descendants
        let mut to_process = replies.clone();
        while !to_process.is_empty() {
            let id = to_process.remove(0);

            if let Some(commit) = self.store.get_commit(&id).await? {
                graph.add_commit(commit.clone());

                // Get replies to this commit
                let child_replies = self.adapter.fetch_replies(&id).await?;
                for reply_id in child_replies {
                    if !to_process.contains(&reply_id) {
                        to_process.push(reply_id);
                    }
                }
            }
        }

        // Find head
        let head_commit = graph.find_head(root_id)?;
        Ok(head_commit.id.clone())
    }

    /// List files in a directory
    ///
    /// # Arguments
    ///
    /// * `path` - Directory path (use "" or "/" for root)
    pub async fn list(&self, path: &str) -> Result<Vec<String>> {
        let all_paths = self.store.list_files().await?;

        if path.is_empty() || path == "/" {
            // Return all files
            Ok(all_paths)
        } else {
            // Filter by directory prefix
            let prefix = if path.ends_with('/') {
                path.to_string()
            } else {
                format!("{}/", path)
            };

            Ok(all_paths
                .into_iter()
                .filter(|p| p.starts_with(&prefix))
                .collect())
        }
    }

    /// Get the history of a file
    ///
    /// Returns all commits in chronological order
    pub async fn history(&self, path: &str) -> Result<Vec<Commit>> {
        let root = self.store.get_file_root(path).await?
            .ok_or_else(|| XFilesError::FileNotFound(path.to_string()))?;

        // Get all commits starting from root
        let mut commits = Vec::new();
        let mut to_process = vec![root.clone()];
        let mut processed = std::collections::HashSet::new();

        while let Some(id) = to_process.pop() {
            if processed.contains(&id) {
                continue;
            }
            processed.insert(id.clone());

            if let Some(commit) = self.store.get_commit(&id).await? {
                // Get children
                let children = self.store.get_children(&id).await?;
                for child in children {
                    to_process.push(child.id);
                }

                commits.push(commit);
            }
        }

        // Sort by timestamp
        commits.sort_by_key(|c| c.timestamp);

        Ok(commits)
    }

    /// Check if a file exists
    pub async fn exists(&self, path: &str) -> Result<bool> {
        self.store.file_exists(path).await
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
