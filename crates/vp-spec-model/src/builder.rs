//! Assembles typed specification structures from a spec checkout.

use vp_core::{ReadError, SpecRepository};

use crate::corpus::collect_markdown_paths;
use crate::document::parse_document;
use crate::document_corpus::DocumentCorpus;
use crate::error::BuildError;
use crate::reference_graph::{build_reference_graph, ReferenceGraph};
use crate::registry_set::RegistrySet;
use crate::rfc::{self, RfcRegistry};
use crate::specification::Specification;
use crate::terminology::{self, TerminologyRegistry};

/// Builds a [`Specification`] from files under a spec root.
#[derive(Debug, Clone, Copy)]
pub struct SpecificationBuilder<'repo> {
    repo: &'repo SpecRepository,
}

impl<'repo> SpecificationBuilder<'repo> {
    pub fn new(repo: &'repo SpecRepository) -> Self {
        Self { repo }
    }

    /// Build a specification containing VP-TERM and VP-RFC registries only.
    pub fn build_registries_only(&self) -> Result<Specification, BuildError> {
        let terminology = self.load_terminology_registry()?;
        let rfcs = self.load_rfc_registry()?;
        let registry_set = RegistrySet::new(terminology, rfcs);
        Ok(Specification::new(
            self.repo.spec_root().to_path_buf(),
            registry_set,
            DocumentCorpus::empty(),
            ReferenceGraph::empty(),
        ))
    }

    /// Build a specification containing the Markdown document corpus only.
    pub fn build_documents_only(&self) -> Result<Specification, BuildError> {
        let document_corpus = self.load_document_corpus()?;
        let registry_set = RegistrySet::empty();
        let reference_graph = build_reference_graph(&registry_set, &document_corpus);
        Ok(Specification::new(
            self.repo.spec_root().to_path_buf(),
            registry_set,
            document_corpus,
            reference_graph,
        ))
    }

    /// Build registries and the Markdown document corpus.
    pub fn build_registries_and_documents(&self) -> Result<Specification, BuildError> {
        let terminology = self.load_terminology_registry()?;
        let rfcs = self.load_rfc_registry()?;
        let registry_set = RegistrySet::new(terminology, rfcs);
        let document_corpus = self.load_document_corpus()?;
        let reference_graph = build_reference_graph(&registry_set, &document_corpus);
        Ok(Specification::new(
            self.repo.spec_root().to_path_buf(),
            registry_set,
            document_corpus,
            reference_graph,
        ))
    }

    /// Load the VP-TERM registry from the spec checkout.
    pub fn load_terminology_registry(&self) -> Result<TerminologyRegistry, BuildError> {
        let path = terminology::REGISTRY_PATH;
        let yaml = self.read_registry_text(path)?;
        terminology::parse_registry_yaml(path, &yaml)
    }

    /// Load the VP-RFC registry from the spec checkout.
    pub fn load_rfc_registry(&self) -> Result<RfcRegistry, BuildError> {
        let path = rfc::REGISTRY_PATH;
        let yaml = self.read_registry_text(path)?;
        rfc::parse_registry_yaml(path, &yaml)
    }

    fn load_document_corpus(&self) -> Result<DocumentCorpus, BuildError> {
        let mut documents = Vec::new();

        for rel_path in collect_markdown_paths(self.repo) {
            let path = rel_path.to_string_lossy().replace('\\', "/");
            let raw_text = match self.repo.read_text(&rel_path) {
                Ok(text) => text,
                Err(ReadError::NotFound) => {
                    return Err(BuildError::DocumentRead {
                        path: path.clone(),
                        message: "file disappeared during corpus load".into(),
                    });
                }
                Err(ReadError::Io(error)) => {
                    return Err(BuildError::DocumentRead {
                        path: path.clone(),
                        message: error.to_string(),
                    });
                }
                Err(ReadError::YamlParse(_)) => {
                    unreachable!("read_text does not parse YAML")
                }
            };

            documents.push(parse_document(path, raw_text)?);
        }

        Ok(DocumentCorpus::from_documents(documents))
    }

    fn read_registry_text(&self, path: &str) -> Result<String, BuildError> {
        match self.repo.read_text(path) {
            Ok(text) => Ok(text),
            Err(ReadError::NotFound) => Err(BuildError::registry_missing(path)),
            Err(ReadError::Io(error)) => Err(BuildError::RegistryRead {
                path: path.to_string(),
                message: error.to_string(),
            }),
            Err(ReadError::YamlParse(_)) => {
                unreachable!("read_text does not parse YAML")
            }
        }
    }
}
