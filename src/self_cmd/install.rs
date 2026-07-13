//! `naba self install --from-build` (SPEC-SELF-001): record the currently-running build as a
//! from-build install.
//!
//! Copies the running binary into the vendor bin dir (`~/.local/bin`, [`crate::dirs::bin_dir`])
//! when it is not already there, and writes the [`FromBuildMarker`] so the source classifier
//! reports [`Source::FromBuild`](super::source::Source::FromBuild). This is the "I built naba
//! from source and want it on my PATH" path — distinct from the cargo-dist vendor install.

use std::path::{Path, PathBuf};

use crate::cli::SelfInstallArgs;
use crate::commands::Globals;
use crate::error::{AppError, AppResult};
use crate::version;

use super::receipt::FromBuildMarker;

/// Dispatch `naba self install`.
pub fn run(args: &SelfInstallArgs, globals: &Globals) -> AppResult<()> {
    if !args.from_build {
        return Err(AppError::usage(
            "naba self install currently supports only --from-build",
        ));
    }
    let exe = std::env::current_exe()
        .map_err(|e| AppError::general(format!("resolve current_exe: {e}")))?;
    let bin_dir = crate::dirs::bin_dir();
    let dest = install_from_build(&exe, &bin_dir, version::VERSION, profile())?;

    if globals.json {
        let obj = serde_json::json!({
            "command": "self install",
            "status": "installed",
            "source": "from-build",
            "path": dest.to_string_lossy(),
            "version": version::VERSION,
        });
        println!("{}", serde_json::to_string_pretty(&obj).unwrap_or_default());
    } else if !globals.quiet {
        println!("installed naba (from-build) -> {}", dest.display());
    }
    Ok(())
}

/// The build profile this binary was compiled with (`debug`/`release`).
fn profile() -> &'static str {
    if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    }
}

/// Copy `exe` into `bin_dir` (as `naba`) when it is not already the same file, and write the
/// from-build marker. Returns the installed binary path. Test seam: explicit dirs/version.
pub fn install_from_build(
    exe: &Path,
    bin_dir: &Path,
    version: &str,
    profile: &str,
) -> AppResult<PathBuf> {
    std::fs::create_dir_all(bin_dir)
        .map_err(|e| AppError::file_io(format!("mkdir {}: {e}", bin_dir.display())))?;
    let dest = bin_dir.join("naba");

    // Skip the copy when the running binary already IS the destination (idempotent re-install).
    let same =
        std::fs::canonicalize(exe).ok() == std::fs::canonicalize(&dest).ok() && dest.exists();
    if !same {
        std::fs::copy(exe, &dest)
            .map_err(|e| AppError::file_io(format!("copy binary to {}: {e}", dest.display())))?;
        set_exec(&dest);
    }

    FromBuildMarker::new(version, profile).write()?;
    Ok(dest)
}

#[cfg(unix)]
fn set_exec(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755));
}

#[cfg(not(unix))]
fn set_exec(_path: &Path) {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // install_from_build writes the marker at dirs::from_build_marker_path(), which reads HOME/
    // NABA_CONFIG_DIR — serialize + isolate.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn install_copies_binary_and_writes_marker() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let d = std::env::temp_dir().join(format!("naba-install-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();

        // Isolate the marker path under a temp config dir.
        let prev_cfg = std::env::var_os("NABA_CONFIG_DIR");
        std::env::set_var("NABA_CONFIG_DIR", d.join("config"));

        // A fake "current exe".
        let exe = d.join("built/naba");
        std::fs::create_dir_all(exe.parent().unwrap()).unwrap();
        std::fs::write(&exe, b"the-built-binary").unwrap();

        let bin_dir = d.join("bin");
        let dest = install_from_build(&exe, &bin_dir, "0.3.0", "release").unwrap();

        assert_eq!(dest, bin_dir.join("naba"));
        assert_eq!(std::fs::read(&dest).unwrap(), b"the-built-binary");
        let marker = FromBuildMarker::load().unwrap().unwrap();
        assert_eq!(marker.source, "from-build");
        assert_eq!(marker.version, "0.3.0");
        assert_eq!(marker.profile, "release");

        match prev_cfg {
            Some(v) => std::env::set_var("NABA_CONFIG_DIR", v),
            None => std::env::remove_var("NABA_CONFIG_DIR"),
        }
        let _ = std::fs::remove_dir_all(&d);
    }
}
