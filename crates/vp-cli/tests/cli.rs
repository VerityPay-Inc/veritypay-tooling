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
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../vp-crossref/tests/fixtures/broken_anchor")
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

fn run_validate_in(cwd: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_vp"))
        .current_dir(cwd)
        .args(args)
        .output()
        .expect("run vp validate")
}

#[test]
fn validate_without_spec_or_config_exits_two() {
    let cwd = tempfile::tempdir().expect("tempdir");

    let output = run_validate_in(cwd.path(), &["validate"]);

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("missing spec root"));
}

#[test]
fn validate_uses_spec_root_from_vp_toml() {
    let cwd = tempfile::tempdir().expect("tempdir");
    let spec = valid_fixture_spec();

    fs::write(
        cwd.path().join(".vp.toml"),
        format!("[validation]\nspec_root = \"{}\"\n", spec.display()),
    )
    .expect("write config");

    let output = run_validate_in(cwd.path(), &["validate"]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("Validation passed."));
}

#[test]
fn validate_cli_spec_overrides_vp_toml_spec_root() {
    let cwd = tempfile::tempdir().expect("tempdir");
    let config_spec = broken_anchor_fixture_spec();
    let cli_spec = valid_fixture_spec();

    fs::write(
        cwd.path().join(".vp.toml"),
        format!("[validation]\nspec_root = \"{}\"\n", config_spec.display()),
    )
    .expect("write config");

    let output = run_validate_in(
        cwd.path(),
        &["validate", "--spec", cli_spec.to_str().expect("utf8 path")],
    );

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn validate_vp_toml_output_json_defaults_format() {
    let cwd = tempfile::tempdir().expect("tempdir");
    let spec = valid_fixture_spec();

    fs::write(
        cwd.path().join(".vp.toml"),
        format!(
            "[validation]\nspec_root = \"{}\"\noutput = \"json\"\n",
            spec.display()
        ),
    )
    .expect("write config");

    let output = run_validate_in(cwd.path(), &["validate"]);

    assert!(output.status.success());
    serde_json::from_slice::<serde_json::Value>(&output.stdout).expect("json stdout");
}

#[test]
fn validate_cli_format_human_overrides_vp_toml_json() {
    let cwd = tempfile::tempdir().expect("tempdir");
    let spec = valid_fixture_spec();

    fs::write(
        cwd.path().join(".vp.toml"),
        format!(
            "[validation]\nspec_root = \"{}\"\noutput = \"json\"\n",
            spec.display()
        ),
    )
    .expect("write config");

    let output = run_validate_in(cwd.path(), &["validate", "--format", "human"]);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Running validators..."));
}

#[test]
fn validate_invalid_vp_toml_exits_two() {
    let cwd = tempfile::tempdir().expect("tempdir");
    fs::write(cwd.path().join(".vp.toml"), "not valid toml [[[").expect("write");

    let output = run_validate_in(cwd.path(), &["validate", "--spec", "/tmp"]);

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("invalid `.vp.toml`"));
}

#[test]
fn validate_unknown_vp_toml_section_exits_two() {
    let cwd = tempfile::tempdir().expect("tempdir");
    fs::write(cwd.path().join(".vp.toml"), "[profile]\nname = \"ci\"\n").expect("write");

    let output = run_validate_in(cwd.path(), &["validate", "--spec", "/tmp"]);

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("unknown section"));
}

#[test]
fn validate_unknown_vp_toml_validation_key_exits_two() {
    let cwd = tempfile::tempdir().expect("tempdir");
    fs::write(cwd.path().join(".vp.toml"), "[validation]\nfoo = \"bar\"\n").expect("write");

    let output = run_validate_in(cwd.path(), &["validate", "--spec", "/tmp"]);

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("unknown key"));
}
