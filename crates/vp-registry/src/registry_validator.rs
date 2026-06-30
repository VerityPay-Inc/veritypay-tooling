//! Registry validator — VP-RFC registry (Milestone B.1).

use vp_core::{ValidationContext, Validator};
use vp_diagnostics::Category;

use crate::rfc_registry;

/// Validates machine-readable registries under `veritypay-spec`.
pub struct RegistryValidator;

impl RegistryValidator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RegistryValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl Validator for RegistryValidator {
    fn name(&self) -> &str {
        "registry"
    }

    fn category(&self) -> Category {
        Category::Registry
    }

    fn validate(&self, ctx: &ValidationContext) -> Vec<vp_diagnostics::Diagnostic> {
        rfc_registry::validate(ctx)
    }
}
