//! Terminology registry validator — VP-TERM registry (Milestone B.2).

use vp_core::{ValidationContext, Validator, ValidatorInfo};
use vp_diagnostics::Category;

use crate::term_registry;

const INFO: ValidatorInfo = ValidatorInfo {
    id: "registry-term",
    name: "Terminology Registry",
    description: "Validates the VP-TERM registry structure and references.",
    category: Category::Registry,
};

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
    fn info(&self) -> ValidatorInfo {
        INFO
    }

    fn validate(&self, ctx: &ValidationContext) -> Vec<vp_diagnostics::Diagnostic> {
        term_registry::validate(ctx)
    }
}
