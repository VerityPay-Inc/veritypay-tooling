//! Registry loading for Edition validation via `vp-spec-model`.

use std::collections::HashSet;

use serde_yaml::Value;
use vp_core::{ReadError, SpecRepository};
use vp_spec_model::{RegistrySet, SpecificationBuilder, RFC_REGISTRY_PATH};

pub fn try_load_registry_set(repo: &SpecRepository) -> Option<RegistrySet> {
    SpecificationBuilder::new(repo)
        .build_registries_only()
        .ok()
        .map(|specification| specification.registry_set)
}

pub fn accepted_rfc_is_known(
    registry_set: Option<&RegistrySet>,
    raw_rfc_ids: &HashSet<String>,
    rfc_id: &str,
) -> bool {
    if let Some(set) = registry_set {
        set.rfcs.get(rfc_id).is_some()
    } else {
        raw_rfc_ids.contains(rfc_id)
    }
}

pub fn registry_snapshot_exists(
    repo: &SpecRepository,
    registry_set: Option<&RegistrySet>,
    path: &str,
) -> bool {
    if let Some(set) = registry_set {
        if path == set.rfcs.source_path.as_str() || path == set.terminology.source_path.as_str() {
            return true;
        }
    }
    repo.is_file(path)
}

/// Fallback RFC id lookup when typed registry loading fails.
pub fn load_rfc_ids_raw(repo: &SpecRepository) -> HashSet<String> {
    let yaml = match repo.read_yaml(RFC_REGISTRY_PATH) {
        Ok(value) => value,
        Err(ReadError::NotFound) | Err(ReadError::Io(_)) | Err(ReadError::YamlParse(_)) => {
            return HashSet::new();
        }
    };

    let Some(mapping) = yaml.as_mapping() else {
        return HashSet::new();
    };

    let Some(entries) = mapping
        .get(Value::from("rfcs"))
        .and_then(Value::as_sequence)
    else {
        return HashSet::new();
    };

    entries
        .iter()
        .filter_map(|entry| {
            entry
                .as_mapping()?
                .get(Value::from("id"))?
                .as_str()
                .map(str::to_owned)
        })
        .collect()
}
