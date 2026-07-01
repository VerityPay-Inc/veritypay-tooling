//! Validation output format mapping.

use vp_core::ValidationOutput;

use crate::output::OutputFormat;

pub fn output_format_from_config(config: &vp_core::ValidationConfig) -> OutputFormat {
    match config.output {
        ValidationOutput::Human => OutputFormat::Human,
        ValidationOutput::Json => OutputFormat::Json,
    }
}

pub fn output_format_from_cli_flag(flag: Option<OutputFormat>, config: &vp_core::ValidationConfig) -> OutputFormat {
    flag.unwrap_or_else(|| output_format_from_config(config))
}

impl From<ValidationOutput> for OutputFormat {
    fn from(value: ValidationOutput) -> Self {
        match value {
            ValidationOutput::Human => Self::Human,
            ValidationOutput::Json => Self::Json,
        }
    }
}
