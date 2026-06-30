use std::process::Command;

#[test]
fn validate_empty_validators_exits_zero() {
    let bin = env!("CARGO_BIN_EXE_vp");
    let spec = env!("CARGO_MANIFEST_DIR");

    let output = Command::new(bin)
        .args(["validate", "--spec", spec])
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
