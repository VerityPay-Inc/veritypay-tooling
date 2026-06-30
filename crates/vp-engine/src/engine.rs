//! Engine lifecycle: orchestration and aggregation.

use vp_core::{ValidationContext, Validator};
use vp_diagnostics::Report;

/// Run all registered validators against `ctx` and aggregate diagnostics.
///
/// The engine does not inspect validator-specific rule logic (ADR-0003).
pub fn run_validation(ctx: &ValidationContext, validators: &[&dyn Validator]) -> Report {
    let mut diagnostics = Vec::new();

    for validator in validators {
        diagnostics.extend(validator.validate(ctx));
    }

    Report::from_diagnostics(diagnostics)
}

#[cfg(test)]
mod tests {
    use super::*;
    use vp_diagnostics::{Category, Diagnostic, RuleId, RuleKind, Severity};

    struct FakeValidator {
        name: &'static str,
        category: Category,
        findings: Vec<Diagnostic>,
    }

    impl Validator for FakeValidator {
        fn name(&self) -> &str {
            self.name
        }

        fn category(&self) -> Category {
            self.category
        }

        fn validate(&self, _ctx: &ValidationContext) -> Vec<Diagnostic> {
            self.findings.clone()
        }
    }

    #[test]
    fn aggregates_diagnostics_from_multiple_validators() {
        let ctx = ValidationContext::new(".");
        let first = FakeValidator {
            name: "first",
            category: Category::Registry,
            findings: vec![Diagnostic::new(
                Severity::Warning,
                RuleId::rfc(RuleKind::UnknownStatus),
                Category::Registry,
                "first warning",
            )],
        };
        let second = FakeValidator {
            name: "second",
            category: Category::Future,
            findings: vec![
                Diagnostic::new(
                    Severity::Error,
                    RuleId::term(RuleKind::UnknownReference),
                    Category::Future,
                    "second error",
                ),
                Diagnostic::new(
                    Severity::Info,
                    RuleId::rfc(RuleKind::RegistryMissing),
                    Category::Future,
                    "second info",
                ),
            ],
        };

        let validators: [&dyn Validator; 2] = [&first, &second];
        let report = run_validation(&ctx, &validators);

        assert_eq!(report.diagnostics.len(), 3);
        assert_eq!(report.error_count, 1);
        assert_eq!(report.warning_count, 1);
        assert_eq!(report.info_count, 1);
        assert!(report.has_errors());
    }

    #[test]
    fn empty_validator_list_produces_clean_report() {
        let ctx = ValidationContext::new(".");
        let report = run_validation(&ctx, &[]);

        assert!(report.diagnostics.is_empty());
        assert_eq!(report.error_count, 0);
        assert_eq!(report.warning_count, 0);
        assert_eq!(report.info_count, 0);
        assert!(!report.has_errors());
    }
}
