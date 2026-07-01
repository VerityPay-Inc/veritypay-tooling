//! Cross-reference validator — Milestone C.2.

use vp_core::{ValidationContext, Validator, ValidatorInfo};
use vp_diagnostics::Category;

use crate::validate;

const INFO: ValidatorInfo = ValidatorInfo {
    id: "crossref",
    name: "Cross References",
    description: "Validates links, anchors, and registry references.",
    category: Category::CrossReference,
};

/// Validates cross-references in the specification Markdown corpus.
pub struct CrossReferenceValidator;

impl CrossReferenceValidator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CrossReferenceValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl Validator for CrossReferenceValidator {
    fn info(&self) -> ValidatorInfo {
        INFO
    }

    fn validate(&self, ctx: &ValidationContext) -> Vec<vp_diagnostics::Diagnostic> {
        validate::validate(ctx)
    }
}
