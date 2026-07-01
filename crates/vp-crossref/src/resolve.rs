//! Path and anchor resolution helpers.

use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};
use std::sync::OnceLock;

use regex::Regex;

/// Split a Markdown link target into path and optional anchor fragment.
pub fn split_link_target(target: &str) -> (String, Option<String>) {
    if let Some((path, anchor)) = target.split_once('#') {
        (
            path.to_string(),
            Some(anchor.to_string()).filter(|a| !a.is_empty()),
        )
    } else {
        (target.to_string(), None)
    }
}

/// Resolve a relative link from `source_file` to a path relative to spec root.
pub fn resolve_relative_link(source_file: &Path, path_part: &str) -> PathBuf {
    if path_part.is_empty() {
        return source_file.to_path_buf();
    }

    let base = source_file
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));

    normalize_path(&base.join(path_part))
}

pub fn normalize_path(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                result.pop();
            }
            Component::CurDir => {}
            Component::Normal(part) => result.push(part),
            Component::RootDir => result.push(component.as_os_str()),
            Component::Prefix(_) => result.push(component.as_os_str()),
        }
    }
    result
}

/// Extract heading slugs and explicit HTML `id` anchors from Markdown content.
pub fn extract_document_anchors(content: &str) -> HashSet<String> {
    let mut anchors = extract_heading_anchors(content);
    anchors.extend(extract_html_id_anchors(content));
    anchors
}

/// Extract GitHub-style heading anchors from Markdown content.
pub fn extract_heading_anchors(content: &str) -> HashSet<String> {
    let mut anchors = HashSet::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('#') {
            continue;
        }
        let mut chars = trimmed.chars();
        while chars.next().is_some_and(|c| c == '#') {}
        let title = chars.as_str().trim();
        if !title.is_empty() {
            anchors.insert(slugify_heading(title));
        }
    }
    anchors
}

fn extract_html_id_anchors(content: &str) -> HashSet<String> {
    static HTML_ID: OnceLock<Regex> = OnceLock::new();
    let re = HTML_ID.get_or_init(|| Regex::new(r#"(?i)<a\s+[^>]*\bid="([^"]+)""#).expect("html id"));

    re.captures_iter(content)
        .filter_map(|caps| caps.get(1).map(|m| m.as_str().to_string()))
        .collect()
}

fn slugify_heading(text: &str) -> String {
    let mut slug = String::new();
    let mut last_hyphen = false;
    for c in text.trim().to_lowercase().chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c);
            last_hyphen = false;
        } else if (c.is_whitespace() || c == '-' || c == '_') && !last_hyphen && !slug.is_empty() {
            slug.push('-');
            last_hyphen = true;
        }
    }
    slug.trim_end_matches('-').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn resolves_parent_relative_links() {
        let resolved = resolve_relative_link(Path::new("docs/a/page.md"), "../other.md");
        assert_eq!(resolved, PathBuf::from("docs/other.md"));
    }

    #[test]
    fn slugifies_headings() {
        let content = "# Domain Overview\n## Sub Section";
        let anchors = extract_heading_anchors(content);
        assert!(anchors.contains("domain-overview"));
        assert!(anchors.contains("sub-section"));
    }

    #[test]
    fn extracts_html_id_anchors() {
        let content = r#"<a id="dm-4-8"></a>
<a id="principle-01"></a>
"#;
        let anchors = extract_document_anchors(content);
        assert!(anchors.contains("dm-4-8"));
        assert!(anchors.contains("principle-01"));
    }
}
