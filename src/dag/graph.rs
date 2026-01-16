//! DAG graph operations and traversal

use crate::dag::commit::{Commit, TweetId};
use crate::error::Result;
use std::collections::{HashMap, HashSet, VecDeque};

/// Manages the commit graph
pub struct CommitGraph {
    /// In-memory commit cache
    commits: HashMap<TweetId, Commit>,
}

impl CommitGraph {
    /// Create a new empty graph
    pub fn new() -> Self {
        Self {
            commits: HashMap::new(),
        }
    }

    /// Add a commit to the graph
    pub fn add_commit(&mut self, commit: Commit) {
        self.commits.insert(commit.id.clone(), commit);
    }

    /// Get a commit by ID
    pub fn get_commit(&self, id: &TweetId) -> Option<&Commit> {
        self.commits.get(id)
    }

    /// Find the latest commit in a chain (BFS traversal)
    pub fn find_head(&self, _start: &TweetId) -> Result<&Commit> {
        todo!("Implement BFS to find head commit")
    }

    /// Get all ancestors of a commit
    pub fn get_ancestors(&self, id: &TweetId) -> Result<Vec<&Commit>> {
        let mut ancestors = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(id.clone());
        visited.insert(id.clone());

        while let Some(current_id) = queue.pop_front() {
            if let Some(commit) = self.commits.get(&current_id) {
                ancestors.push(commit);

                for parent_id in &commit.parents {
                    if !visited.contains(parent_id) {
                        visited.insert(parent_id.clone());
                        queue.push_back(parent_id.clone());
                    }
                }
            }
        }

        Ok(ancestors)
    }

    /// Detect if there are multiple heads (fork)
    pub fn detect_forks(&self, _root: &TweetId) -> Result<Vec<TweetId>> {
        todo!("Implement fork detection")
    }
}

impl Default for CommitGraph {
    fn default() -> Self {
        Self::new()
    }
}
