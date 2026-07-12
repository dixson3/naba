//! `naba self uninstall` (SPEC-SELF-001). Scaffold — implemented in Issue B.7.

use crate::cli::SelfUninstallArgs;
use crate::commands::Globals;
use crate::error::{AppError, AppResult};

pub fn run(_args: &SelfUninstallArgs, _globals: &Globals) -> AppResult<()> {
    Err(AppError::unimplemented("self uninstall"))
}
