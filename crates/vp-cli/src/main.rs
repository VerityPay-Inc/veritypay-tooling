//! VerityPay specification tooling CLI.

use std::io;
use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};
use vp_cli::render::render_validation;
use vp_core::{ValidationContext, Validator};
use vp_crossref::CrossReferenceValidator;
use vp_engine::run_validation;
use vp_registry::{RfcRegistryValidator, TermRegistryValidator};

#[derive(Parser)]
#[command(name = "vp", about = "VerityPay specification tooling", version)]
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
        spec: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        None => {
            println!("vp (bootstrapping)");
        }
        Some(Command::Validate { spec }) => {
            if let Err(code) = run_validate(&spec) {
                process::exit(code);
            }
        }
    }
}

fn run_validate(spec: &PathBuf) -> Result<(), i32> {
    if !spec.is_dir() {
        eprintln!("error: spec path is not a directory: {}", spec.display());
        return Err(2);
    }

    let ctx = ValidationContext::new(spec);
    let rfc = RfcRegistryValidator::new();
    let term = TermRegistryValidator::new();
    let crossref = CrossReferenceValidator::new();
    let validators: [&dyn Validator; 3] = [&rfc, &term, &crossref];
    let result = run_validation(&ctx, &validators);

    let mut stdout = io::stdout().lock();
    if let Err(error) = render_validation(&result, &mut stdout) {
        eprintln!("error: failed to write validation output: {error}");
        return Err(1);
    }

    if result.has_errors() {
        return Err(1);
    }

    Ok(())
}
