//! Markdown reference discovery.

use std::path::Path;

use regex::Regex;
use vp_diagnostics::Location;

use super::constants::SECTION_ID_PREFIXES;
use super::discovery::ReferenceDiscovery;
use super::kind::ReferenceKind;
use super::reference::DiscoveredReference;

/// Discovers VP-TERM, VP-RFC, architecture section IDs, and Markdown links in text.
#[derive(Debug, Default)]
pub struct MarkdownDiscovery;

impl ReferenceDiscovery for MarkdownDiscovery {
    fn discover(&self, source_file: &Path, content: &str) -> Vec<DiscoveredReference> {
        let mut references = Vec::new();
        references.extend(discover_terminology(source_file, content));
        references.extend(discover_rfc(source_file, content));
        references.extend(discover_architecture_sections(source_file, content));
        references.extend(discover_markdown_links(source_file, content));
        references.sort_by(|a, b| {
            a.location
                .line
                .cmp(&b.location.line)
                .then_with(|| a.location.column.cmp(&b.location.column))
                .then_with(|| a.target.cmp(&b.target))
        });
        references
    }
}

impl MarkdownDiscovery {
    pub fn new() -> Self {
        Self
    }
}

fn discover_terminology(source_file: &Path, content: &str) -> Vec<DiscoveredReference> {
    static PATTERN: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let re = PATTERN.get_or_init(|| Regex::new(r"VP-TERM-\d{3,4}").expect("VP-TERM pattern"));

    re.find_iter(content)
        .map(|m| {
            DiscoveredReference::new(
                ReferenceKind::Terminology,
                m.as_str(),
                source_file,
                location_at(content, m.start()),
            )
        })
        .collect()
}

fn discover_rfc(source_file: &Path, content: &str) -> Vec<DiscoveredReference> {
    static PATTERN: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let re = PATTERN.get_or_init(|| Regex::new(r"VP-RFC-\d{4}").expect("VP-RFC pattern"));

    re.find_iter(content)
        .map(|m| {
            DiscoveredReference::new(
                ReferenceKind::Rfc,
                m.as_str(),
                source_file,
                location_at(content, m.start()),
            )
        })
        .collect()
}

fn discover_architecture_sections(source_file: &Path, content: &str) -> Vec<DiscoveredReference> {
    static PATTERN: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let prefix = SECTION_ID_PREFIXES.join("|");
    let re = PATTERN.get_or_init(|| {
        Regex::new(&format!(r"(?:{prefix})-\d+(?:\.\d+)*")).expect("section id pattern")
    });

    re.find_iter(content)
        .map(|m| {
            DiscoveredReference::new(
                ReferenceKind::ArchitectureSection,
                m.as_str(),
                source_file,
                location_at(content, m.start()),
            )
        })
        .collect()
}

fn discover_markdown_links(source_file: &Path, content: &str) -> Vec<DiscoveredReference> {
    static PATTERN: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let re = PATTERN.get_or_init(|| Regex::new(r"\[[^\]]*\]\(([^)]+)\)").expect("markdown link"));

    re.captures_iter(content)
        .filter_map(|caps| {
            let full = caps.get(0)?;
            let target = caps.get(1)?.as_str().trim();
            if target.is_empty() || is_external_link(target) {
                return None;
            }

            let kind = if target.contains('#') {
                ReferenceKind::MarkdownAnchor
            } else {
                ReferenceKind::MarkdownFile
            };

            Some(DiscoveredReference::new(
                kind,
                target,
                source_file,
                location_at(content, full.start()),
            ))
        })
        .collect()
}

fn is_external_link(target: &str) -> bool {
    target.starts_with("http://") || target.starts_with("https://") || target.starts_with("mailto:")
}

fn location_at(content: &str, byte_offset: usize) -> Location {
    let before = &content[..byte_offset];
    let line = before.matches('\n').count() as u32 + 1;
    let line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
    let column = content[line_start..byte_offset].chars().count() as u32 + 1;
    Location::line_column(line, column)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn discover(content: &str) -> Vec<DiscoveredReference> {
        MarkdownDiscovery.discover(Path::new("doc.md"), content)
    }

    fn targets(content: &str) -> Vec<String> {
        discover(content)
            .into_iter()
            .map(|reference| reference.target)
            .collect()
    }

    #[test]
    fn discovers_terminology_and_rfc_ids() {
        let refs = discover("See VP-TERM-001 and VP-RFC-0000 for context.");
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].kind, ReferenceKind::Terminology);
        assert_eq!(refs[0].target, "VP-TERM-001");
        assert_eq!(refs[1].kind, ReferenceKind::Rfc);
        assert_eq!(refs[1].target, "VP-RFC-0000");
    }

    #[test]
    fn discovers_architecture_section_ids() {
        let refs = discover("Defined in DM-1.1 and IM-2.3.4.");
        assert_eq!(refs.len(), 2);
        assert!(refs
            .iter()
            .all(|reference| reference.kind == ReferenceKind::ArchitectureSection));
        assert_eq!(targets("Defined in DM-1.1."), vec!["DM-1.1".to_string()]);
    }

    #[test]
    fn discovers_markdown_file_and_anchor_links() {
        let content = "Read [model](../docs/MODEL.md) and [section](MODEL.md#overview).";
        let refs = discover(content);
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].kind, ReferenceKind::MarkdownFile);
        assert_eq!(refs[0].target, "../docs/MODEL.md");
        assert_eq!(refs[1].kind, ReferenceKind::MarkdownAnchor);
        assert_eq!(refs[1].target, "MODEL.md#overview");
    }

    #[test]
    fn skips_external_links() {
        let refs = discover("[site](https://example.com) [doc](local.md)");
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].target, "local.md");
    }

    #[test]
    fn records_line_and_column() {
        let content = "line one\nline two VP-TERM-002";
        let refs = discover(content);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].location.line, Some(2));
        assert_eq!(refs[0].location.column, Some(10));
    }
}
