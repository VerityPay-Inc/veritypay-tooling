//! Discovered reference value type.

use std::path::PathBuf;

use vp_diagnostics::Location;

use super::kind::ReferenceKind;

/// A reference found in a source document (not yet validated).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredReference {
    pub kind: ReferenceKind,
    pub target: String,
    pub source_file: PathBuf,
    pub location: Location,
}

impl DiscoveredReference {
    pub fn new(
        kind: ReferenceKind,
        target: impl Into<String>,
        source_file: impl Into<PathBuf>,
        location: Location,
    ) -> Self {
        Self {
            kind,
            target: target.into(),
            source_file: source_file.into(),
            location,
        }
    }
}
