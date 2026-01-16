//! Diff operations between commits

use crate::dag::commit::Commit;
use crate::error::Result;

/// Represents a difference between two commits
#[derive(Debug)]
pub enum DiffOp {
    /// Content was added
    Add(Vec<u8>),
    /// Content was removed
    Remove(Vec<u8>),
    /// Content was modified
    Modify { old: Vec<u8>, new: Vec<u8> },
}

/// Compute the diff between two commits
pub fn diff_commits(_old: &Commit, _new: &Commit) -> Result<Vec<DiffOp>> {
    todo!("Implement commit diff")
}

/// Apply a diff to content
pub fn apply_diff(_content: &[u8], _diff: &[DiffOp]) -> Result<Vec<u8>> {
    todo!("Implement diff application")
}
