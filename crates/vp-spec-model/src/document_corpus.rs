//! Document corpus aggregate.

use std::collections::HashMap;
use std::path::Path;

use crate::document::SpecificationDocument;

/// Collection of specification documents discovered under the spec root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentCorpus {
    documents: Vec<SpecificationDocument>,
    by_path: HashMap<String, usize>,
}

impl DocumentCorpus {
    pub(crate) fn empty() -> Self {
        Self {
            documents: Vec::new(),
            by_path: HashMap::new(),
        }
    }

    pub(crate) fn from_documents(documents: Vec<SpecificationDocument>) -> Self {
        let by_path = documents
            .iter()
            .enumerate()
            .map(|(index, document)| (document.relative_path.clone(), index))
            .collect();
        Self { documents, by_path }
    }

    pub fn documents(&self) -> &[SpecificationDocument] {
        &self.documents
    }

    pub fn get(&self, relative_path: impl AsRef<Path>) -> Option<&SpecificationDocument> {
        let key = path_key(relative_path.as_ref());
        self.by_path
            .get(&key)
            .map(|index| &self.documents[*index])
    }
}

fn path_key(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
