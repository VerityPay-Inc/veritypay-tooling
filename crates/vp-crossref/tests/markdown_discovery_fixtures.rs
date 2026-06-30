//! Fixture-based discovery tests (no validation).

use std::fs;
use std::path::PathBuf;

use vp_crossref::{MarkdownDiscovery, ReferenceDiscovery, ReferenceKind};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

#[test]
fn fixture_discovers_all_reference_kinds() {
    let path = fixtures_dir().join("mixed_references.md");
    let content = fs::read_to_string(&path).expect("read fixture");
    let discovery = MarkdownDiscovery::new();
    let refs = discovery.discover(&path, &content);

    let kinds: Vec<_> = refs.iter().map(|r| r.kind).collect();
    assert!(kinds.contains(&ReferenceKind::Terminology));
    assert!(kinds.contains(&ReferenceKind::Rfc));
    assert!(kinds.contains(&ReferenceKind::ArchitectureSection));
    assert!(kinds.contains(&ReferenceKind::MarkdownFile));
    assert!(kinds.contains(&ReferenceKind::MarkdownAnchor));

    assert!(refs.iter().any(|r| r.target == "VP-TERM-001"));
    assert!(refs.iter().any(|r| r.target == "VP-RFC-0000"));
    assert!(refs.iter().any(|r| r.target == "DM-4.8"));
    assert!(refs.iter().any(|r| r.target.contains("DOMAIN_MODEL.md")));
    assert!(refs.iter().any(|r| r.target.contains("#domain-overview")));

    assert!(!refs.iter().any(|r| r.target.starts_with("https://")));
}

#[test]
fn fixture_records_source_file() {
    let path = fixtures_dir().join("mixed_references.md");
    let content = fs::read_to_string(&path).expect("read fixture");
    let discovery = MarkdownDiscovery::new();
    let refs = discovery.discover(&path, &content);

    assert!(refs.iter().all(|r| r.source_file == path));
}
