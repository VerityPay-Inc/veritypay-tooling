//! Specification model loading for cross-reference validation.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use vp_core::SpecRepository;
use vp_spec_model::{DocumentCorpus, RegistrySet, SpecificationBuilder, SpecificationDocument};

use crate::corpus::collect_markdown_files;
use crate::registry_lookup::RegistryLookup;
use crate::resolve::extract_document_anchors;

/// Typed specification data used during cross-reference validation.
#[derive(Debug)]
pub struct CrossrefModel {
    pub registry_set: Option<RegistrySet>,
    pub document_corpus: Option<DocumentCorpus>,
    raw_registry: RegistryLookup,
}

impl CrossrefModel {
    pub fn load(repo: &SpecRepository) -> Self {
        if let Ok(specification) = SpecificationBuilder::new(repo).build_registries_and_documents()
        {
            return Self {
                registry_set: Some(specification.registry_set),
                document_corpus: Some(specification.document_corpus),
                raw_registry: RegistryLookup::default(),
            };
        }

        let registry_set = SpecificationBuilder::new(repo)
            .build_registries_only()
            .ok()
            .map(|specification| specification.registry_set);
        let document_corpus = SpecificationBuilder::new(repo)
            .build_documents_only()
            .ok()
            .map(|specification| specification.document_corpus);

        Self {
            registry_set,
            document_corpus,
            raw_registry: RegistryLookup::load(repo),
        }
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

    pub fn scan_documents(
        &self,
        repo: &SpecRepository,
    ) -> Vec<(PathBuf, String)> {
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
