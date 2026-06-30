//! Structured findings for VerityPay specification tooling validators.

mod report;
mod rule_id;
mod types;

pub use report::Report;
pub use rule_id::{RuleId, RuleKind, RuleScope};
pub use types::{Category, Diagnostic, Location, Severity};
