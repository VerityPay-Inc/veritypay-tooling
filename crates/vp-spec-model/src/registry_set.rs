//! Registry aggregate types.

use crate::rfc::RfcRegistry;
use crate::terminology::TerminologyRegistry;

/// Machine-readable registries loaded from `spec/`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrySet {
    pub terminology: TerminologyRegistry,
    pub rfcs: RfcRegistry,
}

impl RegistrySet {
    pub(crate) fn new(terminology: TerminologyRegistry, rfcs: RfcRegistry) -> Self {
        Self { terminology, rfcs }
    }
}
