//! `naba self update` pipeline (SPEC-SELF-002..007). Scaffold — the fetch/verify/swap pipeline
//! and Homebrew-refuse policy land in Issue B.5/B.6.

use crate::cli::SelfUpdateArgs;
use crate::commands::Globals;
use crate::error::{AppError, AppResult};

pub async fn run(_args: &SelfUpdateArgs, _globals: &Globals) -> AppResult<()> {
    Err(AppError::unimplemented("self update"))
}
