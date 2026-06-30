//! Immutable validation run configuration (ADR-0003).

use std::path::PathBuf;

/// Read-only inputs shared with every validator in a run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationContext {
    pub spec_root: PathBuf,
    pub strict: bool,
}

impl ValidationContext {
    pub fn new(spec_root: impl Into<PathBuf>) -> Self {
        Self {
            spec_root: spec_root.into(),
            strict: false,
        }
    }

    pub fn strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }
}
