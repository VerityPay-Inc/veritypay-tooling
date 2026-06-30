//! Reference classification for discovered citations and links.

/// Kind of reference discovered in specification documents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReferenceKind {
    Terminology,
    Rfc,
    MarkdownFile,
    MarkdownAnchor,
    ArchitectureSection,
    Future,
}
