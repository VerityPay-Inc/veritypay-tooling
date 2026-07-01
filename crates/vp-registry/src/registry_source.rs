//! Shared registry file access for validators.

use vp_core::{parse_yaml, ReadError, SpecRepository};
use vp_spec_model::SpecificationBuilder;

/// Raw registry YAML text, when the file is present and readable.
pub fn read_registry_text(
    repo: &SpecRepository,
    path: &str,
) -> Result<String, RegistryReadOutcome> {
    match repo.read_text(path) {
        Ok(text) => Ok(text),
        Err(ReadError::NotFound) => Err(RegistryReadOutcome::Missing),
        Err(ReadError::Io(error)) => Err(RegistryReadOutcome::Io(error.to_string())),
        Err(ReadError::YamlParse(_)) => unreachable!("read_text does not parse YAML"),
    }
}

/// Parse registry YAML syntax into a generic value for structural validation.
pub fn parse_registry_root(contents: &str) -> Result<serde_yaml::Value, String> {
    parse_yaml(contents).map_err(|error| error.to_string())
}

/// Load a typed RFC registry when the file is structurally deserializable.
pub fn try_load_rfc_registry(
    repo: &SpecRepository,
) -> Option<vp_spec_model::RfcRegistry> {
    SpecificationBuilder::new(repo)
        .load_rfc_registry()
        .ok()
}

/// Load a typed terminology registry when the file is structurally deserializable.
pub fn try_load_terminology_registry(
    repo: &SpecRepository,
) -> Option<vp_spec_model::TerminologyRegistry> {
    SpecificationBuilder::new(repo)
        .load_terminology_registry()
        .ok()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryReadOutcome {
    Missing,
    Io(String),
}
