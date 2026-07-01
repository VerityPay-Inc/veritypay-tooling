//! Registry validator — VP-RFC registry (Milestone B.1).

use vp_core::{ValidationContext, Validator, ValidatorInfo};
use vp_diagnostics::Category;

use crate::rfc_registry;

const INFO: ValidatorInfo = ValidatorInfo {
    id: "registry-rfc",
    name: "RFC Registry",
    description: "Validates the VP-RFC registry structure and references.",
    category: Category::Registry,
};

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
    fn info(&self) -> ValidatorInfo {
        INFO
    }

    fn validate(&self, ctx: &ValidationContext) -> Vec<vp_diagnostics::Diagnostic> {
        rfc_registry::validate(ctx)
    }
}

/// Backward-compatible alias for [`RfcRegistryValidator`].
pub type RegistryValidator = RfcRegistryValidator;
