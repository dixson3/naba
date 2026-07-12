//! `naba self` command group (SPEC-SELF-001..007): update / install / uninstall the naba
//! binary itself.
//!
//! Ported from yoshiko-flow's `self_cmd` module tree (see the plan-005 reference report). The
//! submodules, layered by Epic B issue:
//!
//! - [`source`] (B.2) — install-source classification (Vendor / Homebrew / FromBuild / Unknown).
//! - [`receipt`] (B.3) — the cargo-dist receipt + naba's own from-build marker.
//! - [`archive`] (B.4) — pure-Rust `.tar.gz` extraction + sha256 helpers.
//! - [`update`] (B.5/B.6) — the update pipeline (fetch manifest, verify, swap, skills refresh).
//! - [`install`] / [`uninstall`] (B.7) — from-build install management.
//!
//! This module is the CLI dispatch surface; the source-gate and Homebrew-refuse policy live in
//! [`update`]. naba is async (tokio); [`dispatch`] is `async` to match `commands::dispatch`, and
//! the blocking download/extract/swap work runs in a blocking context inside [`update`].

use crate::cli::SelfCommand;
use crate::commands::Globals;
use crate::error::AppResult;

// Submodules land per Epic B issue: `source` (B.2), `receipt` (B.3), `archive` (B.4) are
// declared when their files are added. The three dispatch targets below are scaffolded now.
pub mod install;
pub mod uninstall;
pub mod update;

/// Dispatch a `naba self <sub>` invocation.
pub async fn dispatch(command: SelfCommand, globals: &Globals) -> AppResult<()> {
    match command {
        SelfCommand::Update(args) => update::run(&args, globals).await,
        SelfCommand::Install(args) => install::run(&args, globals),
        SelfCommand::Uninstall(args) => uninstall::run(&args, globals),
    }
}
