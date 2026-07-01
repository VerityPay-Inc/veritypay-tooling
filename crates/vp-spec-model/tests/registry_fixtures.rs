//! Integration tests for registry loading.

use std::fs;
use std::path::{Path, PathBuf};

use vp_core::SpecRepository;
use vp_spec_model::{BuildError, SpecificationBuilder};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn install_rfc_fixture(root: &Path) {
    let source = fixtures_dir().join("../vp-registry/tests/fixtures/valid");
    fs::create_dir_all(root.join("spec/rfcs")).expect("spec/rfcs");
    fs::copy(
        source.join("registry.yaml"),
        root.join("spec/rfcs/registry.yaml"),
    )
    .expect("copy rfc registry");
}

fn install_term_fixture(root: &Path) {
    let source = fixtures_dir().join("../vp-registry/tests/fixtures/term/valid");
    fs::create_dir_all(root.join("spec/terminology")).expect("spec/terminology");
    fs::copy(
        source.join("registry.yaml"),
        root.join("spec/terminology/registry.yaml"),
    )
    .expect("copy term registry");
}

fn install_valid_fixtures(root: &Path) {
    install_rfc_fixture(root);
    install_term_fixture(root);
}

#[test]
fn loads_valid_rfc_fixture() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_valid_fixtures(dir.path());

    let repo = SpecRepository::new(dir.path());
    let spec = SpecificationBuilder::new(&repo)
        .build_registries_only()
        .expect("build");

    let entry = spec
        .registry_set
        .rfcs
        .get("VP-RFC-0000")
        .expect("rfc entry");
    assert_eq!(entry.title, "RFC Process");
    assert_eq!(entry.path, "rfcs/0000-rfc-process.md");
}

#[test]
fn loads_valid_term_fixture() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_valid_fixtures(dir.path());

    let repo = SpecRepository::new(dir.path());
    let spec = SpecificationBuilder::new(&repo)
        .build_registries_only()
        .expect("build");

    let entry = spec
        .registry_set
        .terminology
        .get("VP-TERM-001")
        .expect("term entry");
    assert_eq!(entry.title, "Protocol");
    assert_eq!(
        entry
            .normative_definition
            .as_ref()
            .map(|d| d.section_id.as_str()),
        Some("DM-1.1")
    );
}

#[test]
fn lookup_by_id_works() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_valid_fixtures(dir.path());

    let repo = SpecRepository::new(dir.path());
    let spec = SpecificationBuilder::new(&repo)
        .build_registries_only()
        .expect("build");

    assert!(spec.registry_set.terminology.get("VP-TERM-001").is_some());
    assert!(spec.registry_set.rfcs.get("VP-RFC-0000").is_some());
    assert!(spec.registry_set.terminology.get("VP-TERM-999").is_none());
}

#[test]
fn missing_registry_file_returns_build_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_rfc_fixture(dir.path());

    let repo = SpecRepository::new(dir.path());
    let error = SpecificationBuilder::new(&repo)
        .build_registries_only()
        .expect_err("missing terminology registry");

    assert!(
        matches!(error, BuildError::RegistryMissing { path } if path == "spec/terminology/registry.yaml")
    );
}

#[test]
fn invalid_yaml_returns_build_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_valid_fixtures(dir.path());
    fs::write(
        dir.path().join("spec/terminology/registry.yaml"),
        "terms: [not a mapping\n",
    )
    .expect("write invalid yaml");

    let repo = SpecRepository::new(dir.path());
    let error = SpecificationBuilder::new(&repo)
        .build_registries_only()
        .expect_err("invalid yaml");

    assert!(
        matches!(error, BuildError::YamlInvalid { path, .. } if path == "spec/terminology/registry.yaml")
    );
}

#[test]
fn real_veritypay_spec_loads_when_present() {
    let spec = fixtures_dir().join("../../../veritypay-spec");
    if !spec.join("spec/terminology/registry.yaml").is_file()
        || !spec.join("spec/rfcs/registry.yaml").is_file()
    {
        return;
    }

    let repo = SpecRepository::new(&spec);
    let built = SpecificationBuilder::new(&repo)
        .build_registries_only()
        .expect("build real spec");

    assert!(!built.registry_set.terminology.entries().is_empty());
    assert!(!built.registry_set.rfcs.entries().is_empty());
    assert!(built.registry_set.terminology.get("VP-TERM-001").is_some());
    assert!(built.registry_set.rfcs.get("VP-RFC-0000").is_some());
}
