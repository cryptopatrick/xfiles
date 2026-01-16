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
    /// Finds commits with no children (terminal nodes in the DAG)
    pub fn find_head(&self, start: &TweetId) -> Result<&Commit> {
        // Find all descendants of the start commit
        let mut reachable = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(start.clone());
        reachable.insert(start.clone());

        while let Some(current_id) = queue.pop_front() {
            if self.commits.contains_key(&current_id) {
                // Find children of this commit
                for (id, candidate) in &self.commits {
                    if candidate.parents.contains(&current_id) && !reachable.contains(id) {
                        reachable.insert(id.clone());
                        queue.push_back(id.clone());
                    }
                }
            }
        }

        // Find commits in the reachable set that have no children (heads)
        let mut heads = Vec::new();
        for id in &reachable {
            let has_children = self.commits.values()
                .any(|c| c.parents.contains(id));

            if !has_children {
                if let Some(commit) = self.commits.get(id) {
                    heads.push(commit);
                }
            }
        }

        // Return the most recent head
        heads.into_iter()
            .max_by_key(|c| c.timestamp)
            .ok_or_else(|| crate::error::XFilesError::CommitNotFound(start.clone()))
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
    /// Returns all head commits reachable from the root
    pub fn detect_forks(&self, root: &TweetId) -> Result<Vec<TweetId>> {
        // Find all descendants of the root
        let mut reachable = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(root.clone());
        reachable.insert(root.clone());

        while let Some(current_id) = queue.pop_front() {
            if self.commits.contains_key(&current_id) {
                // Add children to the reachable set
                for (id, candidate) in &self.commits {
                    if candidate.parents.contains(&current_id) && !reachable.contains(id) {
                        reachable.insert(id.clone());
                        queue.push_back(id.clone());
                    }
                }
            }
        }

        // Find all heads (commits with no children) in the reachable set
        let mut heads = Vec::new();
        for id in &reachable {
            let has_children = self.commits.values()
                .any(|c| c.parents.contains(id));

            if !has_children {
                heads.push(id.clone());
            }
        }

        Ok(heads)
    }
}

impl Default for CommitGraph {
    fn default() -> Self {
        Self::new()
    }
}
