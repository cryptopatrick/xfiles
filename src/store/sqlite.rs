//! SQLite database operations

use crate::dag::commit::{Commit, TweetId};
use crate::error::Result;
use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool, sqlite::SqlitePoolOptions};

/// SQLite store for commit graph and metadata
pub struct SqliteStore {
    pool: SqlitePool,
}

impl SqliteStore {
    /// Create a new SQLite store
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    /// Initialize the database schema
    pub async fn init_schema(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS commits (
                tweet_id TEXT PRIMARY KEY,
                parent_id TEXT,
                timestamp INTEGER NOT NULL,
                author TEXT NOT NULL,
                hash TEXT NOT NULL,
                mime TEXT NOT NULL,
                size INTEGER NOT NULL,
                head BOOLEAN DEFAULT 0
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS chunks (
                tweet_id TEXT PRIMARY KEY,
                parent_commit TEXT NOT NULL,
                idx INTEGER NOT NULL,
                size INTEGER NOT NULL,
                hash TEXT NOT NULL,
                FOREIGN KEY (parent_commit) REFERENCES commits(tweet_id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_parent ON commits(parent_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_timestamp ON commits(timestamp)")
            .execute(&self.pool)
            .await?;

        // Create files table for path-to-root mapping
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS files (
                path TEXT PRIMARY KEY,
                root_tweet_id TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Store a commit in the database
    pub async fn store_commit(&self, commit: &Commit) -> Result<()> {
        // Serialize parents as JSON for storage (supports multiple parents for future merging)
        let parents_json = serde_json::to_string(&commit.parents)?;

        sqlx::query(
            r#"
            INSERT INTO commits (tweet_id, parent_id, timestamp, author, hash, mime, size, head)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(tweet_id) DO UPDATE SET
                parent_id = excluded.parent_id,
                timestamp = excluded.timestamp,
                author = excluded.author,
                hash = excluded.hash,
                mime = excluded.mime,
                size = excluded.size,
                head = excluded.head
            "#,
        )
        .bind(&commit.id)
        .bind(parents_json)
        .bind(commit.timestamp.timestamp())
        .bind(&commit.author)
        .bind(&commit.hash)
        .bind(&commit.mime)
        .bind(commit.size as i64)
        .bind(commit.is_head)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Retrieve a commit by ID
    pub async fn get_commit(&self, id: &TweetId) -> Result<Option<Commit>> {
        let row = sqlx::query(
            r#"
            SELECT tweet_id, parent_id, timestamp, author, hash, mime, size, head
            FROM commits
            WHERE tweet_id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let parents_json: String = row.try_get("parent_id")?;
            let parents: Vec<TweetId> = serde_json::from_str(&parents_json)?;
            let timestamp_secs: i64 = row.try_get("timestamp")?;

            Ok(Some(Commit {
                id: row.try_get("tweet_id")?,
                parents,
                timestamp: DateTime::from_timestamp(timestamp_secs, 0)
                    .unwrap_or_else(|| Utc::now()),
                hash: row.try_get("hash")?,
                author: row.try_get("author")?,
                mime: row.try_get("mime")?,
                size: row.try_get::<i64, _>("size")? as usize,
                is_head: row.try_get("head")?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Get all commits with a specific parent
    pub async fn get_children(&self, parent_id: &TweetId) -> Result<Vec<Commit>> {
        let rows = sqlx::query(
            r#"
            SELECT tweet_id, parent_id, timestamp, author, hash, mime, size, head
            FROM commits
            WHERE parent_id LIKE ?
            "#,
        )
        .bind(format!("%\"{}\"%%", parent_id))
        .fetch_all(&self.pool)
        .await?;

        let mut commits = Vec::new();
        for row in rows {
            let parents_json: String = row.try_get("parent_id")?;
            let parents: Vec<TweetId> = serde_json::from_str(&parents_json)?;
            let timestamp_secs: i64 = row.try_get("timestamp")?;

            commits.push(Commit {
                id: row.try_get("tweet_id")?,
                parents,
                timestamp: DateTime::from_timestamp(timestamp_secs, 0)
                    .unwrap_or_else(|| Utc::now()),
                hash: row.try_get("hash")?,
                author: row.try_get("author")?,
                mime: row.try_get("mime")?,
                size: row.try_get::<i64, _>("size")? as usize,
                is_head: row.try_get("head")?,
            });
        }

        Ok(commits)
    }

    /// Mark a commit as head
    pub async fn set_head(&self, id: &TweetId) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE commits
            SET head = 1
            WHERE tweet_id = ?
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get all head commits
    pub async fn get_heads(&self) -> Result<Vec<Commit>> {
        let rows = sqlx::query(
            r#"
            SELECT tweet_id, parent_id, timestamp, author, hash, mime, size, head
            FROM commits
            WHERE head = 1
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut commits = Vec::new();
        for row in rows {
            let parents_json: String = row.try_get("parent_id")?;
            let parents: Vec<TweetId> = serde_json::from_str(&parents_json)?;
            let timestamp_secs: i64 = row.try_get("timestamp")?;

            commits.push(Commit {
                id: row.try_get("tweet_id")?,
                parents,
                timestamp: DateTime::from_timestamp(timestamp_secs, 0)
                    .unwrap_or_else(|| Utc::now()),
                hash: row.try_get("hash")?,
                author: row.try_get("author")?,
                mime: row.try_get("mime")?,
                size: row.try_get::<i64, _>("size")? as usize,
                is_head: row.try_get("head")?,
            });
        }

        Ok(commits)
    }

    /// Register a file path with its root tweet ID
    pub async fn register_file(&self, path: &str, root_tweet_id: &TweetId) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO files (path, root_tweet_id, created_at)
            VALUES (?, ?, ?)
            ON CONFLICT(path) DO UPDATE SET
                root_tweet_id = excluded.root_tweet_id
            "#,
        )
        .bind(path)
        .bind(root_tweet_id)
        .bind(Utc::now().timestamp())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get the root tweet ID for a file path
    pub async fn get_file_root(&self, path: &str) -> Result<Option<TweetId>> {
        let row = sqlx::query(
            r#"
            SELECT root_tweet_id
            FROM files
            WHERE path = ?
            "#,
        )
        .bind(path)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(row.try_get("root_tweet_id")?))
        } else {
            Ok(None)
        }
    }

    /// List all registered file paths
    pub async fn list_files(&self) -> Result<Vec<String>> {
        let rows = sqlx::query(
            r#"
            SELECT path
            FROM files
            ORDER BY path
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut paths = Vec::new();
        for row in rows {
            paths.push(row.try_get("path")?);
        }

        Ok(paths)
    }

    /// Check if a file exists
    pub async fn file_exists(&self, path: &str) -> Result<bool> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM files
            WHERE path = ?
            "#,
        )
        .bind(path)
        .fetch_one(&self.pool)
        .await?;

        let count: i64 = row.try_get("count")?;
        Ok(count > 0)
    }
}
