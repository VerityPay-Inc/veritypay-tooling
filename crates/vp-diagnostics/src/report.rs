//! Aggregated validation report.

use crate::{Category, Diagnostic, Severity};

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

fn category_rank(category: Category) -> u8 {
    match category {
        Category::Registry => 0,
        Category::CrossReference => 1,
        Category::Metadata => 2,
        Category::Edition => 3,
        Category::Documentation => 4,
        Category::Future => 5,
    }
}

fn line_key(diagnostic: &Diagnostic) -> u32 {
    diagnostic
        .location
        .as_ref()
        .and_then(|location| location.line)
        .unwrap_or(u32::MAX)
}

fn sort_diagnostics(diagnostics: &mut [Diagnostic]) {
    diagnostics.sort_by(|a, b| {
        category_rank(a.category)
            .cmp(&category_rank(b.category))
            .then_with(|| compare_file(a, b))
            .then_with(|| line_key(a).cmp(&line_key(b)))
            .then_with(|| a.rule_id().cmp(b.rule_id()))
            .then_with(|| a.message.cmp(&b.message))
    });
}

fn compare_file(a: &Diagnostic, b: &Diagnostic) -> std::cmp::Ordering {
    match (&a.file, &b.file) {
        (None, None) => std::cmp::Ordering::Equal,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (Some(_), None) => std::cmp::Ordering::Less,
        (Some(left), Some(right)) => left.as_os_str().cmp(right.as_os_str()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Location, RuleId, RuleKind};

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
    fn stable_sort_by_category_file_line_and_rule_id() {
        let report = Report::from_diagnostics(vec![
            Diagnostic::new(
                Severity::Error,
                RuleId::crossref(RuleKind::BrokenLink),
                Category::CrossReference,
                "crossref",
            )
            .with_file("z.md")
            .with_location(Location::line_column(10, 1)),
            Diagnostic::new(
                Severity::Error,
                RuleId::rfc(RuleKind::DuplicateId),
                Category::Registry,
                "registry second line",
            )
            .with_file("a.md")
            .with_location(Location::line_column(20, 1)),
            Diagnostic::new(
                Severity::Error,
                RuleId::rfc(RuleKind::RegistryMissing),
                Category::Registry,
                "registry first line",
            )
            .with_file("a.md")
            .with_location(Location::line_column(5, 1)),
            Diagnostic::new(
                Severity::Error,
                RuleId::rfc(RuleKind::InvalidVersion),
                Category::Registry,
                "registry same line earlier rule",
            )
            .with_file("a.md")
            .with_location(Location::line_column(5, 1)),
        ]);

        assert_eq!(report.diagnostics.len(), 4);
        assert_eq!(report.diagnostics[0].message, "registry same line earlier rule");
        assert_eq!(report.diagnostics[1].message, "registry first line");
        assert_eq!(report.diagnostics[2].message, "registry second line");
        assert_eq!(report.diagnostics[3].message, "crossref");
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
