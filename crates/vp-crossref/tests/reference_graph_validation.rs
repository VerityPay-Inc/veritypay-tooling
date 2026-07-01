//! Integration test: cross-reference validation consumes ReferenceGraph edges.

use std::fs;
use std::path::Path;

use vp_core::{ValidationContext, Validator};
use vp_crossref::{CrossReferenceValidator, CrossrefModel};
use vp_spec_model::{ReferenceKind, SpecificationBuilder};

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

#[test]
fn crossref_validator_consumes_reference_graph_edges() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_registries(dir.path());
    fs::create_dir_all(dir.path().join("docs")).expect("docs");
    fs::write(
        dir.path().join("docs/page.md"),
        "See VP-TERM-999 for details.\n",
    )
    .expect("write page");

    let repo = vp_core::SpecRepository::new(dir.path());
    let graph = SpecificationBuilder::new(&repo)
        .build_registries_and_documents()
        .expect("build spec model")
        .reference_graph()
        .clone();

    assert!(graph.edges().iter().any(|edge| {
        edge.reference_kind == ReferenceKind::Terminology && edge.symbolic_target == "VP-TERM-999"
    }));

    let model = CrossrefModel::load(&repo);
    assert!(model.uses_reference_graph());

    let findings = CrossReferenceValidator::new().validate(&ValidationContext::new(dir.path()));
    assert!(
        findings
            .iter()
            .any(|finding| finding.rule_id() == "vp-crossref-unknown-term"),
        "unknown term edge should produce diagnostic: {findings:?}"
    );
}
