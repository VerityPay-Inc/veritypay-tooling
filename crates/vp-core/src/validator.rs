//! Validator contract between engine and validator crates (ADR-0002, ADR-0003).

use vp_diagnostics::{Category, Diagnostic};

use crate::context::ValidationContext;

/// Plugin interface implemented by each validator crate.
pub trait Validator {
    fn name(&self) -> &str;
    /// Human-readable label for CLI progress output.
    fn label(&self) -> &str;
    fn category(&self) -> Category;
    fn validate(&self, ctx: &ValidationContext) -> Vec<Diagnostic>;
}
