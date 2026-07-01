//! Integration test: valid registries load through vp-spec-model.

use std::fs;
use std::path::Path;

use vp_core::SpecRepository;
use vp_spec_model::SpecificationBuilder;

fn install_valid_fixtures(root: &Path) {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let rfc_fixture = format!("{manifest}/tests/fixtures/valid");
    let term_fixture = format!("{manifest}/tests/fixtures/term/valid");

    fs::create_dir_all(root.join("spec/rfcs")).expect("spec/rfcs");
    fs::copy(
        format!("{rfc_fixture}/registry.yaml"),
        root.join("spec/rfcs/registry.yaml"),
    )
    .expect("copy rfc registry");

    fs::create_dir_all(root.join("spec/terminology")).expect("spec/terminology");
    fs::copy(
        format!("{term_fixture}/registry.yaml"),
        root.join("spec/terminology/registry.yaml"),
    )
    .expect("copy term registry");
}

#[test]
fn valid_registries_load_through_spec_model() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_valid_fixtures(dir.path());

    let repo = SpecRepository::new(dir.path());
    let spec = SpecificationBuilder::new(&repo)
        .build_registries_only()
        .expect("build registries");

    assert!(spec.registry_set.terminology.get("VP-TERM-001").is_some());
    assert!(spec.registry_set.rfcs.get("VP-RFC-0000").is_some());
}
