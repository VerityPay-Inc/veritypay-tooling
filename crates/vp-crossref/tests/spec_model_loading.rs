//! Integration test: cross-reference validation uses DocumentCorpus anchors.

use std::fs;
use std::path::Path;

use vp_core::{ValidationContext, Validator};
use vp_crossref::CrossReferenceValidator;
use vp_spec_model::SpecificationBuilder;

fn install_registries(root: &Path) {
    let manifest = env!("CARGO_MANIFEST_DIR");
    fs::create_dir_all(root.join("spec/terminology")).expect("spec/terminology");
    fs::copy(
        format!("{manifest}/../vp-registry/tests/fixtures/term/valid/registry.yaml"),
        root.join("spec/terminology/registry.yaml"),
    )
    .expect("copy term registry");
    fs::create_dir_all(root.join("spec/rfcs")).expect("spec/rfcs");
    fs::copy(
        format!("{manifest}/../vp-registry/tests/fixtures/valid/registry.yaml"),
        root.join("spec/rfcs/registry.yaml"),
    )
    .expect("copy rfc registry");
}

fn install_html_anchor_fixture(root: &Path) {
    fs::create_dir_all(root.join("docs")).expect("docs");
    fs::write(
        root.join("docs/page.md"),
        "Link to [section](target.md#dm-4-8).\n",
    )
    .expect("write page");
    fs::write(
        root.join("docs/target.md"),
        "# Target\n\n<a id=\"dm-4-8\"></a>\n\nSection body.\n",
    )
    .expect("write target");
}

#[test]
fn crossref_uses_document_corpus_anchor_extraction() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_registries(dir.path());
    install_html_anchor_fixture(dir.path());

    let repo = vp_core::SpecRepository::new(dir.path());
    let target = SpecificationBuilder::new(&repo)
        .build_registries_and_documents()
        .expect("build spec model")
        .document_corpus
        .get("docs/target.md")
        .expect("target document")
        .clone();

    assert!(target
        .sections
        .iter()
        .any(|section| section.anchor == "dm-4-8"));

    let findings = CrossReferenceValidator::new().validate(&ValidationContext::new(dir.path()));
    assert!(
        findings.is_empty(),
        "HTML anchor should resolve via DocumentCorpus sections: {findings:?}"
    );
}
