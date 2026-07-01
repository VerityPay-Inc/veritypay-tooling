//! Edition Manifest validation rules.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use regex::Regex;
use serde_yaml::Value;
use vp_core::{parse_yaml, ReadError, ValidationContext};
use vp_diagnostics::{Category, Diagnostic, Location, RuleId, RuleKind};
use vp_spec_model::RFC_REGISTRY_PATH;

use crate::registry_source::{
    accepted_rfc_is_known, load_rfc_ids_raw, registry_snapshot_exists, try_load_registry_set,
};

const REQUIRED_TOP_LEVEL: &[&str] = &[
    "edition",
    "edition_id",
    "protocol_version",
    "publication_date",
    "status",
    "specification_documents",
    "accepted_rfcs",
    "registry_snapshots",
    "conformance_baseline",
];

const ALLOWED_STATUS: &[&str] = &[
    "draft",
    "candidate",
    "published",
    "maintained",
    "superseded",
    "archived",
];

pub fn validate(ctx: &ValidationContext) -> Vec<Diagnostic> {
    let Some(manifest_rel) = ctx.config().edition.as_ref() else {
        return Vec::new();
    };

    let repo = ctx.repository();
    let manifest_path = manifest_rel.as_path();

    if !repo.is_file(manifest_path) {
        return vec![edition_diagnostic(
            RuleKind::ManifestMissing,
            format!(
                "Edition Manifest file is missing at `{}`",
                manifest_path.display()
            ),
            Some(format!(
                "create `{}` or update the configured edition path",
                manifest_path.display()
            )),
            Some(manifest_path.to_path_buf()),
            None,
        )];
    }

    let contents = match repo.read_text(manifest_path) {
        Ok(text) => text,
        Err(ReadError::NotFound) => {
            return vec![edition_diagnostic(
                RuleKind::ManifestMissing,
                format!(
                    "Edition Manifest file is missing at `{}`",
                    manifest_path.display()
                ),
                Some(format!(
                    "create `{}` or update the configured edition path",
                    manifest_path.display()
                )),
                Some(manifest_path.to_path_buf()),
                None,
            )];
        }
        Err(ReadError::Io(err)) => {
            return vec![edition_diagnostic(
                RuleKind::ManifestMissing,
                format!("Edition Manifest file could not be read: {err}"),
                Some(format!("ensure `{}` is readable", manifest_path.display())),
                Some(manifest_path.to_path_buf()),
                None,
            )];
        }
        Err(ReadError::YamlParse(_)) => unreachable!("read_text does not parse YAML"),
    };

    let root: Value = match parse_yaml(&contents) {
        Ok(value) => value,
        Err(err) => {
            return vec![edition_diagnostic(
                RuleKind::ManifestYamlInvalid,
                format!("Edition Manifest YAML is invalid: {err}"),
                Some("fix YAML syntax in the Edition Manifest".into()),
                Some(manifest_path.to_path_buf()),
                None,
            )];
        }
    };

    let Some(mapping) = root.as_mapping() else {
        return vec![edition_diagnostic(
            RuleKind::ManifestYamlInvalid,
            "Edition Manifest root must be a YAML mapping",
            Some("wrap manifest fields in a mapping at the document root".into()),
            Some(manifest_path.to_path_buf()),
            None,
        )];
    };

    let mut diagnostics = Vec::new();

    for field in REQUIRED_TOP_LEVEL {
        if !mapping.contains_key(Value::from(*field)) {
            diagnostics.push(edition_diagnostic(
                RuleKind::MissingField,
                format!("required top-level field `{field}` is missing"),
                Some(format!("add `{field}` to the Edition Manifest")),
                Some(manifest_path.to_path_buf()),
                Some(Location::yaml_path(*field)),
            ));
        }
    }

    if let Some(edition_id) = mapping
        .get(Value::from("edition_id"))
        .and_then(Value::as_str)
    {
        if !is_valid_edition_id(edition_id) {
            diagnostics.push(edition_diagnostic(
                RuleKind::InvalidEditionId,
                format!("edition_id `{edition_id}` does not match the vp-edition-* pattern"),
                Some("use a stable identifier such as `vp-edition-genesis-1`".into()),
                Some(manifest_path.to_path_buf()),
                Some(Location::yaml_path("edition_id")),
            ));
        }
    }

    if let Some(status) = mapping.get(Value::from("status")).and_then(Value::as_str) {
        if !ALLOWED_STATUS.contains(&status) {
            diagnostics.push(edition_diagnostic(
                RuleKind::InvalidEditionStatus,
                format!("status `{status}` is not a recognized publication lifecycle value"),
                Some(format!("use one of: {}", ALLOWED_STATUS.join(", "))),
                Some(manifest_path.to_path_buf()),
                Some(Location::yaml_path("status")),
            ));
        }
    }

    if let Some(docs) = mapping.get(Value::from("specification_documents")) {
        diagnostics.extend(validate_specification_documents(repo, manifest_path, docs));
    }

    let registry_set = try_load_registry_set(repo);
    let raw_rfc_ids = if registry_set.is_none() {
        load_rfc_ids_raw(repo)
    } else {
        HashSet::new()
    };

    if let Some(rfcs) = mapping.get(Value::from("accepted_rfcs")) {
        diagnostics.extend(validate_accepted_rfcs(
            manifest_path,
            rfcs,
            registry_set.as_ref(),
            &raw_rfc_ids,
        ));
    }

    if let Some(snapshots) = mapping.get(Value::from("registry_snapshots")) {
        diagnostics.extend(validate_registry_snapshots(
            repo,
            manifest_path,
            snapshots,
            registry_set.as_ref(),
        ));
    }

    if let Some(baseline) = mapping.get(Value::from("conformance_baseline")) {
        diagnostics.extend(validate_conformance_baseline(manifest_path, baseline));
    }

    diagnostics
}

fn validate_specification_documents(
    repo: &vp_core::SpecRepository,
    manifest_path: &Path,
    value: &Value,
) -> Vec<Diagnostic> {
    let Some(mapping) = value.as_mapping() else {
        return vec![edition_diagnostic(
            RuleKind::ManifestYamlInvalid,
            "specification_documents must be a mapping of path to version",
            Some("use a YAML mapping such as `docs/foo.md: \"0.1.0\"`".into()),
            Some(manifest_path.to_path_buf()),
            Some(Location::yaml_path("specification_documents")),
        )];
    };

    let mut diagnostics = Vec::new();

    for (path_value, version_value) in mapping {
        let Some(doc_path) = path_value.as_str() else {
            continue;
        };

        let yaml_path = format!("specification_documents.{doc_path}");

        if !repo.is_file(doc_path) {
            diagnostics.push(edition_diagnostic(
                RuleKind::DocumentMissing,
                format!("pinned document `{doc_path}` does not exist under the spec root"),
                Some(format!(
                    "add `{doc_path}` or remove it from specification_documents"
                )),
                Some(manifest_path.to_path_buf()),
                Some(Location::yaml_path(&yaml_path)),
            ));
            continue;
        }

        let Some(pinned_version) = version_value.as_str() else {
            continue;
        };

        let Ok(content) = repo.read_text(doc_path) else {
            continue;
        };

        if let Some(front_matter_version) = parse_front_matter_version(&content) {
            if front_matter_version != pinned_version {
                diagnostics.push(edition_diagnostic(
                    RuleKind::PinnedVersionMismatch,
                    format!(
                        "pinned version `{pinned_version}` for `{doc_path}` does not match front matter version `{front_matter_version}`"
                    ),
                    Some(format!(
                        "align specification_documents[`{doc_path}`] with the document front matter version"
                    )),
                    Some(manifest_path.to_path_buf()),
                    Some(Location::yaml_path(&yaml_path)),
                ));
            }
        }
    }

    diagnostics
}

fn validate_accepted_rfcs(
    manifest_path: &Path,
    value: &Value,
    registry_set: Option<&vp_spec_model::RegistrySet>,
    raw_rfc_ids: &HashSet<String>,
) -> Vec<Diagnostic> {
    let Some(sequence) = value.as_sequence() else {
        return vec![edition_diagnostic(
            RuleKind::ManifestYamlInvalid,
            "accepted_rfcs must be a sequence of VP-RFC ids",
            Some("use a YAML list such as `- VP-RFC-0000`".into()),
            Some(manifest_path.to_path_buf()),
            Some(Location::yaml_path("accepted_rfcs")),
        )];
    };

    let mut diagnostics = Vec::new();

    for (index, entry) in sequence.iter().enumerate() {
        let Some(rfc_id) = entry.as_str() else {
            continue;
        };

        if !accepted_rfc_is_known(registry_set, raw_rfc_ids, rfc_id) {
            diagnostics.push(edition_diagnostic(
                RuleKind::UnknownAcceptedRfc,
                format!("accepted RFC `{rfc_id}` is not listed in {RFC_REGISTRY_PATH}"),
                Some(format!(
                    "add `{rfc_id}` to the RFC registry or remove it from accepted_rfcs"
                )),
                Some(manifest_path.to_path_buf()),
                Some(Location::yaml_path(format!("accepted_rfcs[{index}]"))),
            ));
        }
    }

    diagnostics
}

fn validate_registry_snapshots(
    repo: &vp_core::SpecRepository,
    manifest_path: &Path,
    value: &Value,
    registry_set: Option<&vp_spec_model::RegistrySet>,
) -> Vec<Diagnostic> {
    let Some(mapping) = value.as_mapping() else {
        return vec![edition_diagnostic(
            RuleKind::ManifestYamlInvalid,
            "registry_snapshots must be a mapping of snapshot name to path",
            Some("use a YAML mapping such as `terminology: spec/terminology/registry.yaml`".into()),
            Some(manifest_path.to_path_buf()),
            Some(Location::yaml_path("registry_snapshots")),
        )];
    };

    let mut diagnostics = Vec::new();

    for (name_value, path_value) in mapping {
        let Some(name) = name_value.as_str() else {
            continue;
        };
        let Some(snapshot_ref) = path_value.as_str() else {
            continue;
        };

        let path = snapshot_path_before_rev(snapshot_ref);
        let yaml_path = format!("registry_snapshots.{name}");

        if !registry_snapshot_exists(repo, registry_set, path) {
            diagnostics.push(edition_diagnostic(
                RuleKind::RegistrySnapshotMissing,
                format!("registry snapshot path `{path}` does not exist under the spec root"),
                Some(format!("add `{path}` or update registry_snapshots.{name}")),
                Some(manifest_path.to_path_buf()),
                Some(Location::yaml_path(&yaml_path)),
            ));
        }
    }

    diagnostics
}

fn validate_conformance_baseline(manifest_path: &Path, value: &Value) -> Vec<Diagnostic> {
    let Some(sequence) = value.as_sequence() else {
        return vec![edition_diagnostic(
            RuleKind::ManifestYamlInvalid,
            "conformance_baseline must be a sequence of VP-CS ids",
            Some("use a YAML list such as `- VP-CS-0001`".into()),
            Some(manifest_path.to_path_buf()),
            Some(Location::yaml_path("conformance_baseline")),
        )];
    };

    let re = conformance_id_regex();
    let mut diagnostics = Vec::new();

    for (index, entry) in sequence.iter().enumerate() {
        let Some(id) = entry.as_str() else {
            continue;
        };

        if !re.is_match(id) {
            diagnostics.push(edition_diagnostic(
                RuleKind::InvalidConformanceId,
                format!("conformance baseline id `{id}` does not match the VP-CS-NNNN pattern"),
                Some("use an identifier such as `VP-CS-0001`".into()),
                Some(manifest_path.to_path_buf()),
                Some(Location::yaml_path(format!(
                    "conformance_baseline[{index}]"
                ))),
            ));
        }
    }

    diagnostics
}

fn snapshot_path_before_rev(reference: &str) -> &str {
    reference.split('@').next().unwrap_or(reference)
}

fn is_valid_edition_id(edition_id: &str) -> bool {
    edition_id.starts_with("vp-edition-") && edition_id.len() > "vp-edition-".len()
}

fn parse_front_matter_version(content: &str) -> Option<String> {
    let rest = content.strip_prefix("---")?;
    let end = rest.find("\n---")?;
    let front_matter = &rest[..end];
    let mapping: Value = serde_yaml::from_str(front_matter).ok()?;
    mapping
        .as_mapping()?
        .get(Value::from("version"))?
        .as_str()
        .map(str::to_owned)
}

fn conformance_id_regex() -> Regex {
    Regex::new(r"^VP-CS-\d{4}$").expect("valid conformance id regex")
}

fn edition_diagnostic(
    kind: RuleKind,
    message: impl Into<String>,
    suggestion: Option<String>,
    file: Option<PathBuf>,
    location: Option<Location>,
) -> Diagnostic {
    let rule = RuleId::edition(kind);
    let mut diagnostic = Diagnostic::new(rule.default_severity(), rule, Category::Edition, message);

    if let Some(file) = file {
        diagnostic = diagnostic.with_file(file);
    }
    if let Some(location) = location {
        diagnostic = diagnostic.with_location(location);
    }
    if let Some(suggestion) = suggestion {
        diagnostic = diagnostic.with_suggestion(suggestion);
    }

    diagnostic
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edition_id_pattern() {
        assert!(is_valid_edition_id("vp-edition-genesis-1"));
        assert!(!is_valid_edition_id("genesis-1"));
        assert!(!is_valid_edition_id("vp-edition-"));
    }

    #[test]
    fn snapshot_path_strips_rev_suffix() {
        assert_eq!(
            snapshot_path_before_rev("spec/rfcs/registry.yaml@rev-2026-12-01"),
            "spec/rfcs/registry.yaml"
        );
    }

    #[test]
    fn parses_front_matter_version() {
        let content = "---\ntitle: Example\nversion: 0.1.0\n---\n# Body\n";
        assert_eq!(
            parse_front_matter_version(content).as_deref(),
            Some("0.1.0")
        );
    }
}
