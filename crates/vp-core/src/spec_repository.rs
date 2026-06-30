//! Abstraction for reading files from a specification checkout.

use std::io;
use std::path::{Path, PathBuf};

use serde_yaml::Value;

/// Errors when reading specification content.
#[derive(Debug)]
pub enum ReadError {
    NotFound,
    Io(io::Error),
    YamlParse(serde_yaml::Error),
}

impl ReadError {
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound)
    }
}

/// Single entry point for specification file access (edition-bundle ready).
#[derive(Debug, Clone)]
pub struct SpecRepository {
    spec_root: PathBuf,
}

impl SpecRepository {
    pub fn new(spec_root: impl Into<PathBuf>) -> Self {
        Self {
            spec_root: spec_root.into(),
        }
    }

    pub fn spec_root(&self) -> &Path {
        &self.spec_root
    }

    pub fn canonical_path(&self, rel_path: impl AsRef<Path>) -> PathBuf {
        self.spec_root.join(rel_path.as_ref())
    }

    pub fn exists(&self, rel_path: impl AsRef<Path>) -> bool {
        self.canonical_path(rel_path).exists()
    }

    pub fn is_file(&self, rel_path: impl AsRef<Path>) -> bool {
        self.canonical_path(rel_path).is_file()
    }

    pub fn read_text(&self, rel_path: impl AsRef<Path>) -> Result<String, ReadError> {
        let path = self.canonical_path(&rel_path);
        if !path.is_file() {
            return Err(ReadError::NotFound);
        }
        std::fs::read_to_string(&path).map_err(ReadError::Io)
    }

    pub fn read_yaml(&self, rel_path: impl AsRef<Path>) -> Result<Value, ReadError> {
        let text = self.read_text(rel_path)?;
        crate::yaml::parse_yaml(&text).map_err(ReadError::YamlParse)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn reads_text_and_yaml_from_spec_root() {
        let dir = tempfile::tempdir().expect("tempdir");
        let registry_dir = dir.path().join("spec/rfcs");
        fs::create_dir_all(&registry_dir).expect("mkdir");
        fs::write(
            registry_dir.join("registry.yaml"),
            "spec: RFC-REGISTRY\ntitle: Test\n",
        )
        .expect("write");

        let repo = SpecRepository::new(dir.path());
        assert!(repo.is_file("spec/rfcs/registry.yaml"));
        assert_eq!(
            repo.read_text("spec/rfcs/registry.yaml")
                .expect("read text")
                .trim(),
            "spec: RFC-REGISTRY\ntitle: Test"
        );

        let yaml = repo
            .read_yaml("spec/rfcs/registry.yaml")
            .expect("read yaml");
        assert_eq!(
            yaml.get("spec").and_then(|v| v.as_str()),
            Some("RFC-REGISTRY")
        );
    }

    #[test]
    fn missing_file_returns_not_found() {
        let dir = tempfile::tempdir().expect("tempdir");
        let repo = SpecRepository::new(dir.path());
        assert!(!repo.is_file("spec/rfcs/registry.yaml"));
        assert!(repo
            .read_text("spec/rfcs/registry.yaml")
            .expect_err("missing")
            .is_not_found());
    }
}
