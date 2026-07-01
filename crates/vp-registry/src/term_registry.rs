//! VP-TERM registry rules for `spec/terminology/registry.yaml`.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use serde_yaml::Value;
use vp_core::ValidationContext;
use vp_diagnostics::{Category, Diagnostic, Location, RuleId, RuleKind, Severity};
use vp_spec_model::TerminologyRegistry;

use crate::registry_source::{
    parse_registry_root, read_registry_text, try_load_terminology_registry, RegistryReadOutcome,
};

const REGISTRY_REL_PATH: &str = "spec/terminology/registry.yaml";

const REQUIRED_TOP_LEVEL: &[&str] = &["spec", "title", "version", "status", "terms"];

const REQUIRED_ENTRY_FIELDS: &[&str] = &[
    "id",
    "title",
    "stability",
    "classification",
    "normative_definition",
    "referenced_by",
    "depends_on",
];

const ALLOWED_STABILITY: &[&str] = &[
    "proposed",
    "experimental",
    "stable",
    "reserved",
    "deprecated",
];

const SECTION_ID_PREFIXES: &[&str] = &["DM", "IM", "BM", "DAT", "SM", "CM", "GV", "VI", "GL"];

pub fn validate(ctx: &ValidationContext) -> Vec<Diagnostic> {
    let repo = ctx.repository();

    if !repo.is_file(REGISTRY_REL_PATH) {
        return vec![term_diagnostic(
            RuleKind::RegistryMissing,
            "terminology registry file is missing",
            Some(format!(
                "create `{REGISTRY_REL_PATH}` or verify --spec points at a veritypay-spec root"
            )),
            None,
        )];
    }

    let contents = match read_registry_text(repo, REGISTRY_REL_PATH) {
        Ok(text) => text,
        Err(RegistryReadOutcome::Missing) => {
            return vec![term_diagnostic(
                RuleKind::RegistryMissing,
                "terminology registry file is missing",
                Some(format!(
                    "create `{REGISTRY_REL_PATH}` or verify --spec points at a veritypay-spec root"
                )),
                None,
            )];
        }
        Err(RegistryReadOutcome::Io(err)) => {
            return vec![term_diagnostic(
                RuleKind::RegistryMissing,
                format!("terminology registry file could not be read: {err}"),
                Some(format!("ensure `{REGISTRY_REL_PATH}` is readable")),
                None,
            )];
        }
    };

    let root: Value = match parse_registry_root(&contents) {
        Ok(value) => value,
        Err(err) => {
            return vec![term_diagnostic(
                RuleKind::RegistryYamlInvalid,
                format!("terminology registry YAML is invalid: {err}"),
                Some("fix YAML syntax in spec/terminology/registry.yaml".into()),
                None,
            )];
        }
    };

    let mut diagnostics = validate_term_structure(&root);

    let Some(mapping) = root.as_mapping() else {
        return diagnostics;
    };

    let Some(terms_seq) = mapping
        .get(Value::from("terms"))
        .and_then(Value::as_sequence)
    else {
        return diagnostics;
    };

    if terms_seq.is_empty() {
        return diagnostics;
    }

    if let Some(registry) = try_load_terminology_registry(repo) {
        validate_term_semantics_typed(&registry, &mut diagnostics);
    } else {
        validate_term_semantics_raw(terms_seq, &mut diagnostics);
    }

    diagnostics
}

fn validate_term_structure(root: &Value) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    let Some(mapping) = root.as_mapping() else {
        diagnostics.push(term_diagnostic(
            RuleKind::RegistryYamlInvalid,
            "terminology registry root must be a YAML mapping",
            Some("wrap registry fields in a mapping at the document root".into()),
            None,
        ));
        return diagnostics;
    };

    for field in REQUIRED_TOP_LEVEL {
        if !mapping.contains_key(Value::from(*field)) {
            diagnostics.push(term_diagnostic(
                RuleKind::TopLevelMissingField,
                format!("required top-level field `{field}` is missing"),
                Some(format!("add `{field}` to `{REGISTRY_REL_PATH}`")),
                Some(Location::yaml_path(*field)),
            ));
        }
    }

    let terms_value = mapping.get(Value::from("terms"));
    let Some(terms_seq) = terms_value.and_then(Value::as_sequence) else {
        if mapping.contains_key(Value::from("terms")) {
            diagnostics.push(term_diagnostic(
                RuleKind::RegistryYamlInvalid,
                "field `terms` must be a YAML sequence",
                Some("use a list of term entries under `terms:`".into()),
                Some(Location::yaml_path("terms")),
            ));
        }
        return diagnostics;
    };

    if terms_seq.is_empty() {
        diagnostics.push(term_diagnostic(
            RuleKind::EmptyList,
            "terminology registry `terms` list must not be empty",
            Some("add at least one term entry (e.g. VP-TERM-001)".into()),
            Some(Location::yaml_path("terms")),
        ));
        return diagnostics;
    }

    for (index, entry) in terms_seq.iter().enumerate() {
        let base = format!("terms[{index}]");
        let Some(entry_map) = entry.as_mapping() else {
            diagnostics.push(term_diagnostic(
                RuleKind::RegistryYamlInvalid,
                format!("{base} must be a YAML mapping"),
                Some("each item under `terms` must be a mapping".into()),
                Some(Location::yaml_path(&base)),
            ));
            continue;
        };

        for field in REQUIRED_ENTRY_FIELDS {
            if !entry_map.contains_key(Value::from(*field)) {
                diagnostics.push(term_diagnostic(
                    RuleKind::EntryMissingField,
                    format!("{base} is missing required field `{field}`"),
                    Some(format!("add `{field}` to the term entry at {base}")),
                    Some(Location::yaml_path(format!("{base}.{field}"))),
                ));
            }
        }

        validate_normative_definition(&mut diagnostics, entry_map, &base);

        if let Some(referenced_by) = entry_map.get(Value::from("referenced_by")) {
            if !referenced_by.is_sequence() {
                diagnostics.push(term_diagnostic(
                    RuleKind::InvalidReferencedBy,
                    format!("{base}.referenced_by must be a sequence"),
                    Some("use a YAML list for `referenced_by` (empty list is allowed)".into()),
                    Some(Location::yaml_path(format!("{base}.referenced_by"))),
                ));
            }
        }
    }

    diagnostics
}

fn validate_term_semantics_typed(
    registry: &TerminologyRegistry,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut seen_ids: HashMap<String, usize> = HashMap::new();
    let mut seen_titles: HashMap<String, usize> = HashMap::new();
    let mut ids: HashSet<String> = HashSet::new();

    for (index, entry) in registry.entries().iter().enumerate() {
        let base = format!("terms[{index}]");
        let id = &entry.id;

        if !is_valid_term_id(id) {
            diagnostics.push(term_diagnostic(
                RuleKind::InvalidId,
                format!("{base} id `{id}` must match VP-TERM-NNN (3–4 digits)"),
                Some("use an id such as VP-TERM-001".into()),
                Some(Location::yaml_path(format!("{base}.id"))),
            ));
        } else if let Some(first_index) = seen_ids.insert(id.clone(), index) {
            diagnostics.push(term_diagnostic(
                RuleKind::DuplicateId,
                format!("duplicate term id `{id}` at {base} (first at terms[{first_index}])"),
                Some("assign a unique VP-TERM-NNN id to each entry".into()),
                Some(Location::yaml_path(format!("{base}.id"))),
            ));
        } else {
            ids.insert(id.clone());
        }

        if let Some(first_index) = seen_titles.insert(entry.title.clone(), index) {
            diagnostics.push(term_diagnostic(
                RuleKind::DuplicateTitle,
                format!(
                    "duplicate term title `{}` at {base} (first at terms[{first_index}])",
                    entry.title
                ),
                Some("use a unique title for each VP-TERM entry".into()),
                Some(Location::yaml_path(format!("{base}.title"))),
            ));
        }

        if !ALLOWED_STABILITY.contains(&entry.stability.as_str()) {
            diagnostics.push(term_diagnostic(
                RuleKind::UnknownStability,
                format!("{base} stability `{}` is not allowed", entry.stability),
                Some(format!("use one of: {}", ALLOWED_STABILITY.join(", "))),
                Some(Location::yaml_path(format!("{base}.stability"))),
            ));
        }

        if let Some(normative_definition) = &entry.normative_definition {
            if !is_valid_section_id(&normative_definition.section_id) {
                diagnostics.push(term_diagnostic(
                    RuleKind::InvalidSectionId,
                    format!(
                        "{base} section_id `{}` has an unrecognized prefix",
                        normative_definition.section_id
                    ),
                    Some(format!(
                        "use a section_id with a known prefix: {}",
                        SECTION_ID_PREFIXES.join(", ")
                    )),
                    Some(Location::yaml_path(format!(
                        "{base}.normative_definition.section_id"
                    ))),
                ));
            }
        }
    }

    for (index, entry) in registry.entries().iter().enumerate() {
        let base = format!("terms[{index}]");
        validate_depends_on_typed(diagnostics, &base, &entry.depends_on, &ids);
    }
}

fn validate_term_semantics_raw(terms_seq: &[Value], diagnostics: &mut Vec<Diagnostic>) {
    let mut seen_ids: HashMap<String, usize> = HashMap::new();
    let mut seen_titles: HashMap<String, usize> = HashMap::new();
    let mut ids: HashSet<String> = HashSet::new();

    for (index, entry) in terms_seq.iter().enumerate() {
        let base = format!("terms[{index}]");
        let Some(entry_map) = entry.as_mapping() else {
            continue;
        };

        if let Some(id) = string_field(entry_map, "id") {
            if !is_valid_term_id(&id) {
                diagnostics.push(term_diagnostic(
                    RuleKind::InvalidId,
                    format!("{base} id `{id}` must match VP-TERM-NNN (3–4 digits)"),
                    Some("use an id such as VP-TERM-001".into()),
                    Some(Location::yaml_path(format!("{base}.id"))),
                ));
            } else if let Some(first_index) = seen_ids.insert(id.clone(), index) {
                diagnostics.push(term_diagnostic(
                    RuleKind::DuplicateId,
                    format!("duplicate term id `{id}` at {base} (first at terms[{first_index}])"),
                    Some("assign a unique VP-TERM-NNN id to each entry".into()),
                    Some(Location::yaml_path(format!("{base}.id"))),
                ));
            } else {
                ids.insert(id);
            }
        }

        if let Some(title) = string_field(entry_map, "title") {
            if let Some(first_index) = seen_titles.insert(title.clone(), index) {
                diagnostics.push(term_diagnostic(
                    RuleKind::DuplicateTitle,
                    format!(
                        "duplicate term title `{title}` at {base} (first at terms[{first_index}])"
                    ),
                    Some("use a unique title for each VP-TERM entry".into()),
                    Some(Location::yaml_path(format!("{base}.title"))),
                ));
            }
        }

        if let Some(stability) = string_field(entry_map, "stability") {
            if !ALLOWED_STABILITY.contains(&stability.as_str()) {
                diagnostics.push(term_diagnostic(
                    RuleKind::UnknownStability,
                    format!("{base} stability `{stability}` is not allowed"),
                    Some(format!("use one of: {}", ALLOWED_STABILITY.join(", "))),
                    Some(Location::yaml_path(format!("{base}.stability"))),
                ));
            }
        }

        if let Some(value) = entry_map.get(Value::from("normative_definition")) {
            if let Some(def_map) = value.as_mapping() {
                if let Some(section_id) = string_field(def_map, "section_id") {
                    if !is_valid_section_id(&section_id) {
                        diagnostics.push(term_diagnostic(
                            RuleKind::InvalidSectionId,
                            format!("{base} section_id `{section_id}` has an unrecognized prefix"),
                            Some(format!(
                                "use a section_id with a known prefix: {}",
                                SECTION_ID_PREFIXES.join(", ")
                            )),
                            Some(Location::yaml_path(format!(
                                "{base}.normative_definition.section_id"
                            ))),
                        ));
                    }
                }
            }
        }
    }

    for (index, entry) in terms_seq.iter().enumerate() {
        let base = format!("terms[{index}]");
        let Some(entry_map) = entry.as_mapping() else {
            continue;
        };
        validate_depends_on(diagnostics, entry_map, &base, &ids);
    }
}

fn validate_depends_on_typed(
    diagnostics: &mut Vec<Diagnostic>,
    base: &str,
    depends_on: &[String],
    ids: &HashSet<String>,
) {
    for (ref_index, ref_id) in depends_on.iter().enumerate() {
        if !ids.contains(ref_id) {
            diagnostics.push(term_diagnostic(
                RuleKind::UnknownReference,
                format!("{base}.depends_on references unknown term id `{ref_id}`"),
                Some(format!(
                    "add `{ref_id}` to the registry or fix depends_on at {base}.depends_on[{ref_index}]"
                )),
                Some(Location::yaml_path(format!("{base}.depends_on[{ref_index}]"))),
            ));
        }
    }
}

fn validate_normative_definition(
    diagnostics: &mut Vec<Diagnostic>,
    entry: &serde_yaml::Mapping,
    base: &str,
) {
    let Some(value) = entry.get(Value::from("normative_definition")) else {
        return;
    };

    if value.is_null() {
        return;
    }

    let Some(def_map) = value.as_mapping() else {
        diagnostics.push(term_diagnostic(
            RuleKind::InvalidNormativeDefinition,
            format!("{base}.normative_definition must be a mapping"),
            Some("provide `document` and `section_id` under normative_definition".into()),
            Some(Location::yaml_path(format!("{base}.normative_definition"))),
        ));
        return;
    };

    let has_document = def_map.contains_key(Value::from("document"));
    let has_section_id = def_map.contains_key(Value::from("section_id"));

    if !has_document || !has_section_id {
        diagnostics.push(term_diagnostic(
            RuleKind::InvalidNormativeDefinition,
            format!("{base}.normative_definition requires `document` and `section_id`"),
            Some("add both fields under normative_definition".into()),
            Some(Location::yaml_path(format!("{base}.normative_definition"))),
        ));
    }

    if let Some(section_id) = string_field(def_map, "section_id") {
        if !is_valid_section_id(&section_id) {
            diagnostics.push(term_diagnostic(
                RuleKind::InvalidSectionId,
                format!("{base} section_id `{section_id}` has an unrecognized prefix"),
                Some(format!(
                    "use a section_id with a known prefix: {}",
                    SECTION_ID_PREFIXES.join(", ")
                )),
                Some(Location::yaml_path(format!(
                    "{base}.normative_definition.section_id"
                ))),
            ));
        }
    }
}

fn validate_depends_on(
    diagnostics: &mut Vec<Diagnostic>,
    entry: &serde_yaml::Mapping,
    base: &str,
    ids: &HashSet<String>,
) {
    let Some(value) = entry.get(Value::from("depends_on")) else {
        return;
    };

    let Some(list) = value.as_sequence() else {
        diagnostics.push(term_diagnostic(
            RuleKind::UnknownReference,
            format!("{base}.depends_on must be a sequence"),
            Some("use a YAML list for `depends_on` (empty list is allowed)".into()),
            Some(Location::yaml_path(format!("{base}.depends_on"))),
        ));
        return;
    };

    for (ref_index, item) in list.iter().enumerate() {
        let Some(ref_id) = item.as_str() else {
            diagnostics.push(term_diagnostic(
                RuleKind::UnknownReference,
                format!("{base}.depends_on[{ref_index}] must be a VP-TERM id string"),
                Some("reference entries must be VP-TERM-NNN strings".into()),
                Some(Location::yaml_path(format!(
                    "{base}.depends_on[{ref_index}]"
                ))),
            ));
            continue;
        };

        if !ids.contains(ref_id) {
            diagnostics.push(term_diagnostic(
                RuleKind::UnknownReference,
                format!("{base}.depends_on references unknown term id `{ref_id}`"),
                Some(format!(
                    "add `{ref_id}` to the registry or fix depends_on at {base}.depends_on[{ref_index}]"
                )),
                Some(Location::yaml_path(format!("{base}.depends_on[{ref_index}]"))),
            ));
        }
    }
}

fn string_field(mapping: &serde_yaml::Mapping, field: &str) -> Option<String> {
    mapping
        .get(Value::from(field))
        .and_then(|v| v.as_str())
        .map(str::to_owned)
}

fn is_valid_term_id(id: &str) -> bool {
    let Some(suffix) = id.strip_prefix("VP-TERM-") else {
        return false;
    };
    (suffix.len() == 3 || suffix.len() == 4) && suffix.chars().all(|c| c.is_ascii_digit())
}

fn is_valid_section_id(section_id: &str) -> bool {
    SECTION_ID_PREFIXES.iter().any(|prefix| {
        section_id
            .strip_prefix(prefix)
            .is_some_and(|rest| rest.starts_with('-') && rest.len() > 1)
    })
}

fn term_diagnostic(
    kind: RuleKind,
    message: impl Into<String>,
    suggestion: Option<String>,
    location: Option<Location>,
) -> Diagnostic {
    let mut diagnostic = Diagnostic::new(
        Severity::Error,
        RuleId::term(kind),
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
        let spec_terms = dir.join("spec/terminology");
        fs::create_dir_all(spec_terms)?;
        fs::write(dir.join(REGISTRY_REL_PATH), contents)
    }

    fn ctx(dir: &Path) -> ValidationContext {
        ValidationContext::new(dir)
    }

    fn has_rule(diagnostics: &[Diagnostic], rule_id: &str) -> bool {
        diagnostics.iter().any(|d| d.rule_id() == rule_id)
    }

    const VALID_MINIMAL: &str = r#"
spec: SPEC-0004
title: Test Terminology Registry
version: 0.1.0
status: draft
terms:
  - id: VP-TERM-001
    title: Protocol
    stability: stable
    classification: fundamental
    depends_on: []
    normative_definition:
      document: DOMAIN_MODEL
      section_id: DM-1.1
    referenced_by:
      - MANIFESTO
"#;

    #[test]
    fn valid_minimal_registry() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_registry(dir.path(), VALID_MINIMAL).expect("write registry");
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
        assert!(has_rule(&diagnostics, "vp-term-registry-missing"));
    }

    #[test]
    fn invalid_yaml() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_registry(dir.path(), "terms: [").expect("write registry");
        let diagnostics = validate(&ctx(dir.path()));
        assert!(has_rule(&diagnostics, "vp-term-registry-yaml-invalid"));
    }

    #[test]
    fn duplicate_id() {
        let dir = tempfile::tempdir().expect("tempdir");
        let yaml = r#"
spec: SPEC-0004
title: Test
version: 0.1.0
status: draft
terms:
  - id: VP-TERM-001
    title: One
    stability: stable
    classification: fundamental
    depends_on: []
    normative_definition:
      document: DOMAIN_MODEL
      section_id: DM-1.1
    referenced_by: []
  - id: VP-TERM-001
    title: Two
    stability: stable
    classification: fundamental
    depends_on: []
    normative_definition:
      document: DOMAIN_MODEL
      section_id: DM-1.2
    referenced_by: []
"#;
        write_registry(dir.path(), yaml).expect("write");
        let diagnostics = validate(&ctx(dir.path()));
        assert!(has_rule(&diagnostics, "vp-term-duplicate-id"));
    }

    #[test]
    fn duplicate_title() {
        let dir = tempfile::tempdir().expect("tempdir");
        let yaml = r#"
spec: SPEC-0004
title: Test
version: 0.1.0
status: draft
terms:
  - id: VP-TERM-001
    title: Protocol
    stability: stable
    classification: fundamental
    depends_on: []
    normative_definition:
      document: DOMAIN_MODEL
      section_id: DM-1.1
    referenced_by: []
  - id: VP-TERM-002
    title: Protocol
    stability: stable
    classification: fundamental
    depends_on: []
    normative_definition:
      document: DOMAIN_MODEL
      section_id: DM-1.2
    referenced_by: []
"#;
        write_registry(dir.path(), yaml).expect("write");
        let diagnostics = validate(&ctx(dir.path()));
        assert!(has_rule(&diagnostics, "vp-term-duplicate-title"));
    }

    #[test]
    fn unknown_stability() {
        let dir = tempfile::tempdir().expect("tempdir");
        let yaml = VALID_MINIMAL.replace("stability: stable", "stability: retired");
        write_registry(dir.path(), &yaml).expect("write");
        let diagnostics = validate(&ctx(dir.path()));
        assert!(has_rule(&diagnostics, "vp-term-unknown-stability"));
    }

    #[test]
    fn missing_normative_definition_fields() {
        let dir = tempfile::tempdir().expect("tempdir");
        let yaml = VALID_MINIMAL.replace(
            "normative_definition:\n      document: DOMAIN_MODEL\n      section_id: DM-1.1",
            "normative_definition:\n      document: DOMAIN_MODEL",
        );
        write_registry(dir.path(), &yaml).expect("write");
        let diagnostics = validate(&ctx(dir.path()));
        assert!(has_rule(
            &diagnostics,
            "vp-term-invalid-normative-definition"
        ));
    }

    #[test]
    fn unknown_depends_on() {
        let dir = tempfile::tempdir().expect("tempdir");
        let yaml = VALID_MINIMAL.replace("depends_on: []", "depends_on: [VP-TERM-999]");
        write_registry(dir.path(), &yaml).expect("write");
        let diagnostics = validate(&ctx(dir.path()));
        assert!(has_rule(&diagnostics, "vp-term-unknown-reference"));
    }

    #[test]
    fn invalid_section_id() {
        let dir = tempfile::tempdir().expect("tempdir");
        let yaml = VALID_MINIMAL.replace("section_id: DM-1.1", "section_id: XX-1.1");
        write_registry(dir.path(), &yaml).expect("write");
        let diagnostics = validate(&ctx(dir.path()));
        assert!(has_rule(&diagnostics, "vp-term-invalid-section-id"));
    }
}
