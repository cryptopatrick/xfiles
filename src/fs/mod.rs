//! Filesystem abstraction layer
//!
//! This module provides the logical filesystem API on top of the DAG layer.

pub mod file;
pub mod history;
pub mod merge;
pub mod chunk;

pub use file::XFile;
