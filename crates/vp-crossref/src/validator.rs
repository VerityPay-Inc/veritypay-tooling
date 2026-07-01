//! Cross-reference validator — Milestone C.2.

use vp_core::{ValidationContext, Validator};
use vp_diagnostics::Category;

use crate::validate;

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
    fn name(&self) -> &str {
        "cross-reference"
    }

    fn label(&self) -> &str {
        "Cross References"
    }

    fn category(&self) -> Category {
        Category::CrossReference
    }

    fn validate(&self, ctx: &ValidationContext) -> Vec<vp_diagnostics::Diagnostic> {
        validate::validate(ctx)
    }
}
