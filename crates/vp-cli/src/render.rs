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
        writeln!(out, "{mark} {}", validator.info.name)?;
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
    writeln!(out, "{}", diagnostic.rule.title())?;
    writeln!(out, "{}", diagnostic.rule.description())?;

    if let Some(location) = format_location(diagnostic) {
        writeln!(out)?;
        writeln!(out, "  --> {location}")?;
    }

    writeln!(out)?;
    writeln!(out, "{}", diagnostic.message)?;
    render_annotations(diagnostic, out)
}

fn render_annotations(diagnostic: &Diagnostic, out: &mut dyn Write) -> io::Result<()> {
    render_optional_annotation(out, "Suggestion", &diagnostic.suggestion)?;
    render_optional_annotation(out, "Help", &diagnostic.help)?;
    render_optional_annotation(out, "Note", &diagnostic.note)?;
    render_optional_annotation(out, "Related", &diagnostic.related)?;
    Ok(())
}

fn render_optional_annotation(
    out: &mut dyn Write,
    label: &str,
    value: &Option<String>,
) -> io::Result<()> {
    if let Some(text) = value {
        writeln!(out)?;
        writeln!(out, "{label}:")?;
        writeln!(out, "{text}")?;
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

/// Render only the validation summary counts (quiet mode).
pub fn render_quiet_summary(result: &ValidationResult, out: &mut dyn Write) -> io::Result<()> {
    let report = &result.report;

    writeln!(out, "Validation Summary")?;
    writeln!(out)?;
    writeln!(out, "Errors: {}", report.error_count)?;
    writeln!(out, "Warnings: {}", report.warning_count)?;
    writeln!(out, "Info: {}", report.info_count)?;

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
    use vp_core::ValidatorInfo;
    use vp_diagnostics::{Location, RuleId, RuleKind};
    use vp_engine::ValidatorOutcome;

    fn rfc_registry_info() -> ValidatorInfo {
        ValidatorInfo {
            id: "registry-rfc",
            name: "RFC Registry",
            description: "Validates the VP-RFC registry structure and references.",
            category: Category::Registry,
        }
    }

    fn term_registry_info() -> ValidatorInfo {
        ValidatorInfo {
            id: "registry-term",
            name: "Terminology Registry",
            description: "Validates the VP-TERM registry structure and references.",
            category: Category::Registry,
        }
    }

    fn crossref_info() -> ValidatorInfo {
        ValidatorInfo {
            id: "crossref",
            name: "Cross References",
            description: "Validates links, anchors, and registry references.",
            category: Category::CrossReference,
        }
    }

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
                    info: rfc_registry_info(),
                    passed: false,
                },
                ValidatorOutcome {
                    info: term_registry_info(),
                    passed: true,
                },
                ValidatorOutcome {
                    info: crossref_info(),
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
        assert!(text.contains("Broken Link"));
        assert!(text.contains("  --> docs/README.md:42"));
        assert!(text.contains("broken link"));
        assert!(text.contains("Suggestion:"));
        assert!(text.contains("Fix the href."));
    }

    #[test]
    fn renders_rule_title_for_rfc_version() {
        let diagnostic = Diagnostic::new(
            Severity::Error,
            RuleId::rfc(RuleKind::InvalidVersion),
            Category::Registry,
            "version `not-semver` is not valid semver",
        );

        let mut output = Vec::new();
        render_diagnostic(&diagnostic, &mut output).unwrap();
        let text = String::from_utf8(output).unwrap();

        assert!(text.contains("error[vp-rfc-invalid-version]"));
        assert!(text.contains("Invalid RFC Version"));
    }

    #[test]
    fn renders_optional_annotations_when_present() {
        let diagnostic = Diagnostic::new(
            Severity::Error,
            RuleId::crossref(RuleKind::BrokenLink),
            Category::CrossReference,
            "broken link",
        )
        .with_suggestion("Fix the href.")
        .with_help("See VALIDATION_OUTPUT.md")
        .with_note("This link is in the introduction.")
        .with_related("vp-crossref-broken-anchor");

        let mut output = Vec::new();
        render_diagnostic(&diagnostic, &mut output).unwrap();
        let text = String::from_utf8(output).unwrap();

        assert!(text.contains("Suggestion:\nFix the href."));
        assert!(text.contains("Help:\nSee VALIDATION_OUTPUT.md"));
        assert!(text.contains("Note:\nThis link is in the introduction."));
        assert!(text.contains("Related:\nvp-crossref-broken-anchor"));
    }

    #[test]
    fn omits_unused_annotations() {
        let diagnostic = Diagnostic::new(
            Severity::Error,
            RuleId::crossref(RuleKind::BrokenLink),
            Category::CrossReference,
            "broken link",
        );

        let mut output = Vec::new();
        render_diagnostic(&diagnostic, &mut output).unwrap();
        let text = String::from_utf8(output).unwrap();

        assert!(!text.contains("Suggestion:"));
        assert!(!text.contains("Help:"));
        assert!(!text.contains("Note:"));
        assert!(!text.contains("Related:"));
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
    fn renders_quiet_summary_only() {
        let mut output = Vec::new();
        render_quiet_summary(&sample_result(), &mut output).unwrap();
        let text = String::from_utf8(output).unwrap();

        assert_eq!(
            text,
            "Validation Summary\n\nErrors: 1\nWarnings: 1\nInfo: 0\n"
        );
        assert!(!text.contains(SUMMARY_RULE));
        assert!(!text.contains("Validation failed."));
    }

    #[test]
    fn clean_run_shows_passed_summary() {
        let result = ValidationResult {
            report: Report::default(),
            validators: vec![ValidatorOutcome {
                info: rfc_registry_info(),
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
