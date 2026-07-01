//! `.vp.toml` loading and validation (Milestone C.4, ADR-0004).

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::validation_config::{
    resolve_validation_config, ValidationConfig, ValidationConfigOverrides, ValidationOutput,
};

const VP_TOML: &str = ".vp.toml";

/// Errors loading or parsing configuration.
#[derive(Debug, PartialEq, Eq)]
pub enum ConfigError {
    Io(String),
    Parse(String),
    UnknownSection(String),
    UnknownValidationKey(String),
    InvalidOutput(String),
    MissingSpecRoot,
}

impl ConfigError {
    pub fn message(&self) -> String {
        match self {
            Self::Io(message) => message.clone(),
            Self::Parse(message) => format!("invalid `.vp.toml`: {message}"),
            Self::UnknownSection(section) => {
                format!("unknown section `{section}` in `.vp.toml` (expected `[validation]` only)")
            }
            Self::UnknownValidationKey(key) => {
                format!("unknown key `{key}` in `[validation]` in `.vp.toml`")
            }
            Self::InvalidOutput(value) => {
                format!("invalid output `{value}` in `.vp.toml` (expected `human` or `json`)")
            }
            Self::MissingSpecRoot => {
                "missing spec root: pass `--spec` or set `spec_root` in `.vp.toml`".to_string()
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct ValidationTableRaw {
    #[serde(default)]
    spec_root: Option<String>,
    #[serde(default)]
    profile: Option<String>,
    #[serde(default)]
    output: Option<String>,
    #[serde(default)]
    edition: Option<String>,
    #[serde(default)]
    strict: Option<bool>,
}

const KNOWN_VALIDATION_KEYS: &[&str] = &["spec_root", "profile", "output", "edition", "strict"];

/// Load `.vp.toml` from `directory` if present.
pub fn load_vp_toml_from_dir(
    directory: &Path,
) -> Result<Option<ValidationConfigOverrides>, ConfigError> {
    let path = directory.join(VP_TOML);
    if !path.is_file() {
        return Ok(None);
    }

    let text = fs::read_to_string(&path).map_err(|error| ConfigError::Io(error.to_string()))?;
    parse_vp_toml(&text, &path)
}

/// Load `.vp.toml` from the current working directory if present.
pub fn load_vp_toml_from_cwd() -> Result<Option<ValidationConfigOverrides>, ConfigError> {
    let cwd = std::env::current_dir().map_err(|error| ConfigError::Io(error.to_string()))?;
    load_vp_toml_from_dir(&cwd)
}

fn parse_vp_toml(
    text: &str,
    path: &Path,
) -> Result<Option<ValidationConfigOverrides>, ConfigError> {
    let root: toml::Table = toml::from_str(text)
        .map_err(|error| ConfigError::Parse(format!("{}: {error}", path.display())))?;

    if root.is_empty() {
        return Ok(None);
    }

    for key in root.keys() {
        if key != "validation" {
            return Err(ConfigError::UnknownSection(key.clone()));
        }
    }

    let Some(validation_value) = root.get("validation") else {
        return Ok(None);
    };

    let validation_table = validation_value.as_table().ok_or_else(|| {
        ConfigError::Parse(format!(
            "{}: `[validation]` must be a table",
            path.display()
        ))
    })?;

    reject_unknown_validation_keys(validation_table)?;

    let raw: ValidationTableRaw = validation_value
        .clone()
        .try_into()
        .map_err(|error| ConfigError::Parse(format!("{}: {error}", path.display())))?;

    overrides_from_raw(raw)
}

fn reject_unknown_validation_keys(
    table: &toml::map::Map<String, toml::Value>,
) -> Result<(), ConfigError> {
    for key in table.keys() {
        if !KNOWN_VALIDATION_KEYS.contains(&key.as_str()) {
            return Err(ConfigError::UnknownValidationKey(key.clone()));
        }
    }
    Ok(())
}

fn overrides_from_raw(
    raw: ValidationTableRaw,
) -> Result<Option<ValidationConfigOverrides>, ConfigError> {
    if raw.spec_root.is_none()
        && raw.profile.is_none()
        && raw.output.is_none()
        && raw.edition.is_none()
        && raw.strict.is_none()
    {
        return Ok(None);
    }

    let output = if let Some(value) = raw.output {
        Some(ValidationOutput::parse(&value).ok_or(ConfigError::InvalidOutput(value))?)
    } else {
        None
    };

    Ok(Some(ValidationConfigOverrides {
        spec_root: raw.spec_root.map(PathBuf::from),
        profile: raw.profile,
        output,
        edition: raw.edition.map(PathBuf::from),
        strict: raw.strict,
    }))
}

/// Resolve merged configuration from optional file (already loaded) and CLI overrides.
pub fn resolve_config(
    file: Option<&ValidationConfigOverrides>,
    cli: &ValidationConfigOverrides,
) -> ValidationConfig {
    resolve_validation_config(file, cli)
}

/// Resolve spec root path relative to `base` when not absolute.
pub fn resolve_spec_root_path(spec_root: &Path, base: &Path) -> PathBuf {
    if spec_root.is_absolute() {
        spec_root.to_path_buf()
    } else {
        base.join(spec_root)
    }
}

/// Build resolved config and require a spec root.
pub fn resolve_config_with_spec_root(
    file: Option<&ValidationConfigOverrides>,
    cli: &ValidationConfigOverrides,
    base: &Path,
) -> Result<ValidationConfig, ConfigError> {
    let mut config = resolve_config(file, cli);
    let Some(spec_root) = config.spec_root.take() else {
        return Err(ConfigError::MissingSpecRoot);
    };

    config.spec_root = Some(resolve_spec_root_path(&spec_root, base));
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn missing_file_is_not_an_error() {
        let dir = tempfile::tempdir().expect("tempdir");
        assert_eq!(load_vp_toml_from_dir(dir.path()).expect("load"), None);
    }

    #[test]
    fn loads_validation_section() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(
            dir.path().join(VP_TOML),
            r#"
[validation]
spec_root = "../veritypay-spec"
profile = "ci"
output = "json"
edition = "editions/genesis-edition.yaml"
strict = true
"#,
        )
        .expect("write");

        let overrides = load_vp_toml_from_dir(dir.path())
            .expect("load")
            .expect("overrides");
        assert_eq!(
            overrides.spec_root.as_deref(),
            Some(Path::new("../veritypay-spec"))
        );
        assert_eq!(overrides.profile.as_deref(), Some("ci"));
        assert_eq!(overrides.output, Some(ValidationOutput::Json));
        assert_eq!(
            overrides.edition.as_deref(),
            Some(Path::new("editions/genesis-edition.yaml"))
        );
        assert_eq!(overrides.strict, Some(true));
    }

    #[test]
    fn unknown_top_level_section_errors() {
        let err = parse_vp_toml("[profile]\nname = \"ci\"\n", Path::new(".vp.toml")).unwrap_err();
        assert!(matches!(err, ConfigError::UnknownSection(section) if section == "profile"));
    }

    #[test]
    fn unknown_validation_key_errors() {
        let err =
            parse_vp_toml("[validation]\nfoo = \"bar\"\n", Path::new(".vp.toml")).unwrap_err();
        assert!(matches!(err, ConfigError::UnknownValidationKey(key) if key == "foo"));
    }

    #[test]
    fn invalid_toml_errors() {
        let err = parse_vp_toml("not valid toml [[[", Path::new(".vp.toml")).unwrap_err();
        assert!(matches!(err, ConfigError::Parse(_)));
    }

    #[test]
    fn invalid_output_errors() {
        let err =
            parse_vp_toml("[validation]\noutput = \"xml\"\n", Path::new(".vp.toml")).unwrap_err();
        assert!(matches!(err, ConfigError::InvalidOutput(value) if value == "xml"));
    }

    #[test]
    fn cli_overrides_file_and_defaults() {
        let file = ValidationConfigOverrides {
            spec_root: Some(PathBuf::from("from-file")),
            profile: Some("ci".to_string()),
            output: Some(ValidationOutput::Json),
            edition: None,
            strict: None,
        };
        let cli = ValidationConfigOverrides {
            spec_root: Some(PathBuf::from("from-cli")),
            profile: None,
            output: Some(ValidationOutput::Human),
            edition: None,
            strict: Some(true),
        };

        let config = resolve_config(Some(&file), &cli);
        assert_eq!(config.spec_root.as_deref(), Some(Path::new("from-cli")));
        assert_eq!(config.profile.as_deref(), Some("ci"));
        assert_eq!(config.output, ValidationOutput::Human);
        assert!(config.strict);
    }

    #[test]
    fn missing_spec_root_after_merge_errors() {
        let err = resolve_config_with_spec_root(
            None,
            &ValidationConfigOverrides::default(),
            Path::new("."),
        )
        .unwrap_err();
        assert_eq!(err, ConfigError::MissingSpecRoot);
    }

    #[test]
    fn validator_crates_do_not_parse_vp_toml() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let sources = [
            manifest_dir.join("../vp-registry/src/rfc_registry.rs"),
            manifest_dir.join("../vp-registry/src/term_registry.rs"),
            manifest_dir.join("../vp-crossref/src/validate.rs"),
            manifest_dir.join("../vp-registry/src/registry_validator.rs"),
            manifest_dir.join("../vp-registry/src/term_registry_validator.rs"),
            manifest_dir.join("../vp-crossref/src/validator.rs"),
        ];

        for path in sources {
            let text = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
            assert!(
                !text.contains(".vp.toml"),
                "{} must not parse `.vp.toml`",
                path.display()
            );
            assert!(
                !text.contains("toml::"),
                "{} must not use the toml crate",
                path.display()
            );
        }

        let edition_sources = [
            manifest_dir.join("../vp-edition/src/edition.rs"),
            manifest_dir.join("../vp-edition/src/validator.rs"),
        ];
        for path in edition_sources {
            let text = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
            assert!(
                !text.contains(".vp.toml"),
                "{} must not parse `.vp.toml`",
                path.display()
            );
            assert!(
                !text.contains("toml::"),
                "{} must not use the toml crate",
                path.display()
            );
        }
    }
}
