//! Reference discovery trait.

use std::path::Path;

use crate::reference::Reference;

/// Discovers references within a document without validating them.
pub trait ReferenceDiscovery {
    /// Return every reference found in `content` from `source_file`.
    fn discover(&self, source_file: &Path, content: &str) -> Vec<Reference>;
}
