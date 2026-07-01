//! Registry lookup tables for cross-reference resolution.

use std::collections::HashSet;

use serde_yaml::Value;
use vp_core::{ReadError, SpecRepository};

use crate::constants::{RFC_REGISTRY_PATH, TERM_REGISTRY_PATH};

#[derive(Debug, Default)]
pub struct RegistryLookup {
    pub term_ids: HashSet<String>,
    pub rfc_ids: HashSet<String>,
}

impl RegistryLookup {
    pub fn load(repo: &SpecRepository) -> Self {
        Self {
            term_ids: load_ids(repo, TERM_REGISTRY_PATH, "terms", "id"),
            rfc_ids: load_ids(repo, RFC_REGISTRY_PATH, "rfcs", "id"),
        }
    }
}

fn load_ids(
    repo: &SpecRepository,
    registry_path: &str,
    collection_key: &str,
    id_field: &str,
) -> HashSet<String> {
    let yaml = match repo.read_yaml(registry_path) {
        Ok(value) => value,
        Err(ReadError::NotFound) | Err(ReadError::Io(_)) | Err(ReadError::YamlParse(_)) => {
            return HashSet::new();
        }
    };

    let Some(mapping) = yaml.as_mapping() else {
        return HashSet::new();
    };

    let Some(entries) = mapping
        .get(Value::from(collection_key))
        .and_then(Value::as_sequence)
    else {
        return HashSet::new();
    };

    entries
        .iter()
        .filter_map(|entry| {
            entry
                .as_mapping()?
                .get(Value::from(id_field))?
                .as_str()
                .map(str::to_owned)
        })
        .collect()
}
