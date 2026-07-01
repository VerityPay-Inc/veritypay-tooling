//! Fixture-based integration tests for cross-reference validation.

use std::fs;
use std::path::{Path, PathBuf};

use vp_core::{ValidationContext, Validator};
use vp_crossref::CrossReferenceValidator;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn install_registries(target: &Path) {
    let term_src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../vp-registry/tests/fixtures/term/valid/registry.yaml");
    let rfc_src =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../vp-registry/tests/fixtures/valid/registry.yaml");

    fs::create_dir_all(target.join("spec/terminology")).expect("spec/terminology");
    fs::create_dir_all(target.join("spec/rfcs")).expect("spec/rfcs");
    fs::copy(term_src, target.join("spec/terminology/registry.yaml")).expect("term registry");
    fs::copy(rfc_src, target.join("spec/rfcs/registry.yaml")).expect("rfc registry");
}

fn install_fixture(name: &str, target: &Path) {
    install_registries(target);
    let source = fixtures_dir().join(name);
    copy_dir_recursive(&source, target).expect("copy fixture");
}

fn copy_dir_recursive(from: &Path, to: &Path) -> std::io::Result<()> {
    if !from.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let src = entry.path();
        let dst = to.join(entry.file_name());
        if src.is_dir() {
            fs::create_dir_all(&dst)?;
            copy_dir_recursive(&src, &dst)?;
        } else {
            if let Some(parent) = dst.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&src, &dst)?;
        }
    }
    Ok(())
}

fn rule_present(findings: &[vp_diagnostics::Diagnostic], rule_id: &str) -> bool {
    findings.iter().any(|d| d.rule_id() == rule_id)
}

fn validate_fixture(name: &str) -> Vec<vp_diagnostics::Diagnostic> {
    let dir = tempfile::tempdir().expect("tempdir");
    install_fixture(name, dir.path());
    CrossReferenceValidator::new().validate(&ValidationContext::new(dir.path()))
}

#[test]
fn fixture_valid_term_reference() {
    let findings = validate_fixture("valid_term_ref");
    assert!(findings.is_empty(), "findings: {findings:?}");
}

#[test]
fn fixture_unknown_term_reference() {
    let findings = validate_fixture("unknown_term_ref");
    assert!(rule_present(&findings, "vp-crossref-unknown-term"));
}

#[test]
fn fixture_valid_rfc_reference() {
    let findings = validate_fixture("valid_rfc_ref");
    assert!(findings.is_empty(), "findings: {findings:?}");
}

#[test]
fn fixture_unknown_rfc_reference() {
    let findings = validate_fixture("unknown_rfc_ref");
    assert!(rule_present(&findings, "vp-crossref-unknown-rfc"));
}

#[test]
fn fixture_broken_relative_link() {
    let findings = validate_fixture("broken_link");
    assert!(rule_present(&findings, "vp-crossref-broken-link"));
}

#[test]
fn fixture_valid_relative_link() {
    let findings = validate_fixture("valid_link");
    assert!(!rule_present(&findings, "vp-crossref-broken-link"));
}

#[test]
fn fixture_broken_anchor() {
    let findings = validate_fixture("broken_anchor");
    assert!(rule_present(&findings, "vp-crossref-broken-anchor"));
}

#[test]
fn fixture_valid_anchor() {
    let findings = validate_fixture("valid_anchor");
    assert!(!rule_present(&findings, "vp-crossref-broken-anchor"));
}

#[test]
fn fixture_valid_html_anchor() {
    let findings = validate_fixture("valid_html_anchor");
    assert!(!rule_present(&findings, "vp-crossref-broken-anchor"));
}

#[test]
fn real_veritypay_spec_crossref_runs_when_present() {
    let spec = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../veritypay-spec");
    if !spec.join("spec/terminology/registry.yaml").is_file() {
        return;
    }

    let _findings =
        CrossReferenceValidator::new().validate(&ValidationContext::new(&spec));
}
