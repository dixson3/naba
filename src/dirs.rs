//! XDG base-directory resolution for naba's self-update / vendor-install / preflight surfaces
//! (SPEC-DIRS-001).
//!
//! naba's per-app leaf is `naba`. The four resolved locations:
//!
//! | Dir     | Default            | Env override        | Holds                                   |
//! |:--------|:-------------------|:--------------------|:----------------------------------------|
//! | config  | `~/.config/naba`   | (see [`config_dir`])| receipt + from-build marker             |
//! | cache   | `~/.cache/naba`    | `XDG_CACHE_HOME`    | update-check cache                      |
//! | data    | `~/.local/share/naba` | `XDG_DATA_HOME`  | reserved                                |
//! | bin     | `~/.local/bin`     | `XDG_BIN_HOME`      | the installed `naba` binary (vendor)    |
//!
//! # Config dir is the single source of truth
//!
//! [`config_dir`] delegates to [`crate::config::config_dir`] so `self`/`preflight` never diverge
//! from `config` (SPEC-DIRS-001). That resolver is `NABA_CONFIG_DIR` > `$XDG_CONFIG_HOME/naba` >
//! `~/.config/naba`.
//!
//! # Receipt path must match the cargo-dist installer
//!
//! The cargo-dist `curl|sh` installer writes the receipt to
//! `${XDG_CONFIG_HOME:-$HOME/.config}/naba/naba-receipt.json` — it honors `XDG_CONFIG_HOME` but
//! **not** naba-specific `NABA_CONFIG_DIR`. [`receipt_path`] resolves through [`config_dir`], so
//! in the common case (no `NABA_CONFIG_DIR`) it matches the installer exactly; setting
//! `NABA_CONFIG_DIR` deliberately moves naba's lookup away from the installer's path (a
//! documented caveat, SPEC-DIRS-001).

use std::path::PathBuf;

use crate::config;

/// The per-app directory leaf (`naba`) under each XDG base.
pub const APP: &str = "naba";

/// The cargo-dist receipt basename written by the vendor installer.
pub const RECEIPT_FILE: &str = "naba-receipt.json";

/// naba's own from-build marker basename (written by `naba self install --from-build`).
pub const FROM_BUILD_MARKER_FILE: &str = "naba-from-build.json";

/// The update-check cache basename under the cache dir.
pub const UPDATE_CHECK_FILE: &str = "update-check.json";

/// The skills-install target registry filename (plan-008 Issue 2.1).
pub const SKILLS_INSTALL_FILE: &str = "skills-install.json";

/// `$HOME`, or an empty path when unset (matches [`crate::config`]'s error-swallowing resolution).
fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default()
}

/// Resolve an XDG base dir: `<env-var>` if set and non-empty, else `<home>/<default-rel>`, then
/// join the `naba` app leaf. `default_rel` is a path relative to `$HOME` (e.g. `.cache`).
fn xdg_dir(env_var: &str, default_rel: &str) -> PathBuf {
    if let Some(base) = std::env::var_os(env_var) {
        if !base.is_empty() {
            return PathBuf::from(base).join(APP);
        }
    }
    let mut p = home_dir();
    for seg in default_rel.split('/') {
        p.push(seg);
    }
    p.join(APP)
}

/// Config dir (receipt + from-build marker). Delegates to [`crate::config::config_dir`] — the
/// single source of truth (SPEC-DIRS-001).
pub fn config_dir() -> PathBuf {
    config::config_dir()
}

/// Cache dir (`$XDG_CACHE_HOME/naba` > `~/.cache/naba`). Holds the update-check cache.
pub fn cache_dir() -> PathBuf {
    xdg_dir("XDG_CACHE_HOME", ".cache")
}

/// Data dir (`$XDG_DATA_HOME/naba` > `~/.local/share/naba`). Reserved.
pub fn data_dir() -> PathBuf {
    xdg_dir("XDG_DATA_HOME", ".local/share")
}

/// Binary install dir (`$XDG_BIN_HOME` > `~/.local/bin`). Where the vendor installer places the
/// `naba` binary. Note: no `naba` app leaf — `~/.local/bin` is shared, matching cargo-dist's
/// `install-path = "~/.local/bin"`.
pub fn bin_dir() -> PathBuf {
    if let Some(base) = std::env::var_os("XDG_BIN_HOME") {
        if !base.is_empty() {
            return PathBuf::from(base);
        }
    }
    home_dir().join(".local").join("bin")
}

/// The cargo-dist receipt path (`<config_dir>/naba-receipt.json`). Matches the installer's write
/// location in the common (no `NABA_CONFIG_DIR`) case — see the module docs.
pub fn receipt_path() -> PathBuf {
    config_dir().join(RECEIPT_FILE)
}

/// naba's own from-build marker path (`<config_dir>/naba-from-build.json`).
pub fn from_build_marker_path() -> PathBuf {
    config_dir().join(FROM_BUILD_MARKER_FILE)
}

/// The skills-install target registry path (`<config_dir>/skills-install.json`, plan-008
/// Issue 2.1). Records which harness/scope/path targets `naba skills install` wrote to, so an
/// unqualified `naba skills upgrade` can re-hit every previously-installed target.
pub fn skills_install_path() -> PathBuf {
    config_dir().join(SKILLS_INSTALL_FILE)
}

/// The update-check cache path (`<cache_dir>/update-check.json`), read by `skills preflight`'s
/// binary-up-to-date axis and written by the `self` update-check.
pub fn update_check_path() -> PathBuf {
    cache_dir().join(UPDATE_CHECK_FILE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // These tests read process-global env (HOME, XDG_*). Serialize them.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    struct EnvGuard {
        keys: Vec<(&'static str, Option<std::ffi::OsString>)>,
        _lock: std::sync::MutexGuard<'static, ()>,
    }

    impl EnvGuard {
        fn new(keys: &[&'static str]) -> Self {
            let lock = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
            let saved = keys
                .iter()
                .map(|k| (*k, std::env::var_os(k)))
                .collect::<Vec<_>>();
            for k in keys {
                std::env::remove_var(k);
            }
            EnvGuard {
                keys: saved,
                _lock: lock,
            }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for (k, v) in &self.keys {
                match v {
                    Some(val) => std::env::set_var(k, val),
                    None => std::env::remove_var(k),
                }
            }
        }
    }

    #[test]
    fn cache_dir_honors_xdg_then_default() {
        let _g = EnvGuard::new(&["XDG_CACHE_HOME", "HOME"]);
        std::env::set_var("HOME", "/home/tester");
        // Default under HOME.
        assert_eq!(cache_dir(), PathBuf::from("/home/tester/.cache/naba"));
        // XDG override wins.
        std::env::set_var("XDG_CACHE_HOME", "/xdg/cache");
        assert_eq!(cache_dir(), PathBuf::from("/xdg/cache/naba"));
    }

    #[test]
    fn data_dir_honors_xdg_then_default() {
        let _g = EnvGuard::new(&["XDG_DATA_HOME", "HOME"]);
        std::env::set_var("HOME", "/home/tester");
        assert_eq!(data_dir(), PathBuf::from("/home/tester/.local/share/naba"));
        std::env::set_var("XDG_DATA_HOME", "/xdg/data");
        assert_eq!(data_dir(), PathBuf::from("/xdg/data/naba"));
    }

    #[test]
    fn bin_dir_honors_xdg_bin_home_then_default() {
        let _g = EnvGuard::new(&["XDG_BIN_HOME", "HOME"]);
        std::env::set_var("HOME", "/home/tester");
        // ~/.local/bin default (no `naba` leaf — shared bin dir).
        assert_eq!(bin_dir(), PathBuf::from("/home/tester/.local/bin"));
        std::env::set_var("XDG_BIN_HOME", "/opt/bin");
        assert_eq!(bin_dir(), PathBuf::from("/opt/bin"));
    }

    #[test]
    fn receipt_and_marker_paths_live_under_config_dir() {
        let _g = EnvGuard::new(&["NABA_CONFIG_DIR", "XDG_CONFIG_HOME", "HOME"]);
        std::env::set_var("NABA_CONFIG_DIR", "/cfg");
        assert_eq!(receipt_path(), PathBuf::from("/cfg/naba-receipt.json"));
        assert_eq!(
            from_build_marker_path(),
            PathBuf::from("/cfg/naba-from-build.json")
        );
    }

    #[test]
    fn update_check_path_lives_under_cache_dir() {
        let _g = EnvGuard::new(&["XDG_CACHE_HOME", "HOME"]);
        std::env::set_var("XDG_CACHE_HOME", "/xdg/cache");
        assert_eq!(
            update_check_path(),
            PathBuf::from("/xdg/cache/naba/update-check.json")
        );
    }

    // SPEC-DIRS-001: the receipt lookup matches the cargo-dist installer's XDG_CONFIG_HOME
    // location in the common case (no NABA_CONFIG_DIR).
    #[test]
    fn receipt_path_matches_installer_xdg_config_home() {
        let _g = EnvGuard::new(&["NABA_CONFIG_DIR", "XDG_CONFIG_HOME", "HOME"]);
        std::env::set_var("XDG_CONFIG_HOME", "/xdg/config");
        assert_eq!(
            receipt_path(),
            PathBuf::from("/xdg/config/naba/naba-receipt.json")
        );
    }
}
