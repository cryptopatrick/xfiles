//! Commit data structures and operations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Type alias for Twitter tweet IDs
pub type TweetId = String;

/// Type alias for content hashes
pub type Hash = String;

/// Represents a single commit in the DAG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    /// Tweet ID of this commit
    pub id: TweetId,

    /// Parent commit ID(s)
    pub parents: Vec<TweetId>,

    /// Timestamp of the commit
    pub timestamp: DateTime<Utc>,

    /// Content hash (blake3)
    pub hash: Hash,

    /// Author user ID
    pub author: String,

    /// MIME type of content
    pub mime: String,

    /// Size of content in bytes
    pub size: usize,

    /// Whether this is a head commit
    pub is_head: bool,
}

impl Commit {
    /// Create a new commit
    pub fn new(
        id: TweetId,
        parents: Vec<TweetId>,
        author: String,
        hash: Hash,
        mime: String,
        size: usize,
    ) -> Self {
        Self {
            id,
            parents,
            timestamp: Utc::now(),
            hash,
            author,
            mime,
            size,
            is_head: false,
        }
    }
}

/// Content reference for a commit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentRef {
    /// Tweet IDs of chunks (if chunked)
    pub chunks: Vec<TweetId>,

    /// Hash of complete content
    pub hash: Hash,

    /// Total size
    pub size: usize,
}
