//! Fixture-based integration tests for VP-RFC registry validation.

use std::fs;
use std::path::{Path, PathBuf};

use vp_core::{ValidationContext, Validator};
use vp_registry::RegistryValidator;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn install_fixture(name: &str, target: &Path) {
    let source = fixtures_dir().join(name);
    let registry_src = source.join("registry.yaml");
    let registry_dst_dir = target.join("spec/rfcs");
    fs::create_dir_all(&registry_dst_dir).expect("create spec/rfcs");
    fs::copy(registry_src, registry_dst_dir.join("registry.yaml")).expect("copy registry");

    if name == "valid" || name == "unknown_status" || name == "unknown_dependency" {
        fs::create_dir_all(target.join("rfcs")).expect("create rfcs");
        fs::write(target.join("rfcs/0000-rfc-process.md"), "# RFC").expect("write rfc file");
    }
    if name == "duplicate_id" {
        fs::create_dir_all(target.join("rfcs")).expect("create rfcs");
        fs::write(target.join("rfcs/a.md"), "a").expect("write a");
        fs::write(target.join("rfcs/b.md"), "b").expect("write b");
    }
}

fn rule_present(findings: &[vp_diagnostics::Diagnostic], rule_id: &str) -> bool {
    findings.iter().any(|d| d.rule_id() == rule_id)
}

#[test]
fn fixture_valid_minimal() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_fixture("valid", dir.path());

    let validator = RegistryValidator::new();
    let findings = validator.validate(&ValidationContext::new(dir.path()));
    assert!(findings.is_empty(), "findings: {findings:?}");
}

#[test]
fn fixture_missing_registry() {
    let dir = tempfile::tempdir().expect("tempdir");
    let validator = RegistryValidator::new();
    let findings = validator.validate(&ValidationContext::new(dir.path()));
    assert!(rule_present(&findings, "vp-rfc-registry-missing"));
}

#[test]
fn fixture_invalid_yaml() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_fixture("invalid_yaml", dir.path());

    let validator = RegistryValidator::new();
    let findings = validator.validate(&ValidationContext::new(dir.path()));
    assert!(rule_present(&findings, "vp-rfc-registry-yaml-invalid"));
}

#[test]
fn fixture_duplicate_id() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_fixture("duplicate_id", dir.path());

    let validator = RegistryValidator::new();
    let findings = validator.validate(&ValidationContext::new(dir.path()));
    assert!(rule_present(&findings, "vp-rfc-duplicate-id"));
}

#[test]
fn fixture_unknown_status() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_fixture("unknown_status", dir.path());

    let validator = RegistryValidator::new();
    let findings = validator.validate(&ValidationContext::new(dir.path()));
    assert!(rule_present(&findings, "vp-rfc-unknown-status"));
}

#[test]
fn fixture_missing_path() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_fixture("missing_path", dir.path());

    let validator = RegistryValidator::new();
    let findings = validator.validate(&ValidationContext::new(dir.path()));
    assert!(rule_present(&findings, "vp-rfc-path-missing"));
}

#[test]
fn fixture_unknown_dependency() {
    let dir = tempfile::tempdir().expect("tempdir");
    install_fixture("unknown_dependency", dir.path());

    let validator = RegistryValidator::new();
    let findings = validator.validate(&ValidationContext::new(dir.path()));
    assert!(rule_present(&findings, "vp-rfc-unknown-reference"));
}

#[test]
fn real_veritypay_spec_registry_passes_when_present() {
    let spec = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../veritypay-spec");
    if !spec.join("spec/rfcs/registry.yaml").is_file() {
        return;
    }

    let validator = RegistryValidator::new();
    let findings = validator.validate(&ValidationContext::new(&spec));
    assert!(
        findings.is_empty(),
        "veritypay-spec registry findings: {findings:?}"
    );
}
