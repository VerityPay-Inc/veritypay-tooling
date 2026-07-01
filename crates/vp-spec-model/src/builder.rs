//! Assembles typed specification structures from a spec checkout.

use vp_core::{ReadError, SpecRepository};

use crate::error::BuildError;
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
        ))
    }

    fn load_terminology_registry(&self) -> Result<TerminologyRegistry, BuildError> {
        let path = terminology::REGISTRY_PATH;
        let yaml = self.read_registry_text(path)?;
        terminology::parse_registry_yaml(path, &yaml)
    }

    fn load_rfc_registry(&self) -> Result<RfcRegistry, BuildError> {
        let path = rfc::REGISTRY_PATH;
        let yaml = self.read_registry_text(path)?;
        rfc::parse_registry_yaml(path, &yaml)
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
