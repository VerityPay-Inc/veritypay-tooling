//! Errors building a [`Specification`] from a spec checkout.

use std::fmt;

/// Failure loading or parsing specification artifacts into the model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildError {
    RegistryMissing {
        path: String,
    },
    RegistryRead {
        path: String,
        message: String,
    },
    YamlInvalid {
        path: String,
        message: String,
    },
    DocumentRead {
        path: String,
        message: String,
    },
}

impl BuildError {
    pub fn registry_missing(path: impl Into<String>) -> Self {
        Self::RegistryMissing {
            path: path.into(),
        }
    }

    pub fn yaml_invalid(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self::YamlInvalid {
            path: path.into(),
            message: message.into(),
        }
    }

    pub fn path(&self) -> &str {
        match self {
            Self::RegistryMissing { path }
            | Self::RegistryRead { path, .. }
            | Self::YamlInvalid { path, .. }
            | Self::DocumentRead { path, .. } => path,
        }
    }
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RegistryMissing { path } => {
                write!(f, "registry file is missing at `{path}`")
            }
            Self::RegistryRead { path, message } => {
                write!(f, "registry file `{path}` could not be read: {message}")
            }
            Self::YamlInvalid { path, message } => {
                write!(f, "registry YAML at `{path}` is invalid: {message}")
            }
            Self::DocumentRead { path, message } => {
                write!(f, "document at `{path}` could not be read: {message}")
            }
        }
    }
}

impl std::error::Error for BuildError {}
