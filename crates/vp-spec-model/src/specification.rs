//! Root specification model aggregate.

use std::path::PathBuf;

use crate::registry_set::RegistrySet;

/// Immutable typed view of a specification checkout (initial milestone: registries only).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Specification {
    pub spec_root: PathBuf,
    pub registry_set: RegistrySet,
}

impl Specification {
    pub(crate) fn new(spec_root: PathBuf, registry_set: RegistrySet) -> Self {
        Self {
            spec_root,
            registry_set,
        }
    }
}
