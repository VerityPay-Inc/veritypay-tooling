//! Structured findings for VerityPay specification tooling validators.

mod report;
mod types;

pub use report::Report;
pub use types::{Category, Diagnostic, Location, Severity};
