//! Merge strategies for concurrent writes

use crate::dag::commit::Commit;
use crate::error::Result;

/// Trait for implementing custom merge strategies
pub trait MergeStrategy {
    /// Merge two conflicting commits
    fn merge(&self, base: &Commit, left: &Commit, right: &Commit) -> Result<Vec<u8>>;
}

/// Last-writer-wins merge strategy (default for v0.1)
pub struct LastWriterWins;

impl MergeStrategy for LastWriterWins {
    fn merge(&self, _base: &Commit, _left: &Commit, _right: &Commit) -> Result<Vec<u8>> {
        // For v0.1, simply take the latest commit
        todo!("Implement last-writer-wins merge")
    }
}
