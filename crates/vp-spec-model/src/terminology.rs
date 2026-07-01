//! VP-TERM registry types.

use std::collections::HashMap;

use serde::Deserialize;

pub const REGISTRY_PATH: &str = "spec/terminology/registry.yaml";

/// Typed VP-TERM registry loaded from `spec/terminology/registry.yaml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminologyRegistry {
    pub source_path: String,
    entries: Vec<TerminologyEntry>,
    by_id: HashMap<String, usize>,
}

impl TerminologyRegistry {
    pub(crate) fn from_entries(source_path: impl Into<String>, entries: Vec<TerminologyEntry>) -> Self {
        let source_path = source_path.into();
        let by_id = entries
            .iter()
            .enumerate()
            .map(|(index, entry)| (entry.id.clone(), index))
            .collect();
        Self {
            source_path,
            entries,
            by_id,
        }
    }

    pub fn entries(&self) -> &[TerminologyEntry] {
        &self.entries
    }

    pub fn get(&self, id: &str) -> Option<&TerminologyEntry> {
        self.by_id.get(id).map(|index| &self.entries[*index])
    }
}

/// One VP-TERM registry entry.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TerminologyEntry {
    pub id: String,
    #[serde(default)]
    pub anchor: Option<String>,
    pub title: String,
    pub stability: String,
    pub classification: String,
    #[serde(default)]
    pub normative_definition: Option<NormativeDefinition>,
    #[serde(default)]
    pub referenced_by: Vec<String>,
    #[serde(default)]
    pub depends_on: Vec<String>,
}

/// Normative definition pointer for a terminology entry.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct NormativeDefinition {
    #[serde(default)]
    pub document: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    pub section_id: String,
}

#[derive(Debug, Deserialize)]
struct TerminologyRegistryDocument {
    terms: Vec<TerminologyEntry>,
}

pub(crate) fn parse_registry_yaml(
    source_path: &str,
    yaml: &str,
) -> Result<TerminologyRegistry, super::BuildError> {
    let document: TerminologyRegistryDocument = serde_yaml::from_str(yaml).map_err(|error| {
        super::BuildError::yaml_invalid(source_path, error.to_string())
    })?;
    Ok(TerminologyRegistry::from_entries(
        source_path,
        document.terms,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_by_id() {
        let registry = TerminologyRegistry::from_entries(
            REGISTRY_PATH,
            vec![TerminologyEntry {
                id: "VP-TERM-001".to_string(),
                anchor: None,
                title: "Protocol".to_string(),
                stability: "stable".to_string(),
                classification: "fundamental".to_string(),
                normative_definition: Some(NormativeDefinition {
                    document: Some("DOMAIN_MODEL".to_string()),
                    path: None,
                    section_id: "DM-1.1".to_string(),
                }),
                referenced_by: vec!["MANIFESTO".to_string()],
                depends_on: vec![],
            }],
        );

        assert_eq!(registry.get("VP-TERM-001").map(|e| e.title.as_str()), Some("Protocol"));
        assert!(registry.get("VP-TERM-999").is_none());
    }
}
