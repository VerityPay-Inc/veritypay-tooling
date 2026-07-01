//! JSON validation output (Milestone C.3.2).

use std::io::{self, Write};

use serde::Serialize;
use vp_diagnostics::{Category, Diagnostic, Severity};
use vp_engine::ValidationResult;

/// Top-level JSON output for `vp validate --format json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ValidationJson {
    pub summary: SummaryJson,
    pub diagnostics: Vec<DiagnosticJson>,
}

/// Aggregated severity counts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SummaryJson {
    pub errors: usize,
    pub warnings: usize,
    pub info: usize,
}

/// A single serialized diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DiagnosticJson {
    pub severity: &'static str,
    pub rule_id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub category: &'static str,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<LocationJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related: Option<String>,
}

/// Optional file location within a diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LocationJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

impl ValidationJson {
    pub fn from_result(result: &ValidationResult) -> Self {
        let report = &result.report;
        Self {
            summary: SummaryJson {
                errors: report.error_count,
                warnings: report.warning_count,
                info: report.info_count,
            },
            diagnostics: report
                .diagnostics
                .iter()
                .map(DiagnosticJson::from_diagnostic)
                .collect(),
        }
    }
}

impl DiagnosticJson {
    fn from_diagnostic(diagnostic: &Diagnostic) -> Self {
        Self {
            severity: severity_json(diagnostic.severity),
            rule_id: diagnostic.rule_id(),
            title: diagnostic.rule.title(),
            description: diagnostic.rule.description(),
            category: category_json(diagnostic.category),
            message: diagnostic.message.clone(),
            file: diagnostic
                .file
                .as_ref()
                .map(|path| path.display().to_string()),
            location: diagnostic.location.as_ref().map(|location| LocationJson {
                line: location.line,
                column: location.column,
                path: location.path.clone(),
            }),
            suggestion: diagnostic.suggestion.clone(),
            help: diagnostic.help.clone(),
            note: diagnostic.note.clone(),
            related: diagnostic.related.clone(),
        }
    }
}

fn severity_json(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
    }
}

fn category_json(category: Category) -> &'static str {
    match category {
        Category::Registry => "registry",
        Category::Metadata => "metadata",
        Category::CrossReference => "cross_reference",
        Category::Edition => "edition",
        Category::Documentation => "documentation",
        Category::Future => "future",
    }
}

/// Serialize validation output as pretty-printed JSON.
pub fn render_validation_json(result: &ValidationResult, out: &mut dyn Write) -> io::Result<()> {
    let payload = ValidationJson::from_result(result);
    serde_json::to_writer_pretty(&mut *out, &payload)?;
    writeln!(out)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use vp_core::ValidatorInfo;
    use vp_diagnostics::{Location, Report, RuleId, RuleKind};
    use vp_engine::ValidatorOutcome;

    #[test]
    fn serializes_summary_and_diagnostics() {
        let result = ValidationResult {
            report: Report::from_diagnostics(vec![
                Diagnostic::new(
                    Severity::Error,
                    RuleId::crossref(RuleKind::BrokenLink),
                    Category::CrossReference,
                    "broken link",
                )
                .with_file("docs/README.md")
                .with_location(Location::line_column(42, 9))
                .with_suggestion("Fix the href."),
            ]),
            validators: vec![],
        };

        let json = ValidationJson::from_result(&result);
        assert_eq!(json.summary.errors, 1);
        assert_eq!(json.summary.warnings, 0);
        assert_eq!(json.summary.info, 0);
        assert_eq!(json.diagnostics.len(), 1);
        assert_eq!(json.diagnostics[0].severity, "error");
        assert_eq!(json.diagnostics[0].rule_id, "vp-crossref-broken-link");
        assert_eq!(json.diagnostics[0].title, "Broken Link");
        assert!(!json.diagnostics[0].description.is_empty());
        assert_eq!(json.diagnostics[0].category, "cross_reference");
        assert_eq!(
            json.diagnostics[0].file.as_deref(),
            Some("docs/README.md")
        );
        assert_eq!(json.diagnostics[0].location.as_ref().unwrap().line, Some(42));
    }

    #[test]
    fn omits_optional_diagnostic_fields_when_absent() {
        let result = ValidationResult {
            report: Report::from_diagnostics(vec![Diagnostic::new(
                Severity::Warning,
                RuleId::rfc(RuleKind::UnknownStatus),
                Category::Registry,
                "unknown status",
            )]),
            validators: vec![ValidatorOutcome {
                info: ValidatorInfo {
                    id: "registry-rfc",
                    name: "RFC Registry",
                    description: "Validates the VP-RFC registry structure and references.",
                    category: Category::Registry,
                },
                passed: true,
            }],
        };

        let mut output = Vec::new();
        render_validation_json(&result, &mut output).unwrap();
        let text = String::from_utf8(output).unwrap();
        let value: serde_json::Value = serde_json::from_str(&text).unwrap();

        assert_eq!(value["summary"]["errors"], 0);
        assert_eq!(value["summary"]["warnings"], 1);
        assert_eq!(value["diagnostics"][0]["message"], "unknown status");
        assert!(value["diagnostics"][0].get("file").is_none());
        assert!(value["diagnostics"][0].get("location").is_none());
        assert!(value["diagnostics"][0].get("suggestion").is_none());
    }
}
