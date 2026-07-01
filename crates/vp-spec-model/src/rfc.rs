//! VP-RFC registry types.

use std::collections::HashMap;

use serde::Deserialize;

pub const REGISTRY_PATH: &str = "spec/rfcs/registry.yaml";

/// Typed VP-RFC registry loaded from `spec/rfcs/registry.yaml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RfcRegistry {
    pub source_path: String,
    entries: Vec<RfcEntry>,
    by_id: HashMap<String, usize>,
}

impl RfcRegistry {
    pub(crate) fn empty() -> Self {
        Self::from_entries(REGISTRY_PATH, Vec::new())
    }

    pub(crate) fn from_entries(source_path: impl Into<String>, entries: Vec<RfcEntry>) -> Self {
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

    pub fn entries(&self) -> &[RfcEntry] {
        &self.entries
    }

    pub fn get(&self, id: &str) -> Option<&RfcEntry> {
        self.by_id.get(id).map(|index| &self.entries[*index])
    }
}

/// One VP-RFC registry entry.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RfcEntry {
    pub id: String,
    pub rfc: String,
    #[serde(default)]
    pub anchor: Option<String>,
    pub title: String,
    pub status: String,
    #[serde(rename = "type")]
    pub rfc_type: String,
    pub version: String,
    pub created: String,
    pub updated: String,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub supersedes: Vec<String>,
    #[serde(default)]
    pub superseded_by: Option<String>,
    #[serde(default)]
    pub related_terms: Vec<String>,
    #[serde(default)]
    pub related_architecture: Vec<String>,
    #[serde(default)]
    pub related_conformance: Vec<String>,
    pub path: String,
}

#[derive(Debug, Deserialize)]
struct RfcRegistryDocument {
    rfcs: Vec<RfcEntry>,
}

pub(crate) fn parse_registry_yaml(
    source_path: &str,
    yaml: &str,
) -> Result<RfcRegistry, super::BuildError> {
    let document: RfcRegistryDocument = serde_yaml::from_str(yaml).map_err(|error| {
        super::BuildError::yaml_invalid(source_path, error.to_string())
    })?;
    Ok(RfcRegistry::from_entries(source_path, document.rfcs))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_by_id() {
        let registry = RfcRegistry::from_entries(
            REGISTRY_PATH,
            vec![RfcEntry {
                id: "VP-RFC-0000".to_string(),
                rfc: "0000".to_string(),
                anchor: None,
                title: "RFC Process".to_string(),
                status: "accepted".to_string(),
                rfc_type: "meta".to_string(),
                version: "1.0.0".to_string(),
                created: "2026-06-29".to_string(),
                updated: "2026-06-29".to_string(),
                depends_on: vec![],
                supersedes: vec![],
                superseded_by: None,
                related_terms: vec![],
                related_architecture: vec![],
                related_conformance: vec![],
                path: "rfcs/0000-rfc-process.md".to_string(),
            }],
        );

        assert_eq!(
            registry.get("VP-RFC-0000").map(|e| e.title.as_str()),
            Some("RFC Process")
        );
        assert!(registry.get("VP-RFC-9999").is_none());
    }
}
