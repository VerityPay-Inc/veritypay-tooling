//! Immutable validation run configuration (ADR-0003, ADR-0004).

use std::path::{Path, PathBuf};

use crate::spec_repository::SpecRepository;
use crate::spec_root::canonicalize_spec_root;
use crate::validation_config::ValidationConfig;

/// Read-only inputs shared with every validator in a run.
#[derive(Debug, Clone)]
pub struct ValidationContext {
    repository: SpecRepository,
    config: ValidationConfig,
}

impl ValidationContext {
    /// Build context from a resolved configuration (canonicalizes spec root).
    pub fn from_config(config: ValidationConfig) -> Result<Self, MissingSpecRootError> {
        let spec_root = config
            .spec_root
            .as_ref()
            .ok_or(MissingSpecRootError)?
            .clone();
        let spec_root = canonicalize_spec_root(spec_root);
        let config = config.with_spec_root(spec_root.clone());
        Ok(Self {
            repository: SpecRepository::new(&spec_root),
            config,
        })
    }

    /// Convenience constructor with spec root and built-in defaults.
    pub fn new(spec_root: impl Into<PathBuf>) -> Self {
        Self::from_config(ValidationConfig::default().with_spec_root(spec_root))
            .expect("spec root provided")
    }

    /// Resolved validation configuration for this run.
    pub fn config(&self) -> &ValidationConfig {
        &self.config
    }

    /// Specification checkout root.
    pub fn spec_root(&self) -> &Path {
        self.repository.spec_root()
    }

    /// Whether strict mode is enabled for this run.
    pub fn strict(&self) -> bool {
        self.config.strict
    }

    /// Access specification files under the configured spec root.
    pub fn repository(&self) -> &SpecRepository {
        &self.repository
    }
}

/// Spec root was not present on resolved configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MissingSpecRootError;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation_config::ValidationOutput;

    #[test]
    fn exposes_validation_config() {
        let dir = tempfile::tempdir().expect("tempdir");
        let config = ValidationConfig::default()
            .with_spec_root(dir.path())
            .with_profile("ci")
            .with_output(ValidationOutput::Json)
            .with_strict(true);

        let ctx = ValidationContext::from_config(config.clone()).expect("context");
        assert_eq!(ctx.config().profile, config.profile);
        assert_eq!(ctx.config().output, config.output);
        assert_eq!(ctx.config().edition, config.edition);
        assert_eq!(ctx.config().strict, config.strict);
        assert_eq!(
            ctx.config().spec_root.as_deref(),
            Some(ctx.spec_root())
        );
        assert!(ctx.strict());
    }
}
