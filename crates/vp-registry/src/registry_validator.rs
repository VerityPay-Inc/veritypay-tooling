//! Registry validator — VP-RFC registry (Milestone B.1).

use vp_core::{ValidationContext, Validator};
use vp_diagnostics::Category;

use crate::rfc_registry;

/// Validates `spec/rfcs/registry.yaml`.
pub struct RfcRegistryValidator;

impl RfcRegistryValidator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RfcRegistryValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl Validator for RfcRegistryValidator {
    fn name(&self) -> &str {
        "rfc-registry"
    }

    fn category(&self) -> Category {
        Category::Registry
    }

    fn validate(&self, ctx: &ValidationContext) -> Vec<vp_diagnostics::Diagnostic> {
        rfc_registry::validate(ctx)
    }
}

/// Backward-compatible alias for [`RfcRegistryValidator`].
pub type RegistryValidator = RfcRegistryValidator;
