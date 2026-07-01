//! Root specification model aggregate.

use std::path::PathBuf;

use crate::document_corpus::DocumentCorpus;
use crate::reference_graph::ReferenceGraph;
use crate::registry_set::RegistrySet;

/// Immutable typed view of a specification checkout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Specification {
    pub spec_root: PathBuf,
    pub registry_set: RegistrySet,
    pub document_corpus: DocumentCorpus,
    reference_graph: ReferenceGraph,
}

impl Specification {
    pub(crate) fn new(
        spec_root: PathBuf,
        registry_set: RegistrySet,
        document_corpus: DocumentCorpus,
        reference_graph: ReferenceGraph,
    ) -> Self {
        Self {
            spec_root,
            registry_set,
            document_corpus,
            reference_graph,
        }
    }

    pub fn reference_graph(&self) -> &ReferenceGraph {
        &self.reference_graph
    }
}
