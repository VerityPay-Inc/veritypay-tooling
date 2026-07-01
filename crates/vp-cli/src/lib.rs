//! VerityPay specification tooling CLI library.

pub mod config;
pub mod json;
pub mod output;
pub mod render;

pub use output::{write_validation_output, OutputFormat, OutputOptions};
