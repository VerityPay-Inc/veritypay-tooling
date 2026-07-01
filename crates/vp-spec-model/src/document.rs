//! Specification document types and parsing.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use regex::Regex;
use serde_yaml::Value;

use crate::error::BuildError;

/// One Markdown file in the specification document corpus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecificationDocument {
    pub relative_path: String,
    pub raw_text: String,
    pub front_matter: Option<DocumentFrontMatter>,
    pub sections: Vec<DocumentSection>,
}

/// Typed view of YAML front matter at the top of a specification document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentFrontMatter {
    pub fields: BTreeMap<String, Value>,
    pub version: Option<String>,
    pub status: Option<String>,
    pub title: Option<String>,
    pub start_line: u32,
    pub end_line: u32,
}

/// Structural unit within a document (heading or explicit HTML anchor).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentSection {
    pub level: u8,
    pub title: String,
    pub anchor: String,
    pub line: u32,
}

pub(crate) fn parse_document(
    relative_path: impl Into<String>,
    raw_text: String,
) -> Result<SpecificationDocument, BuildError> {
    let relative_path = relative_path.into();
    let front_matter = parse_front_matter(&relative_path, &raw_text)?;
    let sections = extract_sections(&raw_text);

    Ok(SpecificationDocument {
        relative_path,
        raw_text,
        front_matter,
        sections,
    })
}

fn parse_front_matter(
    path: &str,
    content: &str,
) -> Result<Option<DocumentFrontMatter>, BuildError> {
    let Some(rest) = content.strip_prefix("---") else {
        return Ok(None);
    };

    let Some(end_idx) = rest.find("\n---") else {
        return Ok(None);
    };

    let yaml_str = rest[..end_idx]
        .strip_prefix('\n')
        .unwrap_or(&rest[..end_idx]);
    let yaml: Value = serde_yaml::from_str(yaml_str)
        .map_err(|error| BuildError::yaml_invalid(path, error.to_string()))?;

    let closing_line_start = "---".len() + end_idx + 1;
    let end_line = line_number_at(content, closing_line_start);

    let mut fields = BTreeMap::new();
    if let Some(mapping) = yaml.as_mapping() {
        for (key, value) in mapping {
            if let Some(key) = key.as_str() {
                fields.insert(key.to_string(), value.clone());
            }
        }
    }

    Ok(Some(DocumentFrontMatter {
        version: string_field(&fields, "version"),
        status: string_field(&fields, "status"),
        title: string_field(&fields, "title"),
        fields,
        start_line: 1,
        end_line,
    }))
}

fn string_field(fields: &BTreeMap<String, Value>, key: &str) -> Option<String> {
    fields
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
}

fn extract_sections(content: &str) -> Vec<DocumentSection> {
    let mut sections = Vec::new();

    for (index, line) in content.lines().enumerate() {
        let line_number = index as u32 + 1;
        let trimmed = line.trim();

        if let Some(section) = parse_heading_section(trimmed, line_number) {
            sections.push(section);
        }

        sections.extend(parse_html_anchor_sections(trimmed, line_number));
    }

    sections
}

fn parse_heading_section(trimmed: &str, line: u32) -> Option<DocumentSection> {
    if !trimmed.starts_with('#') {
        return None;
    }

    let mut level = 0u8;
    for ch in trimmed.chars() {
        if ch == '#' {
            level += 1;
        } else {
            break;
        }
    }

    if level == 0 || level > 6 {
        return None;
    }

    let title = trimmed[level as usize..].trim();
    if title.is_empty() {
        return None;
    }

    Some(DocumentSection {
        level,
        title: title.to_string(),
        anchor: slugify_heading(title),
        line,
    })
}

fn parse_html_anchor_sections(trimmed: &str, line: u32) -> Vec<DocumentSection> {
    static HTML_ID: OnceLock<Regex> = OnceLock::new();
    let re = HTML_ID.get_or_init(|| Regex::new(r#"(?i)<a\s+[^>]*\bid="([^"]+)""#).expect("html id"));

    re.captures_iter(trimmed)
        .filter_map(|caps| {
            let anchor = caps.get(1)?.as_str().to_string();
            Some(DocumentSection {
                level: 0,
                title: String::new(),
                anchor,
                line,
            })
        })
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

fn line_number_at(content: &str, byte_offset: usize) -> u32 {
    content[..byte_offset.min(content.len())]
        .matches('\n')
        .count() as u32
        + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_front_matter_fields() {
        let content = "---\ntitle: Example\nversion: 0.2.0\nstatus: draft\n---\n# Body\n";
        let doc = parse_document("docs/example.md", content.to_string()).expect("parse");
        let front_matter = doc.front_matter.expect("front matter");
        assert_eq!(front_matter.title.as_deref(), Some("Example"));
        assert_eq!(front_matter.version.as_deref(), Some("0.2.0"));
        assert_eq!(front_matter.status.as_deref(), Some("draft"));
        assert_eq!(front_matter.start_line, 1);
        assert!(front_matter.end_line >= 4);
    }

    #[test]
    fn extracts_heading_and_html_sections() {
        let content = "# Domain Overview\n\n<a id=\"dm-4-8\"></a>\n\n## Sub Section\n";
        let doc = parse_document("docs/example.md", content.to_string()).expect("parse");
        assert!(doc.sections.iter().any(|section| {
            section.level == 1
                && section.title == "Domain Overview"
                && section.anchor == "domain-overview"
        }));
        assert!(doc.sections.iter().any(|section| {
            section.level == 0 && section.anchor == "dm-4-8"
        }));
        assert!(doc.sections.iter().any(|section| {
            section.level == 2
                && section.title == "Sub Section"
                && section.anchor == "sub-section"
        }));
    }
}
