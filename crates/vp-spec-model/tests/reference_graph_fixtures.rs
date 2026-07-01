//! Integration tests for ReferenceGraph construction.

use std::fs;
use std::path::{Path, PathBuf};

use vp_core::SpecRepository;
use vp_spec_model::{ReferenceKind, ReferenceNodeKind, SpecificationBuilder};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn install_registries(root: &Path) {
    let registry_root = fixtures_dir().join("../vp-registry/tests/fixtures");
    fs::create_dir_all(root.join("spec/terminology")).expect("spec/terminology");
    fs::copy(
        registry_root.join("term/valid/registry.yaml"),
        root.join("spec/terminology/registry.yaml"),
    )
    .expect("copy term registry");
    fs::create_dir_all(root.join("spec/rfcs")).expect("spec/rfcs");
    fs::copy(
        registry_root.join("valid/registry.yaml"),
        root.join("spec/rfcs/registry.yaml"),
    )
    .expect("copy rfc registry");
}

fn install_reference_fixture(root: &Path) {
    fs::create_dir_all(root.join("docs")).expect("docs");
    fs::write(
        root.join("docs/page.md"),
        "See VP-TERM-001 and VP-RFC-0000.\n\nLink to [target](target.md#overview).\n",
    )
    .expect("write page");
    fs::write(
        root.join("docs/target.md"),
        "# Target\n\n## Overview\n\nBody.\n",
    )
    .expect("write target");
}

#[test]
fn graph_contains_vp_term_nodes() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_registries(dir.path());
    install_reference_fixture(dir.path());

    let graph = SpecificationBuilder::new(&SpecRepository::new(dir.path()))
        .build_registries_and_documents()
        .expect("build")
        .reference_graph()
        .clone();

    let term = graph.lookup("term:VP-TERM-001").expect("term node");
    assert_eq!(term.kind, ReferenceNodeKind::VpTerm);
    assert_eq!(term.display_name, "Protocol");
}

#[test]
fn graph_contains_vp_rfc_nodes() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_registries(dir.path());
    install_reference_fixture(dir.path());

    let graph = SpecificationBuilder::new(&SpecRepository::new(dir.path()))
        .build_registries_and_documents()
        .expect("build")
        .reference_graph()
        .clone();

    let rfc = graph.lookup("rfc:VP-RFC-0000").expect("rfc node");
    assert_eq!(rfc.kind, ReferenceNodeKind::VpRfc);
    assert_eq!(rfc.display_name, "RFC Process");
}

#[test]
fn edges_created_from_markdown_references() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_registries(dir.path());
    install_reference_fixture(dir.path());

    let graph = SpecificationBuilder::new(&SpecRepository::new(dir.path()))
        .build_registries_and_documents()
        .expect("build")
        .reference_graph()
        .clone();

    assert!(graph.edges().iter().any(|edge| {
        edge.source == "document:docs/page.md"
            && edge.target == "term:VP-TERM-001"
            && edge.reference_kind == ReferenceKind::Terminology
    }));
    assert!(graph.edges().iter().any(|edge| {
        edge.reference_kind == ReferenceKind::MarkdownAnchor
            && edge.target == "anchor:docs/target.md:overview"
    }));
}

#[test]
fn lookup_incoming_and_outgoing_work() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_registries(dir.path());
    install_reference_fixture(dir.path());

    let graph = SpecificationBuilder::new(&SpecRepository::new(dir.path()))
        .build_registries_and_documents()
        .expect("build")
        .reference_graph()
        .clone();

    let outgoing = graph.outgoing("document:docs/page.md");
    assert!(!outgoing.is_empty());
    assert!(outgoing.iter().any(|edge| edge.target == "term:VP-TERM-001"));

    let incoming = graph.incoming("term:VP-TERM-001");
    assert!(incoming.iter().any(|edge| edge.source == "document:docs/page.md"));
}

#[test]
fn real_veritypay_spec_reference_graph_builds_when_present() {
    let spec = fixtures_dir().join("../../../veritypay-spec");
    if !spec.join("spec/terminology/registry.yaml").is_file() {
        return;
    }

    let graph = SpecificationBuilder::new(&SpecRepository::new(&spec))
        .build_registries_and_documents()
        .expect("build real spec graph")
        .reference_graph()
        .clone();

    assert!(!graph.nodes().is_empty());
    assert!(!graph.edges().is_empty());
    assert!(graph.lookup("term:VP-TERM-001").is_some());
}
