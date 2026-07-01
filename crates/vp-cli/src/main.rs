//! VerityPay specification tooling CLI.

use std::io;
use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};
use vp_cli::config::output_format_from_cli_flag;
use vp_cli::{OutputFormat, OutputOptions, write_validation_output};
use vp_core::{
    load_vp_toml_from_cwd, resolve_config_with_spec_root, ConfigError, ValidationConfigOverrides,
    ValidationContext, ValidationOutput, Validator,
};
use vp_crossref::CrossReferenceValidator;
use vp_engine::run_validation;
use vp_registry::{RfcRegistryValidator, TermRegistryValidator};

#[derive(Parser)]
#[command(
    name = "vp",
    about = "VerityPay specification tooling",
    version,
    after_help = "Configuration:\n  Optional `.vp.toml` in the current directory:\n\n  [validation]\n  spec_root = \"../veritypay-spec\"\n  output = \"human\"\n\n  CLI flags override `.vp.toml` values."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Run specification validators against a checkout.
    Validate {
        /// Path to a `veritypay-spec` repository root.
        #[arg(long)]
        spec: Option<PathBuf>,

        /// Output format (`human` or `json`). Overrides `[validation].output`.
        #[arg(long, value_enum)]
        format: Option<OutputFormat>,

        /// Print only the validation summary counts.
        #[arg(long)]
        quiet: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        None => {
            println!("vp (bootstrapping)");
        }
        Some(Command::Validate {
            spec,
            format,
            quiet,
        }) => {
            if let Err(code) = run_validate(spec, format, quiet) {
                process::exit(code);
            }
        }
    }
}

fn run_validate(
    spec: Option<PathBuf>,
    format: Option<OutputFormat>,
    quiet: bool,
) -> Result<(), i32> {
    let cwd = std::env::current_dir().map_err(|error| {
        eprintln!("error: {error}");
        2
    })?;

    let file_overrides = load_vp_toml_from_cwd().map_err(|error| {
        eprintln!("error: {}", error.message());
        2
    })?;

    let mut cli_overrides = ValidationConfigOverrides::default();
    if let Some(spec) = spec {
        cli_overrides.spec_root = Some(spec);
    }
    if let Some(format) = format {
        cli_overrides.output = Some(match format {
            OutputFormat::Human => ValidationOutput::Human,
            OutputFormat::Json => ValidationOutput::Json,
        });
    }

    let config = resolve_config_with_spec_root(
        file_overrides.as_ref(),
        &cli_overrides,
        &cwd,
    )
    .map_err(|error| {
        eprintln!("error: {}", error.message());
        2
    })?;

    let spec_root = config
        .spec_root
        .as_ref()
        .expect("resolved config always has spec_root");

    if !spec_root.is_dir() {
        eprintln!("error: spec path is not a directory: {}", spec_root.display());
        return Err(2);
    }

    let ctx = ValidationContext::from_config(config.clone()).map_err(|_| {
        eprintln!("error: {}", ConfigError::MissingSpecRoot.message());
        2
    })?;

    let output_options = OutputOptions {
        format: output_format_from_cli_flag(format, ctx.config()),
        quiet,
    };

    let rfc = RfcRegistryValidator::new();
    let term = TermRegistryValidator::new();
    let crossref = CrossReferenceValidator::new();
    let validators: [&dyn Validator; 3] = [&rfc, &term, &crossref];
    let result = run_validation(&ctx, &validators);

    let mut stdout = io::stdout().lock();
    if let Err(error) = write_validation_output(&result, output_options, &mut stdout) {
        eprintln!("error: failed to write validation output: {error}");
        return Err(1);
    }

    if result.has_errors() {
        return Err(1);
    }

    Ok(())
}
