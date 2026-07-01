//! Validation engine orchestration (ADR-0003).

pub mod engine;
pub mod result;

pub use engine::run_validation;
pub use result::{ValidationResult, ValidatorOutcome};
