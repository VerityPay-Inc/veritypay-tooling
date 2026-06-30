//! Severity, category, and diagnostic types.

use std::path::PathBuf;

use crate::rule_id::RuleId;

/// Finding severity — maps to CI exit policy via the engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// Validation category for filtering and reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Category {
    Registry,
    Metadata,
    CrossReference,
    Edition,
    Documentation,
    Future,
}

/// Optional location within a file (line, column, or YAML path).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Location {
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub path: Option<String>,
}

impl Location {
    pub fn line_column(line: u32, column: u32) -> Self {
        Self {
            line: Some(line),
            column: Some(column),
            path: None,
        }
    }

    pub fn yaml_path(path: impl Into<String>) -> Self {
        Self {
            line: None,
            column: None,
            path: Some(path.into()),
        }
    }
}

/// A single validation finding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub rule: RuleId,
    pub category: Category,
    pub file: Option<PathBuf>,
    pub location: Option<Location>,
    pub message: String,
    pub suggestion: Option<String>,
}

impl Diagnostic {
    pub fn new(
        severity: Severity,
        rule: RuleId,
        category: Category,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            rule,
            category,
            file: None,
            location: None,
            message: message.into(),
            suggestion: None,
        }
    }

    /// Stable external rule id for CLI and CI output.
    pub fn rule_id(&self) -> &'static str {
        self.rule.external_id()
    }

    pub fn with_file(mut self, file: impl Into<PathBuf>) -> Self {
        self.file = Some(file.into());
        self
    }

    pub fn with_location(mut self, location: Location) -> Self {
        self.location = Some(location);
        self
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}
