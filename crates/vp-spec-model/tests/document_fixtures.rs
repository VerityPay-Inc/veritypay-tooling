//! Integration tests for document corpus loading.

use std::fs;
use std::path::{Path, PathBuf};

use vp_core::SpecRepository;
use vp_spec_model::SpecificationBuilder;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn install_document_fixture(root: &Path) {
    fs::create_dir_all(root.join("docs/live")).expect("docs/live");
    fs::write(
        root.join("docs/live/example.md"),
        "---\ntitle: Example Doc\nversion: 0.2.0\nstatus: draft\n---\n\
# Domain Overview\n\n<a id=\"dm-4-8\"></a>\n\n## Sub Section\n",
    )
    .expect("write example doc");
    fs::write(root.join("README.md"), "# Root\n").expect("write readme");
}

#[test]
fn loads_document_fixture() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_document_fixture(dir.path());

    let repo = SpecRepository::new(dir.path());
    let spec = SpecificationBuilder::new(&repo)
        .build_documents_only()
        .expect("build documents");

    assert_eq!(spec.document_corpus.documents().len(), 2);
    let doc = spec
        .document_corpus
        .get("docs/live/example.md")
        .expect("example doc");
    assert!(doc.raw_text.contains("Domain Overview"));
}

#[test]
fn extracts_front_matter_version_title_status() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_document_fixture(dir.path());

    let repo = SpecRepository::new(dir.path());
    let doc = SpecificationBuilder::new(&repo)
        .build_documents_only()
        .expect("build")
        .document_corpus
        .get("docs/live/example.md")
        .expect("example")
        .clone();

    let front_matter = doc.front_matter.expect("front matter");
    assert_eq!(front_matter.title.as_deref(), Some("Example Doc"));
    assert_eq!(front_matter.version.as_deref(), Some("0.2.0"));
    assert_eq!(front_matter.status.as_deref(), Some("draft"));
    assert!(front_matter.fields.contains_key("title"));
}

#[test]
fn extracts_markdown_heading_anchors() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_document_fixture(dir.path());

    let repo = SpecRepository::new(dir.path());
    let spec = SpecificationBuilder::new(&repo)
        .build_documents_only()
        .expect("build");
    let doc = spec
        .document_corpus
        .get("docs/live/example.md")
        .expect("example");

    assert!(doc.sections.iter().any(|section| {
        section.level == 1
            && section.anchor == "domain-overview"
            && section.title == "Domain Overview"
    }));
    assert!(doc.sections.iter().any(|section| {
        section.level == 2
            && section.anchor == "sub-section"
            && section.title == "Sub Section"
    }));
}

#[test]
fn extracts_html_id_anchors() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_document_fixture(dir.path());

    let repo = SpecRepository::new(dir.path());
    let spec = SpecificationBuilder::new(&repo)
        .build_documents_only()
        .expect("build");
    let doc = spec
        .document_corpus
        .get("docs/live/example.md")
        .expect("example");

    assert!(doc
        .sections
        .iter()
        .any(|section| section.level == 0 && section.anchor == "dm-4-8"));
}

#[test]
fn corpus_lookup_by_path_works() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_document_fixture(dir.path());

    let repo = SpecRepository::new(dir.path());
    let corpus = SpecificationBuilder::new(&repo)
        .build_documents_only()
        .expect("build")
        .document_corpus;

    assert!(corpus.get("docs/live/example.md").is_some());
    assert!(corpus.get("README.md").is_some());
    assert!(corpus.get("docs/missing.md").is_none());
}

#[test]
fn excluded_folders_are_skipped() {
    let dir = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(dir.path().join("docs/live")).expect("live");
    fs::create_dir_all(dir.path().join("docs/templates/draft")).expect("templates");
    fs::create_dir_all(dir.path().join("docs/snippets/draft")).expect("snippets");
    fs::create_dir_all(dir.path().join("docs/generated/hidden")).expect("generated");
    fs::create_dir_all(dir.path().join("rfcs/templates/draft")).expect("rfc templates");
    fs::write(dir.path().join("docs/live/page.md"), "# live\n").expect("live file");
    fs::write(
        dir.path().join("docs/templates/draft/x.md"),
        "# template\n",
    )
    .expect("template file");
    fs::write(dir.path().join("docs/snippets/draft/x.md"), "# snippet\n").expect("snippet");
    fs::write(
        dir.path().join("docs/generated/hidden/x.md"),
        "# generated\n",
    )
    .expect("generated file");
    fs::write(
        dir.path().join("rfcs/templates/draft/x.md"),
        "# rfc template\n",
    )
    .expect("rfc template file");

    let repo = SpecRepository::new(dir.path());
    let paths: Vec<_> = SpecificationBuilder::new(&repo)
        .build_documents_only()
        .expect("build")
        .document_corpus
        .documents()
        .iter()
        .map(|doc| doc.relative_path.clone())
        .collect();

    assert!(paths.iter().any(|path| path == "docs/live/page.md"));
    assert!(!paths.iter().any(|path| path.contains("templates")));
    assert!(!paths.iter().any(|path| path.starts_with("docs/snippets")));
    assert!(!paths.iter().any(|path| path.contains("generated")));
}

#[test]
fn build_registries_and_documents_loads_both() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_document_fixture(dir.path());

    let registry_root = fixtures_dir().join("../vp-registry/tests/fixtures");
    fs::create_dir_all(dir.path().join("spec/rfcs")).expect("spec/rfcs");
    fs::copy(
        registry_root.join("valid/registry.yaml"),
        dir.path().join("spec/rfcs/registry.yaml"),
    )
    .expect("copy rfc registry");
    fs::create_dir_all(dir.path().join("spec/terminology")).expect("spec/terminology");
    fs::copy(
        registry_root.join("term/valid/registry.yaml"),
        dir.path().join("spec/terminology/registry.yaml"),
    )
    .expect("copy term registry");

    let repo = SpecRepository::new(dir.path());
    let spec = SpecificationBuilder::new(&repo)
        .build_registries_and_documents()
        .expect("build");

    assert!(spec.registry_set.rfcs.get("VP-RFC-0000").is_some());
    assert!(spec.document_corpus.get("docs/live/example.md").is_some());
}

#[test]
fn real_veritypay_spec_document_corpus_loads_when_present() {
    let spec = fixtures_dir().join("../../../veritypay-spec");
    if !spec.join("README.md").is_file() {
        return;
    }

    let repo = SpecRepository::new(&spec);
    let corpus = SpecificationBuilder::new(&repo)
        .build_documents_only()
        .expect("build real spec documents")
        .document_corpus;

    assert!(!corpus.documents().is_empty());
    assert!(corpus.get("README.md").is_some());
}
