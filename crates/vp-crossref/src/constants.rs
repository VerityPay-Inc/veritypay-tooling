//! Shared constants for cross-reference discovery and validation.

pub const TERM_REGISTRY_PATH: &str = "spec/terminology/registry.yaml";
pub const RFC_REGISTRY_PATH: &str = "spec/rfcs/registry.yaml";

pub const ROOT_MARKDOWN_FILES: &[&str] = &["README.md", "CONTRIBUTING.md", "SPECIFICATION_STATUS.md"];

pub const CORPUS_DIRECTORIES: &[&str] = &["docs", "rfcs"];

pub const SKIP_DIRECTORY_NAMES: &[&str] = &["target", "crates", "node_modules", ".git"];

/// Non-live template/snippet trees excluded from cross-reference corpus scans.
pub const EXCLUDED_CORPUS_PREFIXES: &[&str] = &["docs/templates", "docs/snippets", "rfcs/templates"];

pub const SECTION_ID_PREFIXES: &[&str] = &["DM", "IM", "BM", "DAT", "SM", "CM", "GV", "VI", "GL"];

/// Documents that cite illustrative RFC numbers for format examples (VP-RFC-0000).
pub const RFC_ILLUSTRATIVE_DOCUMENTS: &[&str] = &["rfcs/0000-rfc-process.md"];
