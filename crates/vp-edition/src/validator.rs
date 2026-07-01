//! Edition Manifest validator entrypoint.

use vp_core::{ValidationContext, Validator, ValidatorInfo};
use vp_diagnostics::Category;

use crate::edition;

const INFO: ValidatorInfo = ValidatorInfo {
    id: "edition",
    name: "Edition Manifest",
    description: "Validates Edition Manifest structure, pins, and registry references.",
    category: Category::Edition,
};

/// Validates the Edition Manifest when configured on the validation context.
pub struct EditionValidator;

impl EditionValidator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for EditionValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl Validator for EditionValidator {
    fn info(&self) -> ValidatorInfo {
        INFO
    }

    fn validate(&self, ctx: &ValidationContext) -> Vec<vp_diagnostics::Diagnostic> {
        edition::validate(ctx)
    }
}
