//! Root specification model aggregate.

use std::path::PathBuf;

use crate::document_corpus::DocumentCorpus;
use crate::registry_set::RegistrySet;

/// Immutable typed view of a specification checkout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Specification {
    pub spec_root: PathBuf,
    pub registry_set: RegistrySet,
    pub document_corpus: DocumentCorpus,
}

impl Specification {
    pub(crate) fn new(
        spec_root: PathBuf,
        registry_set: RegistrySet,
        document_corpus: DocumentCorpus,
    ) -> Self {
        Self {
            spec_root,
            registry_set,
            document_corpus,
        }
    }
}
