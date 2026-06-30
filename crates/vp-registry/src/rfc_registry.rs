//! VP-RFC registry rules for `spec/rfcs/registry.yaml`.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use semver::Version;
use serde_yaml::Value;
use vp_core::{parse_yaml, ReadError, ValidationContext};
use vp_diagnostics::{Category, Diagnostic, Location, RuleId, RuleKind, Severity};

const REGISTRY_REL_PATH: &str = "spec/rfcs/registry.yaml";

const REQUIRED_TOP_LEVEL: &[&str] = &["spec", "title", "version", "status", "process_rfc", "rfcs"];

const REQUIRED_ENTRY_FIELDS: &[&str] = &[
    "id",
    "rfc",
    "title",
    "status",
    "type",
    "version",
    "created",
    "updated",
    "depends_on",
    "supersedes",
    "superseded_by",
    "related_terms",
    "related_architecture",
    "related_conformance",
    "path",
];

const ALLOWED_STATUS: &[&str] = &[
    "draft",
    "discussion",
    "review",
    "accepted",
    "implemented",
    "verified",
    "superseded",
    "archived",
    "rejected",
];

pub fn validate(ctx: &ValidationContext) -> Vec<Diagnostic> {
    let repo = ctx.repository();

    if !repo.is_file(REGISTRY_REL_PATH) {
        return vec![registry_diagnostic(
            RuleKind::RegistryMissing,
            "RFC registry file is missing",
            Some(format!(
                "create `{REGISTRY_REL_PATH}` or verify --spec points at a veritypay-spec root"
            )),
            None,
        )];
    }

    let contents = match repo.read_text(REGISTRY_REL_PATH) {
        Ok(c) => c,
        Err(ReadError::Io(err)) => {
            return vec![registry_diagnostic(
                RuleKind::RegistryMissing,
                format!("RFC registry file could not be read: {err}"),
                Some(format!("ensure `{REGISTRY_REL_PATH}` is readable")),
                None,
            )];
        }
        Err(ReadError::NotFound) => {
            return vec![registry_diagnostic(
                RuleKind::RegistryMissing,
                "RFC registry file is missing",
                Some(format!(
                    "create `{REGISTRY_REL_PATH}` or verify --spec points at a veritypay-spec root"
                )),
                None,
            )];
        }
        Err(ReadError::YamlParse(_)) => unreachable!("read_text does not parse YAML"),
    };

    let root: Value = match parse_yaml(&contents) {
        Ok(v) => v,
        Err(err) => {
            return vec![registry_diagnostic(
                RuleKind::RegistryYamlInvalid,
                format!("RFC registry YAML is invalid: {err}"),
                Some("fix YAML syntax in spec/rfcs/registry.yaml".into()),
                None,
            )];
        }
    };

    let mut diagnostics = Vec::new();

    let Some(mapping) = root.as_mapping() else {
        diagnostics.push(registry_diagnostic(
            RuleKind::RegistryYamlInvalid,
            "RFC registry root must be a YAML mapping",
            Some("wrap registry fields in a mapping at the document root".into()),
            None,
        ));
        return diagnostics;
    };

    for field in REQUIRED_TOP_LEVEL {
        if !mapping.contains_key(Value::from(*field)) {
            diagnostics.push(registry_diagnostic(
                RuleKind::TopLevelMissingField,
                format!("required top-level field `{field}` is missing"),
                Some(format!("add `{field}` to `{REGISTRY_REL_PATH}`")),
                Some(Location::yaml_path(*field)),
            ));
        }
    }

    let rfcs_value = mapping.get(Value::from("rfcs"));
    let Some(rfcs_seq) = rfcs_value.and_then(Value::as_sequence) else {
        if mapping.contains_key(Value::from("rfcs")) {
            diagnostics.push(registry_diagnostic(
                RuleKind::RegistryYamlInvalid,
                "field `rfcs` must be a YAML sequence",
                Some("use a list of RFC entries under `rfcs:`".into()),
                Some(Location::yaml_path("rfcs")),
            ));
        }
        return diagnostics;
    };

    if rfcs_seq.is_empty() {
        diagnostics.push(registry_diagnostic(
            RuleKind::EmptyList,
            "RFC registry `rfcs` list must not be empty",
            Some("add at least one RFC entry (e.g. VP-RFC-0000)".into()),
            Some(Location::yaml_path("rfcs")),
        ));
        return diagnostics;
    }

    let mut seen_ids: HashMap<String, usize> = HashMap::new();
    let mut ids: HashSet<String> = HashSet::new();

    for (index, entry) in rfcs_seq.iter().enumerate() {
        let base = format!("rfcs[{index}]");
        let Some(entry_map) = entry.as_mapping() else {
            diagnostics.push(registry_diagnostic(
                RuleKind::RegistryYamlInvalid,
                format!("{base} must be a YAML mapping"),
                Some("each item under `rfcs` must be a mapping".into()),
                Some(Location::yaml_path(&base)),
            ));
            continue;
        };

        for field in REQUIRED_ENTRY_FIELDS {
            if !entry_map.contains_key(Value::from(*field)) {
                diagnostics.push(registry_diagnostic(
                    RuleKind::EntryMissingField,
                    format!("{base} is missing required field `{field}`"),
                    Some(format!("add `{field}` to the RFC entry at {base}")),
                    Some(Location::yaml_path(format!("{base}.{field}"))),
                ));
            }
        }

        let id = string_field(entry_map, "id");
        let rfc = string_field(entry_map, "rfc");

        if let Some(id) = &id {
            if !is_valid_rfc_id(id) {
                diagnostics.push(registry_diagnostic(
                    RuleKind::InvalidId,
                    format!("{base} id `{id}` must match VP-RFC-NNNN (four digits)"),
                    Some("use an id such as VP-RFC-0000".into()),
                    Some(Location::yaml_path(format!("{base}.id"))),
                ));
            } else if let Some(first_index) = seen_ids.insert(id.clone(), index) {
                diagnostics.push(registry_diagnostic(
                    RuleKind::DuplicateId,
                    format!("duplicate RFC id `{id}` at {base} (first at rfcs[{first_index}])"),
                    Some("assign a unique VP-RFC-NNNN id to each entry".into()),
                    Some(Location::yaml_path(format!("{base}.id"))),
                ));
            } else {
                ids.insert(id.clone());
            }

            if let Some(rfc) = &rfc {
                if let Some(expected) = rfc_number_from_id(id) {
                    if rfc != &expected {
                        diagnostics.push(registry_diagnostic(
                            RuleKind::IdNumberMismatch,
                            format!(
                                "{base} rfc `{rfc}` does not match id `{id}` (expected `{expected}`)"
                            ),
                            Some(format!("set rfc to `{expected}` for id `{id}`")),
                            Some(Location::yaml_path(format!("{base}.rfc"))),
                        ));
                    }
                }
            }

            if id == "VP-RFC-0000" && rfc.as_deref() != Some("0000") {
                diagnostics.push(registry_diagnostic(
                    RuleKind::IdNumberMismatch,
                    format!(
                        "{base} VP-RFC-0000 must have rfc: 0000 (found {:?})",
                        rfc.as_deref()
                    ),
                    Some("set rfc: 0000 for VP-RFC-0000".into()),
                    Some(Location::yaml_path(format!("{base}.rfc"))),
                ));
            }
        }

        if let Some(status) = string_field(entry_map, "status") {
            if !ALLOWED_STATUS.contains(&status.as_str()) {
                diagnostics.push(registry_diagnostic(
                    RuleKind::UnknownStatus,
                    format!("{base} status `{status}` is not allowed"),
                    Some(format!("use one of: {}", ALLOWED_STATUS.join(", "))),
                    Some(Location::yaml_path(format!("{base}.status"))),
                ));
            }
        }

        if let Some(version) = string_field(entry_map, "version") {
            if !is_valid_semver(&version) {
                diagnostics.push(registry_diagnostic(
                    RuleKind::InvalidVersion,
                    format!("{base} version `{version}` is not valid semver"),
                    Some("use semver such as 1.0.0 or 1.1.0".into()),
                    Some(Location::yaml_path(format!("{base}.version"))),
                ));
            }
        }

        if let Some(path) = string_field(entry_map, "path") {
            if !repo.is_file(&path) {
                diagnostics.push(registry_diagnostic(
                    RuleKind::MissingPath,
                    format!("{base} path `{path}` does not exist under spec root"),
                    Some(format!(
                        "create the file at `{path}` or correct the path in `{REGISTRY_REL_PATH}`"
                    )),
                    Some(Location::yaml_path(format!("{base}.path"))),
                ));
            }
        }
    }

    if let Some(version) = string_field(mapping, "version") {
        if !is_valid_semver(&version) {
            diagnostics.push(registry_diagnostic(
                RuleKind::InvalidVersion,
                format!("top-level version `{version}` is not valid semver"),
                Some("use semver such as 1.0.0".into()),
                Some(Location::yaml_path("version")),
            ));
        }
    }

    for (index, entry) in rfcs_seq.iter().enumerate() {
        let base = format!("rfcs[{index}]");
        let Some(entry_map) = entry.as_mapping() else {
            continue;
        };

        validate_reference_list(&mut diagnostics, entry_map, &base, "depends_on", &ids);
        validate_reference_list(&mut diagnostics, entry_map, &base, "supersedes", &ids);
        validate_optional_reference(&mut diagnostics, entry_map, &base, "superseded_by", &ids);
    }

    diagnostics
}

fn validate_reference_list(
    diagnostics: &mut Vec<Diagnostic>,
    entry: &serde_yaml::Mapping,
    base: &str,
    field: &str,
    ids: &HashSet<String>,
) {
    let Some(value) = entry.get(Value::from(field)) else {
        return;
    };

    let Some(list) = value.as_sequence() else {
        diagnostics.push(registry_diagnostic(
            RuleKind::RegistryYamlInvalid,
            format!("{base}.{field} must be a sequence"),
            Some(format!(
                "use a YAML list for `{field}` (empty list is allowed)"
            )),
            Some(Location::yaml_path(format!("{base}.{field}"))),
        ));
        return;
    };

    for (ref_index, item) in list.iter().enumerate() {
        let Some(ref_id) = item.as_str() else {
            diagnostics.push(registry_diagnostic(
                RuleKind::UnknownReference,
                format!("{base}.{field}[{ref_index}] must be a VP-RFC id string"),
                Some("reference entries must be VP-RFC-NNNN strings".into()),
                Some(Location::yaml_path(format!("{base}.{field}[{ref_index}]"))),
            ));
            continue;
        };

        if !ids.contains(ref_id) {
            diagnostics.push(registry_diagnostic(
                RuleKind::UnknownReference,
                format!("{base}.{field} references unknown RFC id `{ref_id}`"),
                Some(format!(
                    "add `{ref_id}` to the registry or fix the reference at {base}.{field}[{ref_index}]"
                )),
                Some(Location::yaml_path(format!("{base}.{field}[{ref_index}]"))),
            ));
        }
    }
}

fn validate_optional_reference(
    diagnostics: &mut Vec<Diagnostic>,
    entry: &serde_yaml::Mapping,
    base: &str,
    field: &str,
    ids: &HashSet<String>,
) {
    let Some(value) = entry.get(Value::from(field)) else {
        return;
    };

    if value.is_null() {
        return;
    }

    let Some(ref_id) = value.as_str() else {
        diagnostics.push(registry_diagnostic(
            RuleKind::UnknownReference,
            format!("{base}.{field} must be null or a VP-RFC id string"),
            Some("use null or a registered VP-RFC-NNNN id".into()),
            Some(Location::yaml_path(format!("{base}.{field}"))),
        ));
        return;
    };

    if !ids.contains(ref_id) {
        diagnostics.push(registry_diagnostic(
            RuleKind::UnknownReference,
            format!("{base}.{field} references unknown RFC id `{ref_id}`"),
            Some(format!(
                "add `{ref_id}` to the registry or set `{field}` to null"
            )),
            Some(Location::yaml_path(format!("{base}.{field}"))),
        ));
    }
}

fn string_field(mapping: &serde_yaml::Mapping, field: &str) -> Option<String> {
    mapping
        .get(Value::from(field))
        .and_then(|v| v.as_str())
        .map(str::to_owned)
}

fn is_valid_rfc_id(id: &str) -> bool {
    let Some(suffix) = id.strip_prefix("VP-RFC-") else {
        return false;
    };
    suffix.len() == 4 && suffix.chars().all(|c| c.is_ascii_digit())
}

fn rfc_number_from_id(id: &str) -> Option<String> {
    id.strip_prefix("VP-RFC-").map(str::to_owned)
}

fn is_valid_semver(version: &str) -> bool {
    Version::parse(version).is_ok()
}

fn registry_diagnostic(
    kind: RuleKind,
    message: impl Into<String>,
    suggestion: Option<String>,
    location: Option<Location>,
) -> Diagnostic {
    let mut diagnostic = Diagnostic::new(
        Severity::Error,
        RuleId::rfc(kind),
        Category::Registry,
        message,
    )
    .with_file(PathBuf::from(REGISTRY_REL_PATH));

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
    use std::fs;
    use std::io::Result as IoResult;
    use std::path::Path;

    fn write_registry(dir: &Path, contents: &str) -> IoResult<()> {
        let spec_rfcs = dir.join("spec/rfcs");
        fs::create_dir_all(spec_rfcs)?;
        fs::write(dir.join(REGISTRY_REL_PATH), contents)
    }

    fn ctx(dir: &Path) -> ValidationContext {
        ValidationContext::new(dir)
    }

    fn has_rule(diagnostics: &[Diagnostic], rule_id: &str) -> bool {
        diagnostics.iter().any(|d| d.rule_id() == rule_id)
    }

    const VALID_MINIMAL: &str = r#"
spec: RFC-REGISTRY
title: Test RFC Registry
version: 1.0.0
status: draft
process_rfc: VP-RFC-0000
rfcs:
  - id: VP-RFC-0000
    rfc: 0000
    title: RFC Process
    status: accepted
    type: meta
    version: 1.0.0
    created: 2026-06-29
    updated: 2026-06-29
    depends_on: []
    supersedes: []
    superseded_by: null
    related_terms: []
    related_architecture: []
    related_conformance: []
    path: rfcs/0000-rfc-process.md
"#;

    #[test]
    fn valid_minimal_registry() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_registry(dir.path(), VALID_MINIMAL).expect("write registry");
        fs::create_dir_all(dir.path().join("rfcs")).expect("rfcs dir");
        fs::write(dir.path().join("rfcs/0000-rfc-process.md"), "# RFC").expect("rfc file");

        let diagnostics = validate(&ctx(dir.path()));
        assert!(
            diagnostics.is_empty(),
            "unexpected diagnostics: {diagnostics:?}"
        );
    }

    #[test]
    fn missing_registry_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let diagnostics = validate(&ctx(dir.path()));
        assert!(has_rule(&diagnostics, "vp-rfc-registry-missing"));
    }

    #[test]
    fn invalid_yaml() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_registry(dir.path(), "rfcs: [").expect("write registry");
        let diagnostics = validate(&ctx(dir.path()));
        assert!(has_rule(&diagnostics, "vp-rfc-registry-yaml-invalid"));
    }

    #[test]
    fn duplicate_id() {
        let dir = tempfile::tempdir().expect("tempdir");
        let yaml = r#"
spec: RFC-REGISTRY
title: Test
version: 1.0.0
status: draft
process_rfc: VP-RFC-0000
rfcs:
  - id: VP-RFC-0000
    rfc: 0000
    title: One
    status: accepted
    type: meta
    version: 1.0.0
    created: 2026-06-29
    updated: 2026-06-29
    depends_on: []
    supersedes: []
    superseded_by: null
    related_terms: []
    related_architecture: []
    related_conformance: []
    path: rfcs/a.md
  - id: VP-RFC-0000
    rfc: 0000
    title: Two
    status: accepted
    type: meta
    version: 1.0.0
    created: 2026-06-29
    updated: 2026-06-29
    depends_on: []
    supersedes: []
    superseded_by: null
    related_terms: []
    related_architecture: []
    related_conformance: []
    path: rfcs/b.md
"#;
        write_registry(dir.path(), yaml).expect("write");
        fs::create_dir_all(dir.path().join("rfcs")).expect("rfcs");
        fs::write(dir.path().join("rfcs/a.md"), "a").expect("a");
        fs::write(dir.path().join("rfcs/b.md"), "b").expect("b");

        let diagnostics = validate(&ctx(dir.path()));
        assert!(has_rule(&diagnostics, "vp-rfc-duplicate-id"));
    }

    #[test]
    fn unknown_status() {
        let dir = tempfile::tempdir().expect("tempdir");
        let yaml = VALID_MINIMAL.replace("status: accepted", "status: published");
        write_registry(dir.path(), &yaml).expect("write");
        fs::create_dir_all(dir.path().join("rfcs")).expect("dir");
        fs::write(dir.path().join("rfcs/0000-rfc-process.md"), "#").expect("file");

        let diagnostics = validate(&ctx(dir.path()));
        assert!(has_rule(&diagnostics, "vp-rfc-unknown-status"));
    }

    #[test]
    fn missing_path() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_registry(dir.path(), VALID_MINIMAL).expect("write");

        let diagnostics = validate(&ctx(dir.path()));
        assert!(has_rule(&diagnostics, "vp-rfc-path-missing"));
    }

    #[test]
    fn unknown_dependency() {
        let dir = tempfile::tempdir().expect("tempdir");
        let yaml = VALID_MINIMAL.replace("depends_on: []", "depends_on: [VP-RFC-9999]");
        write_registry(dir.path(), &yaml).expect("write");
        fs::create_dir_all(dir.path().join("rfcs")).expect("dir");
        fs::write(dir.path().join("rfcs/0000-rfc-process.md"), "#").expect("file");

        let diagnostics = validate(&ctx(dir.path()));
        assert!(has_rule(&diagnostics, "vp-rfc-unknown-reference"));
    }
}
