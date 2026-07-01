//! Typed specification model for `veritypay-spec`.

mod builder;
mod corpus;
mod document;
mod document_corpus;
mod error;
mod link_resolve;
mod reference_discovery;
mod reference_graph;
mod registry_set;
mod rfc;
mod specification;
mod terminology;

pub use builder::SpecificationBuilder;
pub use document::{DocumentFrontMatter, DocumentSection, SpecificationDocument};
pub use document_corpus::DocumentCorpus;
pub use error::BuildError;
pub use link_resolve::{normalize_path, resolve_relative_link, split_link_target};
pub use reference_discovery::{
    DiscoveredReference, MarkdownDiscovery, ReferenceDiscovery, ReferenceKind,
    SECTION_ID_PREFIXES,
};
pub use reference_graph::{
    ReferenceEdge, ReferenceGraph, ReferenceNode, ReferenceNodeKind,
};
pub use registry_set::RegistrySet;
pub use rfc::{RfcEntry, RfcRegistry, REGISTRY_PATH as RFC_REGISTRY_PATH};
pub use specification::Specification;
pub use terminology::{
    NormativeDefinition, TerminologyEntry, TerminologyRegistry,
    REGISTRY_PATH as TERMINOLOGY_REGISTRY_PATH,
};
