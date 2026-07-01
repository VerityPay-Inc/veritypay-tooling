//! Resolved validation configuration (ADR-0004).

use std::path::PathBuf;

/// Output format for a validation session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ValidationOutput {
    #[default]
    Human,
    Json,
}

impl ValidationOutput {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "human" => Some(Self::Human),
            "json" => Some(Self::Json),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Human => "human",
            Self::Json => "json",
        }
    }
}

/// Immutable value object: resolved inputs for one validation execution.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ValidationConfig {
    pub spec_root: Option<PathBuf>,
    pub profile: Option<String>,
    pub output: ValidationOutput,
    pub edition: Option<PathBuf>,
    pub strict: bool,
}

impl ValidationConfig {
    /// Built-in defaults when no file or CLI values are supplied.
    pub fn builtins() -> Self {
        Self::default()
    }

    pub fn with_spec_root(mut self, spec_root: impl Into<PathBuf>) -> Self {
        self.spec_root = Some(spec_root.into());
        self
    }

    pub fn with_profile(mut self, profile: impl Into<String>) -> Self {
        self.profile = Some(profile.into());
        self
    }

    pub fn with_output(mut self, output: ValidationOutput) -> Self {
        self.output = output;
        self
    }

    pub fn with_edition(mut self, edition: impl Into<PathBuf>) -> Self {
        self.edition = Some(edition.into());
        self
    }

    pub fn with_strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }
}

/// Partial overrides applied during config merge (file or CLI layer).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValidationConfigOverrides {
    pub spec_root: Option<PathBuf>,
    pub profile: Option<String>,
    pub output: Option<ValidationOutput>,
    pub edition: Option<PathBuf>,
    pub strict: Option<bool>,
}

impl ValidationConfigOverrides {
    pub fn spec_root(spec_root: impl Into<PathBuf>) -> Self {
        Self {
            spec_root: Some(spec_root.into()),
            ..Self::default()
        }
    }
}

/// Merge `overrides` onto `base` when override fields are present.
pub fn apply_overrides(base: &mut ValidationConfig, overrides: &ValidationConfigOverrides) {
    if let Some(spec_root) = &overrides.spec_root {
        base.spec_root = Some(spec_root.clone());
    }
    if let Some(profile) = &overrides.profile {
        base.profile = Some(profile.clone());
    }
    if let Some(output) = overrides.output {
        base.output = output;
    }
    if let Some(edition) = &overrides.edition {
        base.edition = Some(edition.clone());
    }
    if let Some(strict) = overrides.strict {
        base.strict = strict;
    }
}

/// Resolve configuration: defaults → file → CLI.
pub fn resolve_validation_config(
    file: Option<&ValidationConfigOverrides>,
    cli: &ValidationConfigOverrides,
) -> ValidationConfig {
    let mut config = ValidationConfig::builtins();
    if let Some(file_overrides) = file {
        apply_overrides(&mut config, file_overrides);
    }
    apply_overrides(&mut config, cli);
    config
}
