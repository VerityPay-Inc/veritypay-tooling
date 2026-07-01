//! Terminology registry validator — VP-TERM registry (Milestone B.2).

use vp_core::{ValidationContext, Validator};
use vp_diagnostics::Category;

use crate::term_registry;

/// Validates `spec/terminology/registry.yaml`.
pub struct TermRegistryValidator;

impl TermRegistryValidator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TermRegistryValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl Validator for TermRegistryValidator {
    fn name(&self) -> &str {
        "term-registry"
    }

    fn label(&self) -> &str {
        "Terminology Registry"
    }

    fn category(&self) -> Category {
        Category::Registry
    }

    fn validate(&self, ctx: &ValidationContext) -> Vec<vp_diagnostics::Diagnostic> {
        term_registry::validate(ctx)
    }
}
