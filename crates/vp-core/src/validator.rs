//! Validator contract between engine and validator crates (ADR-0002, ADR-0003).

use vp_diagnostics::Diagnostic;

use crate::context::ValidationContext;
use crate::validator_info::ValidatorInfo;

/// Plugin interface implemented by each validator crate.
pub trait Validator {
    /// Stable identity metadata for CLI progress, profiles, and introspection.
    fn info(&self) -> ValidatorInfo;

    fn validate(&self, ctx: &ValidationContext) -> Vec<Diagnostic>;
}
