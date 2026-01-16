//! File operations and XFile implementation

use crate::dag::commit::{Commit, TweetId};
use crate::error::Result;
use crate::remote::RemoteAdapter;
use crate::store::{SqliteStore, cache::ContentCache};
use crate::fs::chunk::chunk_content;
use crate::util::hash::compute_hash;
use std::sync::Arc;

/// Represents a file in the xfiles filesystem
pub struct XFile {
    /// Path to the file
    pub path: String,
    /// Current head commit
    pub head: TweetId,
    /// SQLite store
    store: Arc<SqliteStore>,
    /// Remote adapter
    adapter: Arc<dyn RemoteAdapter>,
    /// Content cache
    cache: Arc<ContentCache>,
    /// Author username
    author: String,
}

impl XFile {
    /// Create a new XFile instance
    pub fn new(
        path: String,
        head: TweetId,
        store: Arc<SqliteStore>,
        adapter: Arc<dyn RemoteAdapter>,
        cache: Arc<ContentCache>,
        author: String,
    ) -> Self {
        Self {
            path,
            head,
            store,
            adapter,
            cache,
            author,
        }
    }

    /// Read the current contents of the file
    pub async fn read(&self) -> Result<Vec<u8>> {
        // Check cache first
        if let Some(content) = self.cache.get(&self.head) {
            return Ok(content);
        }

        // Fetch from remote
        let content = self.adapter.fetch(&self.head).await?;

        // Cache it
        self.cache.put(self.head.clone(), content.clone());

        Ok(content)
    }

    /// Write new content to the file (creates a new commit)
    pub async fn write(&mut self, data: impl AsRef<[u8]>) -> Result<()> {
        let data = data.as_ref();
        let hash = compute_hash(data);

        // Chunk the content if needed
        let chunks = chunk_content(data)?;

        // Post chunks to remote
        let mut chunk_ids = Vec::new();
        if chunks.len() == 1 {
            // Single chunk - post as reply to current head
            let id = self.adapter.store_reply(&self.head, &chunks[0]).await?;
            chunk_ids.push(id.clone());

            // Create and store commit
            let commit = Commit::new(
                id.clone(),
                vec![self.head.clone()],
                self.author.clone(),
                hash.clone(),
                "text/plain".to_string(),
                data.len(),
            );

            self.store.store_commit(&commit).await?;
            self.store.set_head(&id).await?;

            // Update head
            self.head = id;
        } else {
            // Multiple chunks - post first as reply, rest as chain
            let first_id = self.adapter.store_reply(&self.head, &chunks[0]).await?;
            chunk_ids.push(first_id.clone());

            let mut prev_id = first_id.clone();
            for chunk in chunks.iter().skip(1) {
                let id = self.adapter.store_reply(&prev_id, chunk).await?;
                chunk_ids.push(id.clone());
                prev_id = id;
            }

            // Create commit pointing to first chunk
            let commit = Commit::new(
                first_id.clone(),
                vec![self.head.clone()],
                self.author.clone(),
                hash.clone(),
                "text/plain".to_string(),
                data.len(),
            );

            self.store.store_commit(&commit).await?;
            self.store.set_head(&first_id).await?;

            // Update head
            self.head = first_id;
        }

        // Cache the content
        self.cache.put(self.head.clone(), data.to_vec());

        Ok(())
    }

    /// Delete the file (creates a tombstone commit)
    pub async fn delete(&mut self) -> Result<()> {
        // Post a tombstone marker
        let tombstone = b"[DELETED]";
        let id = self.adapter.store_reply(&self.head, tombstone).await?;

        let commit = Commit::new(
            id.clone(),
            vec![self.head.clone()],
            self.author.clone(),
            compute_hash(tombstone),
            "application/x-xfiles-tombstone".to_string(),
            tombstone.len(),
        );

        self.store.store_commit(&commit).await?;
        self.store.set_head(&id).await?;

        self.head = id;

        Ok(())
    }

    /// Get the current head commit ID
    pub fn head(&self) -> &TweetId {
        &self.head
    }

    /// Get the file path
    pub fn path(&self) -> &str {
        &self.path
    }
}
