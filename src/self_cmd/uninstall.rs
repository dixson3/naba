//! `naba self uninstall` (SPEC-SELF-001): remove the from-build install marker (and, with
//! `--force`, the installed binary at `~/.local/bin/naba`).
//!
//! The marker is always removed. The binary is removed only with `--force` — an unguarded delete
//! of the running binary is surprising, so the default reports the path and leaves it in place.

use std::path::Path;

use crate::cli::SelfUninstallArgs;
use crate::commands::Globals;
use crate::error::{AppError, AppResult};

use super::receipt::FromBuildMarker;

/// Dispatch `naba self uninstall`.
pub fn run(args: &SelfUninstallArgs, globals: &Globals) -> AppResult<()> {
    let marker_path = crate::dirs::from_build_marker_path();
    let bin = crate::dirs::bin_dir().join("naba");
    let outcome = do_uninstall(&bin, &marker_path, args.force)?;

    if globals.json {
        let obj = serde_json::json!({
            "command": "self uninstall",
            "status": "uninstalled",
            "marker_removed": outcome.marker_removed,
            "binary_removed": outcome.binary_removed,
            "binary_path": bin.to_string_lossy(),
        });
        println!("{}", serde_json::to_string_pretty(&obj).unwrap_or_default());
    } else if !globals.quiet {
        if outcome.marker_removed {
            println!("removed from-build marker");
        } else {
            println!("no from-build marker present");
        }
        if outcome.binary_removed {
            println!("removed {}", bin.display());
        } else if bin.exists() {
            println!(
                "left {} in place (re-run with --force to remove it)",
                bin.display()
            );
        }
    }
    Ok(())
}

/// The result of an uninstall.
pub struct UninstallOutcome {
    pub marker_removed: bool,
    pub binary_removed: bool,
}

/// Remove the marker (always) and, when `remove_binary`, the binary at `bin`. Test seam over
/// explicit paths. Idempotent — absent marker/binary is not an error.
pub fn do_uninstall(
    bin: &Path,
    marker_path: &Path,
    remove_binary: bool,
) -> AppResult<UninstallOutcome> {
    let marker_removed = marker_path.exists();
    FromBuildMarker::remove_from(marker_path)?;

    let mut binary_removed = false;
    if remove_binary && bin.exists() {
        std::fs::remove_file(bin)
            .map_err(|e| AppError::file_io(format!("remove {}: {e}", bin.display())))?;
        binary_removed = true;
    }
    Ok(UninstallOutcome {
        marker_removed,
        binary_removed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uninstall_removes_marker_and_optionally_binary() {
        let d = std::env::temp_dir().join(format!("naba-uninstall-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        let marker = d.join("naba-from-build.json");
        let bin = d.join("naba");
        std::fs::write(&marker, b"{}").unwrap();
        std::fs::write(&bin, b"binary").unwrap();

        // Without --force: marker gone, binary stays.
        let out = do_uninstall(&bin, &marker, false).unwrap();
        assert!(out.marker_removed);
        assert!(!out.binary_removed);
        assert!(!marker.exists());
        assert!(bin.exists());

        // With --force: binary removed too. Marker already gone → marker_removed=false.
        let out = do_uninstall(&bin, &marker, true).unwrap();
        assert!(!out.marker_removed);
        assert!(out.binary_removed);
        assert!(!bin.exists());

        // Idempotent: nothing left → both false, still Ok.
        let out = do_uninstall(&bin, &marker, true).unwrap();
        assert!(!out.marker_removed && !out.binary_removed);
        let _ = std::fs::remove_dir_all(&d);
    }
}
