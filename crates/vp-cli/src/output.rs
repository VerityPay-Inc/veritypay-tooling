//! Validation output dispatch (Milestone C.3.2).

use std::io::{self, Write};

use clap::ValueEnum;
use vp_engine::ValidationResult;

use crate::json::render_validation_json;
use crate::render::{render_quiet_summary, render_validation};

/// Output format for `vp validate`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
pub enum OutputFormat {
    /// Human-readable grouped diagnostics (default).
    #[default]
    Human,
    /// Machine-readable JSON for CI.
    Json,
}

/// Options controlling validation output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct OutputOptions {
    pub format: OutputFormat,
    pub quiet: bool,
}

impl OutputOptions {
    pub fn human() -> Self {
        Self::default()
    }

    pub fn json() -> Self {
        Self {
            format: OutputFormat::Json,
            quiet: false,
        }
    }

    pub fn quiet() -> Self {
        Self {
            format: OutputFormat::Human,
            quiet: true,
        }
    }
}

/// Write validation output according to `options`.
pub fn write_validation_output(
    result: &ValidationResult,
    options: OutputOptions,
    out: &mut dyn Write,
) -> io::Result<()> {
    match options.format {
        OutputFormat::Human if options.quiet => render_quiet_summary(result, out),
        OutputFormat::Human => render_validation(result, out),
        OutputFormat::Json => render_validation_json(result, out),
    }
}
