//! `naba self install --from-build` (SPEC-SELF-001). Scaffold — implemented in Issue B.7.

use crate::cli::SelfInstallArgs;
use crate::commands::Globals;
use crate::error::{AppError, AppResult};

pub fn run(_args: &SelfInstallArgs, _globals: &Globals) -> AppResult<()> {
    Err(AppError::unimplemented("self install"))
}
