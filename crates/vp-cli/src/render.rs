//! Human-readable validation output (Milestone C.3.1).

use std::io::{self, Write};

use vp_diagnostics::{Category, Diagnostic, Report, Severity};
use vp_engine::ValidationResult;

const SUMMARY_RULE: &str = "────────────────────────────";

const CATEGORY_ORDER: [Category; 6] = [
    Category::Registry,
    Category::CrossReference,
    Category::Metadata,
    Category::Edition,
    Category::Documentation,
    Category::Future,
];

/// Render validation progress, grouped diagnostics, and summary to `out`.
pub fn render_validation(result: &ValidationResult, out: &mut dyn Write) -> io::Result<()> {
    render_validator_progress(&result.validators, out)?;
    render_diagnostics(&result.report, out)?;
    render_summary(result, out)
}

fn render_validator_progress(
    validators: &[vp_engine::ValidatorOutcome],
    out: &mut dyn Write,
) -> io::Result<()> {
    writeln!(out, "Running validators...")?;
    writeln!(out)?;

    for validator in validators {
        let mark = if validator.passed { "✓" } else { "✗" };
        writeln!(out, "{mark} {}", validator.label)?;
    }

    if !validators.is_empty() {
        writeln!(out)?;
    }

    Ok(())
}

fn render_diagnostics(report: &Report, out: &mut dyn Write) -> io::Result<()> {
    if report.diagnostics.is_empty() {
        return Ok(());
    }

    let mut wrote_group = false;

    for category in CATEGORY_ORDER {
        let group: Vec<&Diagnostic> = report
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.category == category)
            .collect();

        if group.is_empty() {
            continue;
        }

        if wrote_group {
            writeln!(out)?;
        }

        writeln!(out, "{}", category_title(category))?;
        writeln!(out)?;

        for (index, diagnostic) in group.iter().enumerate() {
            if index > 0 {
                writeln!(out)?;
            }
            render_diagnostic(diagnostic, out)?;
        }

        wrote_group = true;
    }

    if wrote_group {
        writeln!(out)?;
    }

    Ok(())
}

fn render_diagnostic(diagnostic: &Diagnostic, out: &mut dyn Write) -> io::Result<()> {
    writeln!(
        out,
        "{}[{}]",
        severity_label(diagnostic.severity),
        diagnostic.rule_id()
    )?;

    if let Some(location) = format_location(diagnostic) {
        writeln!(out, "  --> {location}")?;
        writeln!(out)?;
    }

    writeln!(out, "{}", diagnostic.message)?;

    if let Some(suggestion) = &diagnostic.suggestion {
        writeln!(out)?;
        writeln!(out, "Suggestion:")?;
        writeln!(out, "{suggestion}")?;
    }

    Ok(())
}

fn render_summary(result: &ValidationResult, out: &mut dyn Write) -> io::Result<()> {
    let report = &result.report;

    writeln!(out, "{SUMMARY_RULE}")?;
    writeln!(out)?;
    writeln!(out, "Validation Summary")?;
    writeln!(out)?;
    writeln!(out, "Errors:   {}", report.error_count)?;
    writeln!(out, "Warnings: {}", report.warning_count)?;
    writeln!(out, "Info:     {}", report.info_count)?;
    writeln!(out)?;

    if result.has_errors() {
        writeln!(out, "Validation failed.")?;
    } else {
        writeln!(out, "Validation passed.")?;
    }

    Ok(())
}

fn severity_label(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
    }
}

fn category_title(category: Category) -> &'static str {
    match category {
        Category::Registry => "Registry",
        Category::CrossReference => "Cross References",
        Category::Metadata => "Metadata",
        Category::Edition => "Edition",
        Category::Documentation => "Documentation",
        Category::Future => "Future",
    }
}

fn format_location(diagnostic: &Diagnostic) -> Option<String> {
    let file = diagnostic.file.as_ref()?;

    let mut location = file.display().to_string();

    if let Some(loc) = &diagnostic.location {
        if let Some(line) = loc.line {
            location.push(':');
            location.push_str(&line.to_string());
            if let Some(column) = loc.column {
                location.push(':');
                location.push_str(&column.to_string());
            }
        } else if let Some(path) = &loc.path {
            location.push(':');
            location.push_str(path);
        }
    }

    Some(location)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use vp_diagnostics::{Location, RuleId, RuleKind};
    use vp_engine::ValidatorOutcome;

    fn sample_result() -> ValidationResult {
        ValidationResult {
            report: Report::from_diagnostics(vec![
                Diagnostic::new(
                    Severity::Error,
                    RuleId::crossref(RuleKind::BrokenLink),
                    Category::CrossReference,
                    "broken link to missing doc",
                )
                .with_file("docs/README.md")
                .with_location(Location::line_column(42, 1))
                .with_suggestion("Update the link target."),
                Diagnostic::new(
                    Severity::Warning,
                    RuleId::rfc(RuleKind::UnknownStatus),
                    Category::Registry,
                    "unknown RFC status",
                ),
            ]),
            validators: vec![
                ValidatorOutcome {
                    name: "rfc-registry".to_string(),
                    label: "RFC Registry".to_string(),
                    passed: false,
                },
                ValidatorOutcome {
                    name: "term-registry".to_string(),
                    label: "Terminology Registry".to_string(),
                    passed: true,
                },
                ValidatorOutcome {
                    name: "cross-reference".to_string(),
                    label: "Cross References".to_string(),
                    passed: false,
                },
            ],
        }
    }

    #[test]
    fn renders_progress_from_engine_outcomes() {
        let mut output = Vec::new();
        render_validator_progress(&sample_result().validators, &mut output).unwrap();
        let text = String::from_utf8(output).unwrap();

        assert!(text.contains("Running validators..."));
        assert!(text.contains("✓ Terminology Registry"));
        assert!(text.contains("✗ RFC Registry"));
        assert!(text.contains("✗ Cross References"));
    }

    #[test]
    fn renders_diagnostic_with_location_and_suggestion() {
        let diagnostic = Diagnostic::new(
            Severity::Error,
            RuleId::crossref(RuleKind::BrokenLink),
            Category::CrossReference,
            "broken link",
        )
        .with_file(PathBuf::from("docs/README.md"))
        .with_location(Location::line_column(42, 1))
        .with_suggestion("Fix the href.");

        let mut output = Vec::new();
        render_diagnostic(&diagnostic, &mut output).unwrap();
        let text = String::from_utf8(output).unwrap();

        assert!(text.contains("error[vp-crossref-broken-link]"));
        assert!(text.contains("  --> docs/README.md:42"));
        assert!(text.contains("broken link"));
        assert!(text.contains("Suggestion:"));
        assert!(text.contains("Fix the href."));
    }

    #[test]
    fn omits_location_when_file_missing() {
        let diagnostic = Diagnostic::new(
            Severity::Error,
            RuleId::rfc(RuleKind::RegistryMissing),
            Category::Registry,
            "registry missing",
        );

        assert!(format_location(&diagnostic).is_none());
    }

    #[test]
    fn groups_diagnostics_by_category_in_order() {
        let mut output = Vec::new();
        render_diagnostics(&sample_result().report, &mut output).unwrap();
        let text = String::from_utf8(output).unwrap();

        let registry_pos = text.find("Registry").expect("registry group");
        let crossref_pos = text.find("Cross References").expect("crossref group");
        assert!(registry_pos < crossref_pos);
        assert!(text.contains("unknown RFC status"));
        assert!(text.contains("broken link to missing doc"));
    }

    #[test]
    fn renders_validation_summary() {
        let mut output = Vec::new();
        render_summary(&sample_result(), &mut output).unwrap();
        let text = String::from_utf8(output).unwrap();

        assert!(text.contains(SUMMARY_RULE));
        assert!(text.contains("Validation Summary"));
        assert!(text.contains("Errors:   1"));
        assert!(text.contains("Warnings: 1"));
        assert!(text.contains("Info:     0"));
        assert!(text.contains("Validation failed."));
    }

    #[test]
    fn clean_run_shows_passed_summary() {
        let result = ValidationResult {
            report: Report::default(),
            validators: vec![ValidatorOutcome {
                name: "rfc-registry".to_string(),
                label: "RFC Registry".to_string(),
                passed: true,
            }],
        };

        let mut output = Vec::new();
        render_validation(&result, &mut output).unwrap();
        let text = String::from_utf8(output).unwrap();

        assert!(text.contains("✓ RFC Registry"));
        assert!(text.contains("Errors:   0"));
        assert!(text.contains("Validation passed."));
        assert!(!text.contains("error["));
        assert!(!text.contains("warning["));
        assert!(!text.contains("info["));
    }
}
