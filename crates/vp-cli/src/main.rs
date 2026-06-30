//! VerityPay specification tooling CLI.

use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};
use vp_core::ValidationContext;
use vp_engine::run_validation;

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
    let validators: [&dyn vp_core::Validator; 0] = [];
    let report = run_validation(&ctx, &validators);

    print_summary(&report);

    if report.has_errors() {
        return Err(1);
    }

    Ok(())
}

fn print_summary(report: &vp_diagnostics::Report) {
    println!(
        "{} errors, {} warnings, {} info",
        report.error_count, report.warning_count, report.info_count
    );
}
