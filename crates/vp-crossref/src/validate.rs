//! Cross-reference validation rules.

use std::path::Path;

use regex::Regex;
use vp_core::{ReadError, SpecRepository, ValidationContext};
use vp_diagnostics::{Category, Diagnostic, Location, RuleId, RuleKind, Severity};
use vp_spec_model::{ReferenceEdge, ReferenceGraph, ReferenceNodeKind};

use crate::constants::{RFC_ILLUSTRATIVE_DOCUMENTS, SECTION_ID_PREFIXES};
use crate::discovery::ReferenceDiscovery;
use crate::kind::ReferenceKind;
use crate::markdown::MarkdownDiscovery;
use crate::reference::Reference;
use crate::resolve::{resolve_relative_link, split_link_target};
use crate::spec_model::CrossrefModel;

const VALID_TERM_ID: &str = r"^VP-TERM-\d{3,4}$";
const VALID_RFC_ID: &str = r"^VP-RFC-\d{4}$";

pub fn validate(ctx: &ValidationContext) -> Vec<Diagnostic> {
    let repo = ctx.repository();
    let model = CrossrefModel::load(repo);
    let mut diagnostics = Vec::new();

    for (rel_path, content) in model.scan_documents(repo) {
        diagnostics.extend(discover_invalid_reference_formats(&rel_path, &content));
    }

    if let Some(graph) = model.reference_graph() {
        for edge in graph.edges() {
            diagnostics.extend(validate_graph_edge(repo, &model, graph, edge));
        }
    } else {
        let discovery = MarkdownDiscovery::new();
        for (rel_path, content) in model.scan_documents(repo) {
            for reference in discovery.discover(&rel_path, &content) {
                diagnostics.extend(validate_reference(
                    repo, &model, &rel_path, &content, &reference,
                ));
            }
        }
    }

    diagnostics
}

fn validate_graph_edge(
    repo: &SpecRepository,
    model: &CrossrefModel,
    graph: &ReferenceGraph,
    edge: &ReferenceEdge,
) -> Vec<Diagnostic> {
    let Some(reference) = reference_from_edge(graph, edge) else {
        return Vec::new();
    };

    let source_path = reference.source_file.clone();
    let content = model
        .document_content(repo, &source_path)
        .unwrap_or_default();

    validate_reference(repo, model, &source_path, &content, &reference)
}

fn reference_from_edge(graph: &ReferenceGraph, edge: &ReferenceEdge) -> Option<Reference> {
    let source = graph.lookup(&edge.source)?;
    if source.kind != ReferenceNodeKind::Document {
        return None;
    }

    Some(Reference::new(
        edge.reference_kind,
        edge.symbolic_target.clone(),
        Path::new(&source.display_name),
        edge.source_location.clone(),
    ))
}

fn validate_reference(
    repo: &SpecRepository,
    model: &CrossrefModel,
    source_content_path: &Path,
    source_content: &str,
    reference: &Reference,
) -> Vec<Diagnostic> {
    match reference.kind {
        ReferenceKind::Terminology => validate_term_reference(model, reference),
        ReferenceKind::Rfc => validate_rfc_reference(model, reference),
        ReferenceKind::MarkdownFile => validate_markdown_file(model, repo, reference),
        ReferenceKind::MarkdownAnchor => {
            validate_markdown_anchor(model, repo, source_content_path, source_content, reference)
        }
        ReferenceKind::ArchitectureSection => validate_architecture_section(reference),
        ReferenceKind::Future => Vec::new(),
    }
}

fn validate_term_reference(model: &CrossrefModel, reference: &Reference) -> Vec<Diagnostic> {
    if model.term_is_known(&reference.target) {
        return Vec::new();
    }

    vec![crossref_diagnostic(
        RuleKind::UnknownTerm,
        format!("unknown VP-TERM reference `{}`", reference.target),
        Some(format!(
            "add `{}` to spec/terminology/registry.yaml or correct the citation",
            reference.target
        )),
        &reference.source_file,
        reference.location.clone(),
    )]
}

fn validate_rfc_reference(model: &CrossrefModel, reference: &Reference) -> Vec<Diagnostic> {
    if is_illustrative_rfc_document(&reference.source_file) {
        return Vec::new();
    }

    if model.rfc_is_known(&reference.target) {
        return Vec::new();
    }

    vec![crossref_diagnostic(
        RuleKind::UnknownRfc,
        format!("unknown VP-RFC reference `{}`", reference.target),
        Some(format!(
            "add `{}` to spec/rfcs/registry.yaml or correct the citation",
            reference.target
        )),
        &reference.source_file,
        reference.location.clone(),
    )]
}

fn validate_markdown_file(
    model: &CrossrefModel,
    repo: &SpecRepository,
    reference: &Reference,
) -> Vec<Diagnostic> {
    let (path_part, _) = split_link_target(&reference.target);
    let resolved = resolve_relative_link(&reference.source_file, &path_part);
    if model.link_target_exists(repo, &resolved) {
        return Vec::new();
    }

    vec![crossref_diagnostic(
        RuleKind::BrokenLink,
        format!("broken relative link `{}`", reference.target),
        Some(format!(
            "create `{}` or update the link in {}",
            resolved.display(),
            reference.source_file.display()
        )),
        &reference.source_file,
        reference.location.clone(),
    )]
}

fn validate_markdown_anchor(
    model: &CrossrefModel,
    repo: &SpecRepository,
    source_content_path: &Path,
    source_content: &str,
    reference: &Reference,
) -> Vec<Diagnostic> {
    let (path_part, anchor) = split_link_target(&reference.target);
    let Some(anchor) = anchor else {
        return validate_markdown_file(model, repo, reference);
    };

    let resolved = if path_part.is_empty() {
        source_content_path.to_path_buf()
    } else {
        resolve_relative_link(&reference.source_file, &path_part)
    };

    if !model.link_target_exists(repo, &resolved) {
        return vec![crossref_diagnostic(
            RuleKind::BrokenLink,
            format!("broken relative link `{}`", reference.target),
            Some(format!(
                "create `{}` or update the link in {}",
                resolved.display(),
                reference.source_file.display()
            )),
            &reference.source_file,
            reference.location.clone(),
        )];
    }

    if model
        .document_corpus
        .as_ref()
        .and_then(|corpus| corpus.get(&resolved))
        .is_none()
        && resolved != source_content_path
    {
        match repo.read_text(&resolved) {
            Err(ReadError::NotFound) if repo.canonical_path(&resolved).is_dir() => {
                return Vec::new();
            }
            Err(ReadError::NotFound) => {
                return vec![crossref_diagnostic(
                    RuleKind::BrokenLink,
                    format!("broken relative link `{}`", reference.target),
                    Some(format!(
                        "create `{}` under the spec root",
                        resolved.display()
                    )),
                    &reference.source_file,
                    reference.location.clone(),
                )];
            }
            Err(_) => return Vec::new(),
            Ok(_) => {}
        }
    }

    let anchors = model.document_anchors(repo, &resolved, source_content_path, source_content);

    if anchors.contains(&anchor) {
        return Vec::new();
    }

    vec![crossref_diagnostic(
        RuleKind::BrokenAnchor,
        format!("broken anchor `#{}` in link `{}`", anchor, reference.target),
        Some(format!(
            "add a matching heading to {} or fix the link fragment",
            resolved.display()
        )),
        &reference.source_file,
        reference.location.clone(),
    )]
}

fn validate_architecture_section(reference: &Reference) -> Vec<Diagnostic> {
    if is_valid_section_id(&reference.target) {
        return Vec::new();
    }

    vec![crossref_diagnostic(
        RuleKind::InvalidReferenceFormat,
        format!(
            "architecture section id `{}` has an unrecognized prefix",
            reference.target
        ),
        Some(format!(
            "use a section id with a known prefix: {}",
            SECTION_ID_PREFIXES.join(", ")
        )),
        &reference.source_file,
        reference.location.clone(),
    )]
}

fn discover_invalid_reference_formats(source_file: &Path, content: &str) -> Vec<Diagnostic> {
    static TERM_LAX: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    static RFC_LAX: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    static TERM_STRICT: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    static RFC_STRICT: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();

    let term_lax = TERM_LAX.get_or_init(|| Regex::new(r"VP-TERM-\d+").expect("term lax"));
    let rfc_lax = RFC_LAX.get_or_init(|| Regex::new(r"VP-RFC-\d+").expect("rfc lax"));
    let term_strict = TERM_STRICT.get_or_init(|| Regex::new(VALID_TERM_ID).expect("term strict"));
    let rfc_strict = RFC_STRICT.get_or_init(|| Regex::new(VALID_RFC_ID).expect("rfc strict"));

    let mut diagnostics = Vec::new();

    for m in term_lax.find_iter(content) {
        if term_strict.is_match(m.as_str()) {
            continue;
        }
        diagnostics.push(crossref_diagnostic(
            RuleKind::InvalidReferenceFormat,
            format!("`{}` is not a valid VP-TERM id", m.as_str()),
            Some("use VP-TERM-NNN or VP-TERM-NNNN".into()),
            source_file,
            location_at(content, m.start()),
        ));
    }

    for m in rfc_lax.find_iter(content) {
        if rfc_strict.is_match(m.as_str()) {
            continue;
        }
        diagnostics.push(crossref_diagnostic(
            RuleKind::InvalidReferenceFormat,
            format!("`{}` is not a valid VP-RFC id", m.as_str()),
            Some("use VP-RFC-NNNN (four digits)".into()),
            source_file,
            location_at(content, m.start()),
        ));
    }

    diagnostics
}

fn is_valid_section_id(section_id: &str) -> bool {
    SECTION_ID_PREFIXES.iter().any(|prefix| {
        section_id
            .strip_prefix(prefix)
            .is_some_and(|rest| rest.starts_with('-') && rest.len() > 1)
    })
}

fn is_illustrative_rfc_document(source_file: &Path) -> bool {
    RFC_ILLUSTRATIVE_DOCUMENTS
        .iter()
        .any(|path| source_file == Path::new(path))
}

fn crossref_diagnostic(
    kind: RuleKind,
    message: impl Into<String>,
    suggestion: Option<String>,
    file: &Path,
    location: Location,
) -> Diagnostic {
    let mut diagnostic = Diagnostic::new(
        Severity::Error,
        RuleId::crossref(kind),
        Category::CrossReference,
        message,
    )
    .with_file(file)
    .with_location(location);

    if let Some(suggestion) = suggestion {
        diagnostic = diagnostic.with_suggestion(suggestion);
    }

    diagnostic
}

fn location_at(content: &str, byte_offset: usize) -> Location {
    let before = &content[..byte_offset];
    let line = before.matches('\n').count() as u32 + 1;
    let line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
    let column = content[line_start..byte_offset].chars().count() as u32 + 1;
    Location::line_column(line, column)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    fn write_valid_registries(root: &Path) {
        let term_fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../vp-registry/tests/fixtures/term/valid/registry.yaml");
        let rfc_fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../vp-registry/tests/fixtures/valid/registry.yaml");

        fs::create_dir_all(root.join("spec/terminology")).expect("term dir");
        fs::create_dir_all(root.join("spec/rfcs")).expect("rfc dir");
        fs::copy(term_fixture, root.join("spec/terminology/registry.yaml")).expect("term registry");
        fs::copy(rfc_fixture, root.join("spec/rfcs/registry.yaml")).expect("rfc registry");
    }

    fn has_rule(diagnostics: &[Diagnostic], rule_id: &str) -> bool {
        diagnostics.iter().any(|d| d.rule_id() == rule_id)
    }

    #[test]
    fn unknown_term_reference_fails() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_valid_registries(dir.path());
        fs::create_dir_all(dir.path().join("docs")).expect("docs");
        fs::write(
            dir.path().join("docs/page.md"),
            "See VP-TERM-999 for details.",
        )
        .expect("write");

        let model = CrossrefModel::load(&vp_core::SpecRepository::new(dir.path()));
        assert!(model.uses_reference_graph());

        let diagnostics = validate(&ValidationContext::new(dir.path()));
        assert!(has_rule(&diagnostics, "vp-crossref-unknown-term"));
    }

    #[test]
    fn valid_term_reference_passes() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_valid_registries(dir.path());
        fs::create_dir_all(dir.path().join("docs")).expect("docs");
        fs::write(
            dir.path().join("docs/page.md"),
            "See VP-TERM-001 for details.",
        )
        .expect("write");

        let diagnostics = validate(&ValidationContext::new(dir.path()));
        assert!(!has_rule(&diagnostics, "vp-crossref-unknown-term"));
    }
}
