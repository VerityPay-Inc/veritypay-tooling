//! Spec root path normalization.

use std::path::{Path, PathBuf};

/// Resolve `spec_root` to an absolute path when the checkout exists on disk.
pub fn canonicalize_spec_root(spec_root: impl AsRef<Path>) -> PathBuf {
    let path = spec_root.as_ref().to_path_buf();
    std::fs::canonicalize(&path).unwrap_or(path)
}
