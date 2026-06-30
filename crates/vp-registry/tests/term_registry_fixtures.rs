//! Fixture-based integration tests for VP-TERM registry validation.

use std::fs;
use std::path::{Path, PathBuf};

use vp_core::{ValidationContext, Validator};
use vp_registry::TermRegistryValidator;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/term")
}

fn install_fixture(name: &str, target: &Path) {
    let source = fixtures_dir().join(name);
    let registry_dst_dir = target.join("spec/terminology");
    fs::create_dir_all(&registry_dst_dir).expect("create spec/terminology");
    fs::copy(
        source.join("registry.yaml"),
        registry_dst_dir.join("registry.yaml"),
    )
    .expect("copy registry");
}

fn rule_present(findings: &[vp_diagnostics::Diagnostic], rule_id: &str) -> bool {
    findings.iter().any(|d| d.rule_id() == rule_id)
}

#[test]
fn fixture_valid_minimal() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_fixture("valid", dir.path());

    let validator = TermRegistryValidator::new();
    let findings = validator.validate(&ValidationContext::new(dir.path()));
    assert!(findings.is_empty(), "findings: {findings:?}");
}

#[test]
fn fixture_missing_registry() {
    let dir = tempfile::tempdir().expect("tempdir");
    let validator = TermRegistryValidator::new();
    let findings = validator.validate(&ValidationContext::new(dir.path()));
    assert!(rule_present(&findings, "vp-term-registry-missing"));
}

#[test]
fn fixture_invalid_yaml() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_fixture("invalid_yaml", dir.path());

    let validator = TermRegistryValidator::new();
    let findings = validator.validate(&ValidationContext::new(dir.path()));
    assert!(rule_present(&findings, "vp-term-registry-yaml-invalid"));
}

#[test]
fn fixture_duplicate_id() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_fixture("duplicate_id", dir.path());

    let validator = TermRegistryValidator::new();
    let findings = validator.validate(&ValidationContext::new(dir.path()));
    assert!(rule_present(&findings, "vp-term-duplicate-id"));
}

#[test]
fn fixture_duplicate_title() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_fixture("duplicate_title", dir.path());

    let validator = TermRegistryValidator::new();
    let findings = validator.validate(&ValidationContext::new(dir.path()));
    assert!(rule_present(&findings, "vp-term-duplicate-title"));
}

#[test]
fn fixture_unknown_stability() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_fixture("unknown_stability", dir.path());

    let validator = TermRegistryValidator::new();
    let findings = validator.validate(&ValidationContext::new(dir.path()));
    assert!(rule_present(&findings, "vp-term-unknown-stability"));
}

#[test]
fn fixture_missing_normative_definition() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_fixture("missing_normative_definition", dir.path());

    let validator = TermRegistryValidator::new();
    let findings = validator.validate(&ValidationContext::new(dir.path()));
    assert!(rule_present(
        &findings,
        "vp-term-invalid-normative-definition"
    ));
}

#[test]
fn fixture_unknown_dependency() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_fixture("unknown_dependency", dir.path());

    let validator = TermRegistryValidator::new();
    let findings = validator.validate(&ValidationContext::new(dir.path()));
    assert!(rule_present(&findings, "vp-term-unknown-reference"));
}

#[test]
fn fixture_invalid_section_id() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_fixture("invalid_section_id", dir.path());

    let validator = TermRegistryValidator::new();
    let findings = validator.validate(&ValidationContext::new(dir.path()));
    assert!(rule_present(&findings, "vp-term-invalid-section-id"));
}

#[test]
fn real_veritypay_spec_terminology_passes_when_present() {
    let spec = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../veritypay-spec");
    if !spec.join("spec/terminology/registry.yaml").is_file() {
        return;
    }

    let validator = TermRegistryValidator::new();
    let findings = validator.validate(&ValidationContext::new(&spec));
    assert!(
        findings.is_empty(),
        "veritypay-spec terminology findings: {findings:?}"
    );
}
