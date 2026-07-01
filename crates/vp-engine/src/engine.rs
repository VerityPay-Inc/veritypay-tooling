//! Engine lifecycle: orchestration and aggregation.

use vp_core::{ValidationContext, Validator};
use vp_diagnostics::{Report, Severity};

use crate::result::{ValidationResult, ValidatorOutcome};

/// Run all registered validators against `ctx` and aggregate diagnostics.
///
/// The engine does not inspect validator-specific rule logic (ADR-0003).
pub fn run_validation(ctx: &ValidationContext, validators: &[&dyn Validator]) -> ValidationResult {
    let mut diagnostics = Vec::new();
    let mut outcomes = Vec::with_capacity(validators.len());

    for validator in validators {
        let findings = validator.validate(ctx);
        let passed = !findings
            .iter()
            .any(|diagnostic| diagnostic.severity == Severity::Error);
        outcomes.push(ValidatorOutcome {
            name: validator.name().to_string(),
            label: validator.label().to_string(),
            passed,
        });
        diagnostics.extend(findings);
    }

    ValidationResult {
        report: Report::from_diagnostics(diagnostics),
        validators: outcomes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vp_diagnostics::{Category, Diagnostic, RuleId, RuleKind, Severity};

    struct FakeValidator {
        name: &'static str,
        label: &'static str,
        category: Category,
        findings: Vec<Diagnostic>,
    }

    impl Validator for FakeValidator {
        fn name(&self) -> &str {
            self.name
        }

        fn label(&self) -> &str {
            self.label
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
            label: "First",
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
            label: "Second",
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
        let result = run_validation(&ctx, &validators);

        assert_eq!(result.report.diagnostics.len(), 3);
        assert_eq!(result.report.error_count, 1);
        assert_eq!(result.report.warning_count, 1);
        assert_eq!(result.report.info_count, 1);
        assert!(result.has_errors());
        assert_eq!(result.validators.len(), 2);
        assert!(result.validators[0].passed);
        assert!(!result.validators[1].passed);
        assert_eq!(result.validators[0].label, "First");
    }

    #[test]
    fn empty_validator_list_produces_clean_report() {
        let ctx = ValidationContext::new(".");
        let result = run_validation(&ctx, &[]);

        assert!(result.report.diagnostics.is_empty());
        assert_eq!(result.report.error_count, 0);
        assert_eq!(result.report.warning_count, 0);
        assert_eq!(result.report.info_count, 0);
        assert!(!result.has_errors());
        assert!(result.validators.is_empty());
    }
}
