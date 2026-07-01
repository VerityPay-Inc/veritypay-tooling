//! Specification model loading for cross-reference validation.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use vp_core::SpecRepository;
use vp_spec_model::{
    DocumentCorpus, ReferenceGraph, RegistrySet, SpecificationBuilder, SpecificationDocument,
};

use crate::corpus::collect_markdown_files;
use crate::registry_lookup::RegistryLookup;
use crate::resolve::extract_document_anchors;

/// Typed specification data used during cross-reference validation.
#[derive(Debug)]
pub struct CrossrefModel {
    pub registry_set: Option<RegistrySet>,
    pub document_corpus: Option<DocumentCorpus>,
    reference_graph: Option<ReferenceGraph>,
    raw_registry: RegistryLookup,
}

impl CrossrefModel {
    pub fn load(repo: &SpecRepository) -> Self {
        if let Ok(specification) = SpecificationBuilder::new(repo).build_registries_and_documents()
        {
            let reference_graph = specification.reference_graph().clone();
            return Self {
                registry_set: Some(specification.registry_set),
                document_corpus: Some(specification.document_corpus),
                reference_graph: Some(reference_graph),
                raw_registry: RegistryLookup::default(),
            };
        }

        let registry_set = SpecificationBuilder::new(repo)
            .build_registries_only()
            .ok()
            .map(|specification| specification.registry_set);
        let documents_spec = SpecificationBuilder::new(repo).build_documents_only().ok();
        let document_corpus = documents_spec
            .as_ref()
            .map(|specification| specification.document_corpus.clone());
        let reference_graph =
            documents_spec.map(|specification| specification.reference_graph().clone());

        Self {
            registry_set,
            document_corpus,
            reference_graph,
            raw_registry: RegistryLookup::load(repo),
        }
    }

    pub fn reference_graph(&self) -> Option<&ReferenceGraph> {
        self.reference_graph.as_ref()
    }

    pub fn uses_reference_graph(&self) -> bool {
        self.reference_graph.is_some()
    }

    pub fn term_is_known(&self, term_id: &str) -> bool {
        if let Some(registry_set) = &self.registry_set {
            registry_set.terminology.get(term_id).is_some()
        } else {
            self.raw_registry.term_ids.contains(term_id)
        }
    }

    pub fn rfc_is_known(&self, rfc_id: &str) -> bool {
        if let Some(registry_set) = &self.registry_set {
            registry_set.rfcs.get(rfc_id).is_some()
        } else {
            self.raw_registry.rfc_ids.contains(rfc_id)
        }
    }

    pub fn scan_documents(&self, repo: &SpecRepository) -> Vec<(PathBuf, String)> {
        if let Some(corpus) = &self.document_corpus {
            return corpus
                .documents()
                .iter()
                .map(|document| {
                    (
                        PathBuf::from(&document.relative_path),
                        document.raw_text.clone(),
                    )
                })
                .collect();
        }

        collect_markdown_files(repo)
            .into_iter()
            .filter_map(|rel_path| {
                repo.read_text(&rel_path)
                    .ok()
                    .map(|content| (rel_path, content))
            })
            .collect()
    }

    pub fn document_content(&self, repo: &SpecRepository, path: &Path) -> Option<String> {
        if let Some(content) = self
            .document_corpus
            .as_ref()
            .and_then(|corpus| corpus.get(path))
            .map(|document| document.raw_text.clone())
        {
            return Some(content);
        }

        repo.read_text(path).ok()
    }

    pub fn link_target_exists(&self, repo: &SpecRepository, resolved: &Path) -> bool {
        if resolved.as_os_str().is_empty() {
            return true;
        }

        if self
            .document_corpus
            .as_ref()
            .and_then(|corpus| corpus.get(resolved))
            .is_some()
        {
            return true;
        }

        let canonical = repo.canonical_path(resolved);
        canonical.is_file() || canonical.is_dir()
    }

    pub fn document_anchors(
        &self,
        repo: &SpecRepository,
        resolved: &Path,
        source_content_path: &Path,
        source_content: &str,
    ) -> HashSet<String> {
        if let Some(document) = self
            .document_corpus
            .as_ref()
            .and_then(|corpus| corpus.get(resolved))
        {
            return section_anchors(document);
        }

        if resolved == source_content_path {
            extract_document_anchors(source_content)
        } else if let Ok(content) = repo.read_text(resolved) {
            extract_document_anchors(&content)
        } else {
            HashSet::new()
        }
    }
}

fn section_anchors(document: &SpecificationDocument) -> HashSet<String> {
    document
        .sections
        .iter()
        .map(|section| section.anchor.clone())
        .collect()
}
