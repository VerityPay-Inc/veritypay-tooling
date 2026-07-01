//! Stable identity metadata for validators.

use vp_diagnostics::Category;

/// Documented identity for a validator plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidatorInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub category: Category,
}
