//! Integration test: Edition validator consumes registries through vp-spec-model.

use std::fs;
use std::path::Path;

use vp_core::{ValidationConfig, ValidationContext, Validator};
use vp_edition::EditionValidator;
use vp_spec_model::SpecificationBuilder;

fn install_valid_fixtures(root: &Path) {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let rfc_fixture = format!("{manifest}/../vp-registry/tests/fixtures/valid");
    let term_fixture = format!("{manifest}/../vp-registry/tests/fixtures/term/valid");

    fs::create_dir_all(root.join("spec/rfcs")).expect("spec/rfcs");
    fs::copy(
        format!("{rfc_fixture}/registry.yaml"),
        root.join("spec/rfcs/registry.yaml"),
    )
    .expect("copy rfc registry");
    fs::create_dir_all(root.join("rfcs")).expect("rfcs");
    fs::write(root.join("rfcs/0000-rfc-process.md"), "# RFC").expect("rfc file");

    fs::create_dir_all(root.join("spec/terminology")).expect("spec/terminology");
    fs::copy(
        format!("{term_fixture}/registry.yaml"),
        root.join("spec/terminology/registry.yaml"),
    )
    .expect("copy term registry");

    fs::create_dir_all(root.join("docs")).expect("docs");
    fs::write(
        root.join("docs/example.md"),
        "---\ntitle: Example\nversion: 0.1.0\n---\n# Example\n",
    )
    .expect("write pinned document");

    fs::create_dir_all(root.join("editions")).expect("editions");
    fs::write(
        root.join("editions/test-edition.yaml"),
        r#"edition: Test
edition_id: vp-edition-test-1
protocol_version: vp-protocol-1.0
publication_date: 2026-06-29
status: candidate
specification_documents:
  docs/example.md: "0.1.0"
accepted_rfcs:
  - VP-RFC-0000
registry_snapshots:
  rfcs: spec/rfcs/registry.yaml
conformance_baseline:
  - VP-CS-0001
"#,
    )
    .expect("write manifest");
}

#[test]
fn edition_validator_consumes_registry_set_through_spec_model() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_valid_fixtures(dir.path());

    let ctx = ValidationContext::from_config(
        ValidationConfig::default()
            .with_spec_root(dir.path())
            .with_edition("editions/test-edition.yaml"),
    )
    .expect("context");

    let registry_set = SpecificationBuilder::new(ctx.repository())
        .build_registries_only()
        .expect("build registries")
        .registry_set;

    assert!(registry_set.rfcs.get("VP-RFC-0000").is_some());

    let findings = EditionValidator::new().validate(&ctx);
    assert!(
        findings.is_empty(),
        "edition validation should pass using RegistrySet data: {findings:?}"
    );
}
