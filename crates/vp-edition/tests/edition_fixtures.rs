//! Fixture-based integration tests for Edition Manifest validation.

use std::fs;
use std::path::Path;

use vp_core::{ValidationConfig, ValidationContext, Validator};
use vp_edition::EditionValidator;

const MANIFEST_REL: &str = "editions/test-edition.yaml";
const DOC_REL: &str = "docs/example.md";

fn install_registries(root: &Path) {
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
}

fn install_pinned_document(root: &Path, version: &str) {
    fs::create_dir_all(root.join("docs")).expect("docs");
    fs::write(
        root.join(DOC_REL),
        format!("---\ntitle: Example\nversion: {version}\n---\n# Example\n"),
    )
    .expect("write pinned document");
}

fn write_manifest(root: &Path, contents: &str) {
    fs::create_dir_all(root.join("editions")).expect("editions");
    fs::write(root.join(MANIFEST_REL), contents).expect("write manifest");
}

fn valid_manifest_yaml() -> &'static str {
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
"#
}

fn ctx_with_edition(root: &Path) -> ValidationContext {
    ValidationContext::from_config(
        ValidationConfig::default()
            .with_spec_root(root)
            .with_edition(MANIFEST_REL),
    )
    .expect("context")
}

fn ctx_without_edition(root: &Path) -> ValidationContext {
    ValidationContext::new(root)
}

fn rule_present(findings: &[vp_diagnostics::Diagnostic], rule_id: &str) -> bool {
    findings.iter().any(|finding| finding.rule_id() == rule_id)
}

#[test]
fn fixture_no_edition_path_returns_no_diagnostics() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_registries(dir.path());
    write_manifest(dir.path(), valid_manifest_yaml());

    let findings = EditionValidator::new().validate(&ctx_without_edition(dir.path()));
    assert!(findings.is_empty(), "findings: {findings:?}");
}

#[test]
fn fixture_valid_minimal() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_registries(dir.path());
    install_pinned_document(dir.path(), "0.1.0");
    write_manifest(dir.path(), valid_manifest_yaml());

    let findings = EditionValidator::new().validate(&ctx_with_edition(dir.path()));
    assert!(findings.is_empty(), "findings: {findings:?}");
}

#[test]
fn fixture_missing_manifest() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_registries(dir.path());

    let findings = EditionValidator::new().validate(&ctx_with_edition(dir.path()));
    assert!(rule_present(&findings, "vp-edition-manifest-missing"));
}

#[test]
fn fixture_invalid_yaml() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_registries(dir.path());
    write_manifest(dir.path(), "edition: [not a mapping\n");

    let findings = EditionValidator::new().validate(&ctx_with_edition(dir.path()));
    assert!(rule_present(&findings, "vp-edition-yaml-invalid"));
}

#[test]
fn fixture_missing_required_field() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_registries(dir.path());
    write_manifest(
        dir.path(),
        r#"edition: Test
edition_id: vp-edition-test-1
protocol_version: vp-protocol-1.0
publication_date: 2026-06-29
status: candidate
accepted_rfcs: []
registry_snapshots: {}
conformance_baseline: []
"#,
    );

    let findings = EditionValidator::new().validate(&ctx_with_edition(dir.path()));
    assert!(rule_present(&findings, "vp-edition-missing-field"));
}

#[test]
fn fixture_invalid_edition_id() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_registries(dir.path());
    install_pinned_document(dir.path(), "0.1.0");

    let mut manifest = valid_manifest_yaml().to_string();
    manifest = manifest.replace("vp-edition-test-1", "genesis-1");
    write_manifest(dir.path(), &manifest);

    let findings = EditionValidator::new().validate(&ctx_with_edition(dir.path()));
    assert!(rule_present(&findings, "vp-edition-invalid-id"));
}

#[test]
fn fixture_invalid_status() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_registries(dir.path());
    install_pinned_document(dir.path(), "0.1.0");

    let mut manifest = valid_manifest_yaml().to_string();
    manifest = manifest.replace("status: candidate", "status: pending");
    write_manifest(dir.path(), &manifest);

    let findings = EditionValidator::new().validate(&ctx_with_edition(dir.path()));
    assert!(rule_present(&findings, "vp-edition-invalid-status"));
}

#[test]
fn fixture_missing_pinned_document() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_registries(dir.path());
    write_manifest(dir.path(), valid_manifest_yaml());

    let findings = EditionValidator::new().validate(&ctx_with_edition(dir.path()));
    assert!(rule_present(&findings, "vp-edition-document-missing"));
}

#[test]
fn fixture_front_matter_version_mismatch() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_registries(dir.path());
    install_pinned_document(dir.path(), "0.2.0");
    write_manifest(dir.path(), valid_manifest_yaml());

    let findings = EditionValidator::new().validate(&ctx_with_edition(dir.path()));
    assert!(rule_present(&findings, "vp-edition-version-mismatch"));
}

#[test]
fn fixture_unknown_accepted_rfc() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_registries(dir.path());
    install_pinned_document(dir.path(), "0.1.0");

    let mut manifest = valid_manifest_yaml().to_string();
    manifest = manifest.replace("- VP-RFC-0000", "- VP-RFC-0099");
    write_manifest(dir.path(), &manifest);

    let findings = EditionValidator::new().validate(&ctx_with_edition(dir.path()));
    assert!(rule_present(&findings, "vp-edition-unknown-rfc"));
}

#[test]
fn fixture_missing_registry_snapshot() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_registries(dir.path());
    install_pinned_document(dir.path(), "0.1.0");

    let mut manifest = valid_manifest_yaml().to_string();
    manifest = manifest.replace(
        "rfcs: spec/rfcs/registry.yaml",
        "rfcs: spec/rfcs/missing-registry.yaml",
    );
    write_manifest(dir.path(), &manifest);

    let findings = EditionValidator::new().validate(&ctx_with_edition(dir.path()));
    assert!(rule_present(&findings, "vp-edition-registry-missing"));
}

#[test]
fn fixture_invalid_conformance_id() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_registries(dir.path());
    install_pinned_document(dir.path(), "0.1.0");

    let mut manifest = valid_manifest_yaml().to_string();
    manifest = manifest.replace("- VP-CS-0001", "- VP-CS-1");
    write_manifest(dir.path(), &manifest);

    let findings = EditionValidator::new().validate(&ctx_with_edition(dir.path()));
    assert!(rule_present(&findings, "vp-edition-invalid-conformance-id"));
}

#[test]
fn fixture_registry_snapshot_rev_suffix() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_registries(dir.path());
    install_pinned_document(dir.path(), "0.1.0");

    let mut manifest = valid_manifest_yaml().to_string();
    manifest = manifest.replace(
        "rfcs: spec/rfcs/registry.yaml",
        "rfcs: spec/rfcs/registry.yaml@rev-2026-06-29",
    );
    write_manifest(dir.path(), &manifest);

    let findings = EditionValidator::new().validate(&ctx_with_edition(dir.path()));
    assert!(findings.is_empty(), "findings: {findings:?}");
}
