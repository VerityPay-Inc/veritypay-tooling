use std::path::PathBuf;
use std::process::Command;

fn valid_fixture_spec() -> PathBuf {
    let fixture =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../vp-registry/tests/fixtures/valid");
    let spec_root = tempfile::tempdir().expect("tempdir");
    let root = spec_root.path();

    std::fs::create_dir_all(root.join("spec/rfcs")).expect("spec/rfcs");
    std::fs::copy(
        fixture.join("registry.yaml"),
        root.join("spec/rfcs/registry.yaml"),
    )
    .expect("copy registry");
    std::fs::create_dir_all(root.join("rfcs")).expect("rfcs");
    std::fs::write(root.join("rfcs/0000-rfc-process.md"), "# RFC").expect("rfc file");

    spec_root.keep()
}

#[test]
fn validate_with_registry_validator_exits_zero() {
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
    assert!(stdout.contains("0 errors, 0 warnings, 0 info"));
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
