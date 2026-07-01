//! VerityPay specification tooling CLI library.

pub mod config;
pub mod json;
pub mod output;
pub mod render;

pub use output::{OutputFormat, OutputOptions, write_validation_output};
