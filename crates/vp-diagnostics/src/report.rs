//! Aggregated validation report.

use crate::{Diagnostic, Severity};

/// Aggregated diagnostics from one or more validators.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Report {
    pub diagnostics: Vec<Diagnostic>,
    pub error_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
}

impl Report {
    pub fn from_diagnostics(mut diagnostics: Vec<Diagnostic>) -> Self {
        sort_diagnostics(&mut diagnostics);

        let mut error_count = 0;
        let mut warning_count = 0;
        let mut info_count = 0;

        for diagnostic in &diagnostics {
            match diagnostic.severity {
                Severity::Error => error_count += 1,
                Severity::Warning => warning_count += 1,
                Severity::Info => info_count += 1,
            }
        }

        Self {
            diagnostics,
            error_count,
            warning_count,
            info_count,
        }
    }

    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }
}

fn sort_diagnostics(diagnostics: &mut [Diagnostic]) {
    diagnostics.sort_by(|a, b| {
        a.file
            .as_ref()
            .map(|p| p.as_os_str())
            .cmp(&b.file.as_ref().map(|p| p.as_os_str()))
            .then_with(|| a.rule_id().cmp(b.rule_id()))
            .then_with(|| a.message.cmp(&b.message))
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Category, Location, RuleId, RuleKind};

    #[test]
    fn counts_by_severity() {
        let report = Report::from_diagnostics(vec![
            Diagnostic::new(
                Severity::Error,
                RuleId::rfc(RuleKind::DuplicateId),
                Category::Registry,
                "error",
            ),
            Diagnostic::new(
                Severity::Warning,
                RuleId::rfc(RuleKind::UnknownStatus),
                Category::Registry,
                "warn",
            ),
            Diagnostic::new(
                Severity::Info,
                RuleId::term(RuleKind::InvalidSectionId),
                Category::Registry,
                "info",
            ),
            Diagnostic::new(
                Severity::Error,
                RuleId::term(RuleKind::UnknownReference),
                Category::Future,
                "error",
            ),
        ]);

        assert_eq!(report.error_count, 2);
        assert_eq!(report.warning_count, 1);
        assert_eq!(report.info_count, 1);
        assert!(report.has_errors());
    }

    #[test]
    fn no_errors_when_clean() {
        let report = Report::from_diagnostics(vec![Diagnostic::new(
            Severity::Warning,
            RuleId::rfc(RuleKind::MissingPath),
            Category::Documentation,
            "warn",
        )]);

        assert!(!report.has_errors());
    }

    #[test]
    fn stable_sort_by_file_and_rule_id() {
        let report = Report::from_diagnostics(vec![
            Diagnostic::new(
                Severity::Info,
                RuleId::rfc(RuleKind::DuplicateId),
                Category::Registry,
                "second",
            )
            .with_file("b.md"),
            Diagnostic::new(
                Severity::Info,
                RuleId::rfc(RuleKind::RegistryMissing),
                Category::Registry,
                "first",
            )
            .with_file("a.md"),
        ]);

        assert_eq!(
            report.diagnostics[0]
                .file
                .as_ref()
                .map(|p| p.to_string_lossy().into_owned()),
            Some("a.md".to_string())
        );
    }

    #[test]
    fn location_helpers() {
        let loc = Location::line_column(10, 5);
        assert_eq!(loc.line, Some(10));
        assert_eq!(loc.column, Some(5));

        let yaml = Location::yaml_path("terms[0].id");
        assert_eq!(yaml.path.as_deref(), Some("terms[0].id"));
    }
}
