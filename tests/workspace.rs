//! Workspace integration tests.

#[test]
fn workspace_crates_are_linkable() {
    use vp_core::{ValidationContext, Validator};
    use vp_diagnostics::{Category, Diagnostic, Report, Severity};
    use vp_engine::run_validation;

    let _ = (
        ValidationContext::new("."),
        Report::default(),
        Diagnostic::new(Severity::Info, "link", Category::Future, "ok"),
    );

    let report = run_validation(&ValidationContext::new("."), &[] as &[&dyn Validator]);
    assert!(!report.has_errors());
}
