//! naba — Nanobanana image generation CLI (Rust port, plan-004).
//!
//! Entry point: parse the CLI, force clap parse errors to exit 1 (SPEC-EXIT-003),
//! run TTY autodetect (SPEC-GLOBAL-003), dispatch, and map errors to their exit codes
//! (SPEC-EXIT-001/002).

// Scaffolding: several error constructors, exit-code constants, and the doctor version
// formatter are the public surface for later issues (2.2–2.6, 3.x, 4.x) and are not yet
// exercised by the stubs.
#![allow(dead_code)]

mod cli;
mod commands;
mod config;
mod dirs;
mod doctor;
mod embed;
mod error;
mod harness;
mod mcp;
mod output;
mod preflight;
mod prompt;
mod provider;
mod self_cmd;
mod skills;
mod skills_install;
mod version;

use std::io::{stdin, stdout, IsTerminal, Write};
use std::process::exit;

use clap::error::ErrorKind;
use clap::Parser;

use cli::Cli;
use commands::Globals;

#[tokio::main]
async fn main() {
    // SPEC-EXIT-003 / SPEC-INV-004: clap parse errors must exit 1, not clap's default 2.
    // --help / --version still render to stdout and exit 0 via clap's own `.exit()`.
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => match e.kind() {
            ErrorKind::DisplayHelp
            | ErrorKind::DisplayVersion
            | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => e.exit(),
            _ => {
                // Print clap's formatted diagnostic to stderr, then override the exit code to 1.
                let _ = e.print();
                exit(error::exit::GENERAL);
            }
        },
    };

    // SPEC-GLOBAL-003: TTY autodetect. stdout not a terminal → force --json;
    // stdin not a terminal → force --no-input (accepted but drives nothing, SPEC-GLOBAL-004).
    let json = cli.json || !stdout().is_terminal();
    let no_input = cli.no_input || !stdin().is_terminal();

    let globals = Globals {
        json,
        output: cli.output,
        quiet: cli.quiet,
        model: cli.model,
        no_input,
        provider: cli.provider,
    };

    if let Err(err) = commands::dispatch(cli.command, &globals).await {
        // SPEC-EXIT-002: print the error to stderr (no prefix — matches Go's
        // `fmt.Fprintln(os.Stderr, err)`) and exit with the error's code.
        let mut stderr = std::io::stderr();
        let _ = writeln!(stderr, "{err}");
        exit(err.code);
    }
}
