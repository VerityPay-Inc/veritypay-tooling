use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn install_rfc_valid(root: &Path) {
    let fixture =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../vp-registry/tests/fixtures/valid");
    fs::create_dir_all(root.join("spec/rfcs")).expect("spec/rfcs");
    fs::copy(
        fixture.join("registry.yaml"),
        root.join("spec/rfcs/registry.yaml"),
    )
    .expect("copy rfc registry");
    fs::create_dir_all(root.join("rfcs")).expect("rfcs");
    fs::write(root.join("rfcs/0000-rfc-process.md"), "# RFC").expect("rfc file");
}

fn install_term_valid(root: &Path) {
    let fixture =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../vp-registry/tests/fixtures/term/valid");
    fs::create_dir_all(root.join("spec/terminology")).expect("spec/terminology");
    fs::copy(
        fixture.join("registry.yaml"),
        root.join("spec/terminology/registry.yaml"),
    )
    .expect("copy term registry");
}

fn valid_fixture_spec() -> PathBuf {
    let spec_root = tempfile::tempdir().expect("tempdir");
    let root = spec_root.path().to_path_buf();
    install_rfc_valid(&root);
    install_term_valid(&root);
    spec_root.keep()
}

fn broken_anchor_fixture_spec() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../vp-crossref/tests/fixtures/broken_anchor")
}

#[test]
fn validate_human_format_exits_zero() {
    let bin = env!("CARGO_BIN_EXE_vp");
    let spec = valid_fixture_spec();

    let output = Command::new(bin)
        .args(["validate", "--spec", spec.to_str().expect("utf8 path")])
        .output()
        .expect("run vp validate");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Running validators..."));
    assert!(stdout.contains("✓ RFC Registry"));
    assert!(stdout.contains("✓ Terminology Registry"));
    assert!(stdout.contains("✓ Cross References"));
    assert!(stdout.contains("Validation Summary"));
    assert!(stdout.contains("Errors:   0"));
    assert!(stdout.contains("Warnings: 0"));
    assert!(stdout.contains("Info:     0"));
    assert!(stdout.contains("Validation passed."));
}

#[test]
fn validate_json_format_exits_zero() {
    let bin = env!("CARGO_BIN_EXE_vp");
    let spec = valid_fixture_spec();

    let output = Command::new(bin)
        .args([
            "validate",
            "--spec",
            spec.to_str().expect("utf8 path"),
            "--format",
            "json",
        ])
        .output()
        .expect("run vp validate --format json");

    assert!(output.status.success());

    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse json stdout");
    assert_eq!(value["summary"]["errors"], 0);
    assert_eq!(value["summary"]["warnings"], 0);
    assert_eq!(value["summary"]["info"], 0);
    assert!(value["diagnostics"].as_array().unwrap().is_empty());
    assert!(!String::from_utf8_lossy(&output.stdout).contains("Running validators"));
}

#[test]
fn validate_json_format_reports_failures() {
    let bin = env!("CARGO_BIN_EXE_vp");
    let spec = broken_anchor_fixture_spec();

    let output = Command::new(bin)
        .args([
            "validate",
            "--spec",
            spec.to_str().expect("utf8 path"),
            "--format",
            "json",
        ])
        .output()
        .expect("run vp validate --format json");

    assert!(!output.status.success());

    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse json stdout");
    assert!(value["summary"]["errors"].as_u64().unwrap() >= 1);

    let diagnostics = value["diagnostics"].as_array().expect("diagnostics array");
    assert!(!diagnostics.is_empty());
    assert_eq!(diagnostics[0]["severity"], "error");
    assert!(diagnostics[0]["rule_id"].is_string());
    assert!(diagnostics[0]["message"].is_string());
}

#[test]
fn validate_quiet_format_exits_zero() {
    let bin = env!("CARGO_BIN_EXE_vp");
    let spec = valid_fixture_spec();

    let output = Command::new(bin)
        .args([
            "validate",
            "--spec",
            spec.to_str().expect("utf8 path"),
            "--quiet",
        ])
        .output()
        .expect("run vp validate --quiet");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        stdout,
        "Validation Summary\n\nErrors: 0\nWarnings: 0\nInfo: 0\n"
    );
    assert!(!stdout.contains("Running validators"));
    assert!(!stdout.contains("Validation passed."));
}

#[test]
fn validate_quiet_format_reports_failure_exit_code() {
    let bin = env!("CARGO_BIN_EXE_vp");
    let spec = broken_anchor_fixture_spec();

    let output = Command::new(bin)
        .args([
            "validate",
            "--spec",
            spec.to_str().expect("utf8 path"),
            "--quiet",
        ])
        .output()
        .expect("run vp validate --quiet");

    assert_eq!(output.status.code(), Some(1));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.starts_with("Validation Summary\n\n"));
    assert!(stdout.contains("Errors: "));
    assert!(!stdout.contains("error["));
}

#[test]
fn no_args_prints_bootstrapping() {
    let bin = env!("CARGO_BIN_EXE_vp");

    let output = Command::new(bin).output().expect("run vp");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "vp (bootstrapping)"
    );
}
