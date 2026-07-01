//! Validation run result including per-validator outcomes.

use vp_core::ValidatorInfo;
use vp_diagnostics::Report;

/// Outcome of a single validator invocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidatorOutcome {
    pub info: ValidatorInfo,
    pub passed: bool,
}

/// Aggregated validation run output from the engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationResult {
    pub report: Report,
    pub validators: Vec<ValidatorOutcome>,
}

impl ValidationResult {
    pub fn has_errors(&self) -> bool {
        self.report.has_errors()
    }
}
