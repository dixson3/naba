//! Config load/save, env/path resolution, and `config get`/`config set` backing (Issue 3.1).
//!
//! Port of Go `internal/config/{config.go,auth.go}`. The YAML schema, key set, path
//! resolution, and precedence rules are pinned by SPEC §6 (SPEC-CFGSCHEMA-001..006), §3.8
//! (SPEC-CONFIG-001..003), and §9 (SPEC-ERR-007..009).
//!
//! # YAML crate (SPEC-MIGRATE-004)
//!
//! Uses **`serde_norway`** — a maintained fork of `serde_yaml`. **`serde_yml` is forbidden**
//! (RUSTSEC-2025-0068). `serde_norway::to_string` omits fields marked
//! `skip_serializing_if = "String::is_empty"`, reproducing Go's `omitempty`.
//!
//! # Home directory
//!
//! Resolved by reading `$HOME` directly (matching Go's `os.UserHomeDir`, which on unix is
//! `$HOME`) — no `dirs`/`home` crate dependency. An unset `$HOME` yields an empty base, the
//! same graceful-degradation Go shows when `UserHomeDir` errors.
//!
//! # Config auto-migration (Issue 3.2 — SPEC-MIGRATE-001..004)
//!
//! [`Config::load`] runs [`migrate_if_needed`] before the read/parse. For the **current**
//! schema this is a guaranteed **no-op**: the `provider`/`model` additions are
//! **additive-optional** (SPEC-MIGRATE-001), so absent keys resolve to defaults on read and
//! the file is never opened for writing — a load of an old 6-key `config.yaml` leaves it
//! **byte-identical on disk**, comments and all.
//!
//! The `.bak`-backed rewrite machinery ([`migrate_file`]) is nonetheless real and fully
//! tested: it hooks the seam a **future structural migration** would use
//! ([`needs_structural_migration`] returns `true`), backing the original up to `<path>.bak`
//! before a serde round-trip rewrite, idempotent and graceful on
//! empty/missing/malformed/already-new inputs (SPEC-MIGRATE-002). A structural rewrite
//! **loses YAML comments** (serde round-trip does not preserve them) — an accepted loss
//! mitigated by the `.bak` backup (SPEC-MIGRATE-003).

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::provider::gemini;
use crate::provider::select::ConfigDefaults;

const CONFIG_FILE_NAME: &str = "config.yaml";

/// `GEMINI_API_KEY` — the Gemini key env var and the config-file precedence winner
/// (SPEC-CFGSCHEMA-003).
pub const ENV_API_KEY: &str = "GEMINI_API_KEY";
/// `OPENROUTER_API_KEY` — the OpenRouter key env var. [NEW] env-only; no config key exists
/// for it in this port (SPEC-CFGSCHEMA-003).
pub const ENV_OPENROUTER_API_KEY: &str = "OPENROUTER_API_KEY";
/// `NABA_OUTPUT_DIR` — output-dir override, consumed by MCP only (SPEC-CFGSCHEMA-004/005).
pub const ENV_OUTPUT_DIR: &str = "NABA_OUTPUT_DIR";
/// `NABA_CONFIG_DIR` — config-dir override (SPEC-CFGSCHEMA-001).
pub const ENV_CONFIG_DIR: &str = "NABA_CONFIG_DIR";

/// The naba configuration (SPEC-CFGSCHEMA-001). All keys are optional (`omitempty` via
/// `skip_serializing_if`); a zero `Config` serializes to an empty document.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub api_key: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub model: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub default_output_dir: String,
    /// imageConfig aspect default; a per-call `--aspect` flag overrides it.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub aspect: String,
    /// imageConfig resolution default; a per-call `--resolution` flag overrides it.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub resolution: String,
    /// Model alias (fast/high). `model` beats it in [`Config::resolve_model`].
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub quality: String,
    /// [NEW] Provider selector default (gemini/openrouter). Consumed by the 2.5 selector via
    /// [`Config::to_config_defaults`].
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub provider: String,
}

/// Valid config keys (SPEC-CFGSCHEMA-001). Order is `api_key, model, provider,
/// default_output_dir, aspect, resolution, quality` — `provider` is [NEW] and its placement
/// (after `model`) is [DIVERGENCE] from Go's 6-key list. This order is what
/// `config get`/`config set` join into their `Valid keys:` error lines (SPEC-ERR-008/009).
pub const VALID_KEYS: [&str; 7] = [
    "api_key",
    "model",
    "provider",
    "default_output_dir",
    "aspect",
    "resolution",
    "quality",
];

/// The valid config keys (SPEC-CFGSCHEMA-001). See [`VALID_KEYS`] for the pinned ordering.
pub fn valid_keys() -> &'static [&'static str] {
    &VALID_KEYS
}

/// `VALID_KEYS` joined with `", "` for the `Valid keys: <list>` error lines.
fn valid_keys_joined() -> String {
    VALID_KEYS.join(", ")
}

/// The home directory from `$HOME`, or empty when unset (matches Go's error-swallowing
/// `UserHomeDir`).
fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default()
}

/// Config directory (SPEC-CFGSCHEMA-001, SPEC-DIRS-001): `NABA_CONFIG_DIR` >
/// `$XDG_CONFIG_HOME/naba` > `<home>/.config/naba`.
///
/// This is the **single source of truth** for the config dir — `self`/`preflight`
/// ([`crate::dirs`]) defer to it so they never diverge from `config`. The `XDG_CONFIG_HOME`
/// fallback was added for SPEC-DIRS so the naba **receipt lookup** matches where the
/// cargo-dist installer writes it (`${XDG_CONFIG_HOME:-$HOME/.config}/naba/naba-receipt.json`).
/// `NABA_CONFIG_DIR` is naba-specific and the installer does not honor it — overriding it moves
/// naba's lookup away from the installer's path (documented in SPEC-DIRS).
pub fn config_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os(ENV_CONFIG_DIR) {
        if !dir.is_empty() {
            return PathBuf::from(dir);
        }
    }
    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
        if !xdg.is_empty() {
            return PathBuf::from(xdg).join("naba");
        }
    }
    home_dir().join(".config").join("naba")
}

/// Full path to `config.yaml` (SPEC-CFGSCHEMA-001).
pub fn config_path() -> PathBuf {
    config_dir().join(CONFIG_FILE_NAME)
}

/// The MCP XDG-default output dir `<home>/.local/share/naba/images` (SPEC-CFGSCHEMA-004).
/// Empty when `$HOME` is unset (matches Go).
pub fn default_output_dir() -> String {
    let home = home_dir();
    if home.as_os_str().is_empty() {
        return String::new();
    }
    home.join(".local")
        .join("share")
        .join("naba")
        .join("images")
        .to_string_lossy()
        .into_owned()
}

impl Config {
    /// Load `config.yaml`. A missing file → zero-value [`Config`], no error
    /// (SPEC-CFGSCHEMA-002). A read error other than not-found, or a YAML parse error,
    /// surfaces as `ExitGeneral` (1) — doctor relies on parseability as a health signal.
    ///
    /// Read → parse pipeline; the Issue 3.2 migration ([`migrate_if_needed`]) slots ahead of
    /// the read. For the current additive-optional schema it is a no-op (SPEC-MIGRATE-001), so
    /// the file is left byte-identical.
    pub fn load() -> AppResult<Config> {
        let path = config_path();
        // SPEC-MIGRATE-001: zero-rewrite structural-migration check. A guaranteed no-op for the
        // current additive-optional schema — the file is never written and stays byte-identical.
        migrate_if_needed()?;
        let data = match std::fs::read_to_string(&path) {
            Ok(data) => data,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Config::default()),
            Err(e) => return Err(AppError::general(e.to_string())),
        };
        // SPEC-MIGRATE-001 zero-rewrite default: an empty file parses to a zero Config.
        serde_norway::from_str(&data).map_err(|e| AppError::general(e.to_string()))
    }

    /// Write `config.yaml`: mkdir `0o755`, file `0o644` (SPEC-CFGSCHEMA-002). Save failures
    /// surface as `ExitFileIO` (10) so the `config set` layer reports `save config: %v`.
    pub fn save(&self) -> AppResult<()> {
        let dir = config_dir();
        std::fs::create_dir_all(&dir).map_err(|e| AppError::file_io(e.to_string()))?;
        set_mode(&dir, 0o755);

        let data = serde_norway::to_string(self).map_err(|e| AppError::file_io(e.to_string()))?;
        let path = config_path();
        std::fs::write(&path, data.as_bytes()).map_err(|e| AppError::file_io(e.to_string()))?;
        set_mode(&path, 0o644);
        Ok(())
    }

    /// Value for `key`, or `""` for an unset field or an unknown key (SPEC-CONFIG-002 — the
    /// command layer distinguishes unset-vs-unknown; this accessor mirrors Go's `Get`).
    pub fn get(&self, key: &str) -> String {
        match key {
            "api_key" => self.api_key.clone(),
            "model" => self.model.clone(),
            "provider" => self.provider.clone(),
            "default_output_dir" => self.default_output_dir.clone(),
            "aspect" => self.aspect.clone(),
            "resolution" => self.resolution.clone(),
            "quality" => self.quality.clone(),
            _ => String::new(),
        }
    }

    /// Set `key` to `value`; returns `false` on an unknown key (SPEC-CONFIG-003 — the command
    /// layer turns `false` into the `unknown key` usage error, exit 2).
    #[must_use]
    pub fn set(&mut self, key: &str, value: &str) -> bool {
        match key {
            "api_key" => self.api_key = value.to_string(),
            "model" => self.model = value.to_string(),
            "provider" => self.provider = value.to_string(),
            "default_output_dir" => self.default_output_dir = value.to_string(),
            "aspect" => self.aspect = value.to_string(),
            "resolution" => self.resolution = value.to_string(),
            "quality" => self.quality = value.to_string(),
            _ => return false,
        }
        true
    }

    /// Model implied by config: `model` key > `quality`→model tier > `""` (unset)
    /// (SPEC-CFGSCHEMA-006). An invalid `quality` yields SPEC-ERR-007
    /// `invalid quality %q in config (valid: fast, high)`. The tier mapping stays in lockstep
    /// with [`gemini`]'s model constants.
    pub fn resolve_model(&self) -> AppResult<String> {
        if !self.model.is_empty() {
            return Ok(self.model.clone());
        }
        if !self.quality.is_empty() {
            return match self.quality.as_str() {
                "fast" => Ok(gemini::FLASH_MODEL.to_string()),
                "high" => Ok(gemini::PRO_MODEL.to_string()),
                other => Err(AppError::general(format!(
                    "invalid quality {other:?} in config (valid: fast, high)"
                ))),
            };
        }
        Ok(String::new())
    }

    /// Gemini API key: `GEMINI_API_KEY` env > config `api_key` (SPEC-CFGSCHEMA-003).
    pub fn resolve_api_key(&self) -> String {
        match std::env::var(ENV_API_KEY) {
            Ok(k) if !k.is_empty() => k,
            _ => self.api_key.clone(),
        }
    }

    /// [NEW] OpenRouter API key: `OPENROUTER_API_KEY` env only — no config key exists for it
    /// (SPEC-CFGSCHEMA-003). Empty when unset. `&self` for call-site symmetry with
    /// [`resolve_api_key`].
    pub fn resolve_openrouter_api_key(&self) -> String {
        std::env::var(ENV_OPENROUTER_API_KEY).unwrap_or_default()
    }

    /// Output dir (MCP only — the CLI ignores this, SPEC-CFGSCHEMA-005): `NABA_OUTPUT_DIR` env
    /// > config `default_output_dir` > XDG default `<home>/.local/share/naba/images`
    /// > (SPEC-CFGSCHEMA-004).
    pub fn resolve_output_dir(&self) -> String {
        if let Ok(dir) = std::env::var(ENV_OUTPUT_DIR) {
            if !dir.is_empty() {
                return dir;
            }
        }
        if !self.default_output_dir.is_empty() {
            return self.default_output_dir.clone();
        }
        default_output_dir()
    }

    /// The 2.5 selector seam (SPEC-PROVIDER-007): produce the [`ConfigDefaults`] the provider
    /// selector consumes. `provider` is the raw config value (None when unset); `model` is the
    /// **resolved** model (`model` > `quality`→tier, SPEC-CFGSCHEMA-006), so the selector never
    /// re-derives the quality tier. An invalid config `quality` surfaces SPEC-ERR-007 here.
    pub fn to_config_defaults(&self) -> AppResult<ConfigDefaults> {
        let model = self.resolve_model()?;
        Ok(ConfigDefaults {
            provider: opt(&self.provider),
            model: opt(&model),
        })
    }
}

// ---------------------------------------------------------------------------
// Config auto-migration (Issue 3.2 — SPEC-MIGRATE-001..004).
// ---------------------------------------------------------------------------

/// Filename suffix for the pre-migration backup (SPEC-MIGRATE-002): `config.yaml.bak`.
const MIGRATION_BACKUP_EXT: &str = "bak";

/// Whether the on-disk YAML requires a **structural** rewrite (SPEC-MIGRATE-001/002).
///
/// For the current schema this is **always `false`**: the `provider`/`model` additions are
/// additive-optional, so absent keys resolve to defaults on read and no rewrite is needed
/// (the zero-rewrite default). This is the seam a future structural migration hooks — it
/// would return `true` when `parsed` is in an old shape a serde round-trip must normalize,
/// keyed off a marker the round-trip removes (so the second run sees the new shape and
/// no-ops; see [`migrate_file`]'s idempotency contract).
fn needs_structural_migration(_parsed: &serde_norway::Value) -> bool {
    false
}

/// `<path>.bak` — the backup sibling (full filename plus `.bak`, so `config.yaml` →
/// `config.yaml.bak`, not `config.bak`).
fn backup_path(path: &Path) -> PathBuf {
    let mut s = path.as_os_str().to_owned();
    s.push(".");
    s.push(MIGRATION_BACKUP_EXT);
    PathBuf::from(s)
}

/// Run the structural-migration check against `config.yaml` (SPEC-MIGRATE-001/002).
///
/// The **zero-rewrite default**: for the current additive-optional schema
/// [`needs_structural_migration`] is always `false`, so this is a guaranteed no-op — the file
/// is never opened for writing and stays byte-identical (comments preserved). See
/// [`migrate_file`] for the machinery exercised when a real structural migration ships.
pub fn migrate_if_needed() -> AppResult<()> {
    migrate_file(&config_path(), needs_structural_migration)
}

/// Core migration engine (SPEC-MIGRATE-002/003), parameterized by the `needs` predicate so the
/// tests can drive a **synthetic** structural migration without one shipping in production.
///
/// Contract:
/// - **missing file** → no-op (`Ok`), no `.bak`;
/// - **empty / whitespace-only** → no-op;
/// - **malformed YAML** → no-op (skipped — [`Config::load`] surfaces the parse error);
/// - **already-new / additive-optional** (`needs` == `false`) → no-op, file untouched;
/// - **rewrite required** (`needs` == `true`) → write `<path>.bak` with the **original bytes**
///   first, then serde round-trip the document through [`Config`] and rewrite `<path>`.
///
/// **Idempotency** (SPEC-MIGRATE-002): a real `needs` predicate keys off an old-shape marker
/// that the round-trip normalizes away, so the second run observes the new shape and no-ops.
/// `.bak` is therefore written **at most once** and is never clobbered on a subsequent load —
/// the backup always holds the true pre-migration original.
///
/// **Comment loss** (SPEC-MIGRATE-003): the serde round-trip does not preserve YAML comments;
/// this is an accepted loss, mitigated by the `.bak` backup and the zero-rewrite default.
fn migrate_file(path: &Path, needs: fn(&serde_norway::Value) -> bool) -> AppResult<()> {
    let data = match std::fs::read_to_string(path) {
        Ok(d) => d,
        // Missing file: nothing to migrate (matches load()'s not-found → zero Config).
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        // Any other hard read error: best-effort skip — load() re-reads and surfaces it.
        Err(_) => return Ok(()),
    };
    // Empty / whitespace-only file: no document to migrate.
    if data.trim().is_empty() {
        return Ok(());
    }
    // Malformed YAML: skip the rewrite; load()'s parse re-surfaces the error, so we neither
    // crash nor destroy the user's (repairable) file.
    let value: serde_norway::Value = match serde_norway::from_str(&data) {
        Ok(v) => v,
        Err(_) => return Ok(()),
    };
    // Zero-rewrite default / already-migrated: leave the file byte-identical.
    if !needs(&value) {
        return Ok(());
    }
    // Structural migration required: back the original up, then round-trip rewrite.
    let bak = backup_path(path);
    std::fs::write(&bak, data.as_bytes()).map_err(|e| AppError::file_io(e.to_string()))?;
    set_mode(&bak, 0o644);
    let cfg: Config =
        serde_norway::from_str(&data).map_err(|e| AppError::general(e.to_string()))?;
    let rewritten = serde_norway::to_string(&cfg).map_err(|e| AppError::file_io(e.to_string()))?;
    std::fs::write(path, rewritten.as_bytes()).map_err(|e| AppError::file_io(e.to_string()))?;
    set_mode(path, 0o644);
    Ok(())
}

/// `Some(s)` for a non-empty string, else `None`.
fn opt(s: &str) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}

/// Set unix mode on a path; best-effort (a perms failure never fails the save on its own).
#[cfg(unix)]
fn set_mode(path: &std::path::Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;
    let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode));
}

#[cfg(not(unix))]
fn set_mode(_path: &std::path::Path, _mode: u32) {}

// ---------------------------------------------------------------------------
// config get / config set command backing (SPEC-CONFIG-002/003, SPEC-ERR-008/009).
// ---------------------------------------------------------------------------

/// Backing for `config get <key>` (SPEC-CONFIG-002): returns the value to print. Load error →
/// `ExitGeneral` `load config: %v`; unset key → `ExitGeneral` `key %q is not set` (SPEC-ERR-008).
/// An unknown key is also "not set" (Go's `Get` returns `""` for both) — the value is empty, so
/// it takes the unset branch, matching Go exactly.
pub fn get_value(key: &str) -> AppResult<String> {
    let cfg = Config::load().map_err(|e| AppError::general(format!("load config: {e}")))?;
    let value = cfg.get(key);
    if value.is_empty() {
        return Err(AppError::general(format!(
            "key {key:?} is not set\n\nValid keys: {}",
            valid_keys_joined()
        )));
    }
    Ok(value)
}

/// Backing for `config set <key> <value>` (SPEC-CONFIG-003): loads, sets, saves. Load error →
/// `ExitGeneral` `load config: %v`; unknown key → `ExitUsage` `unknown key %q` (SPEC-ERR-009,
/// exit 2); save error → `ExitFileIO` `save config: %v` (exit 10). On success returns `()`; the
/// command layer prints `Set %s = %s` unless `--quiet`.
pub fn set_value(key: &str, value: &str) -> AppResult<()> {
    let mut cfg = Config::load().map_err(|e| AppError::general(format!("load config: {e}")))?;
    if !cfg.set(key, value) {
        return Err(AppError::usage(format!(
            "unknown key {key:?}\n\nValid keys: {}",
            valid_keys_joined()
        )));
    }
    cfg.save()
        .map_err(|e| AppError::file_io(format!("save config: {e}")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::exit;
    use std::sync::{Mutex, MutexGuard};

    // Config path + several resolvers read process-global env (NABA_CONFIG_DIR, HOME,
    // GEMINI_API_KEY, ...). Serialize env-touching tests so they don't race.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    /// A test scope with an isolated NABA_CONFIG_DIR tempdir (no real home is touched) and the
    /// env lock held. Cleans the env vars it sets on drop.
    struct Scope {
        _guard: MutexGuard<'static, ()>,
        dir: PathBuf,
    }

    impl Scope {
        fn new(name: &str) -> Self {
            let guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
            let mut dir = std::env::temp_dir();
            dir.push(format!("naba-cfg-test-{}-{}", std::process::id(), name));
            let _ = std::fs::remove_dir_all(&dir);
            std::env::set_var(ENV_CONFIG_DIR, &dir);
            // Clear key/output env so tests control precedence explicitly.
            std::env::remove_var(ENV_API_KEY);
            std::env::remove_var(ENV_OPENROUTER_API_KEY);
            std::env::remove_var(ENV_OUTPUT_DIR);
            Scope { _guard: guard, dir }
        }
    }

    impl Drop for Scope {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.dir);
            std::env::remove_var(ENV_CONFIG_DIR);
            std::env::remove_var(ENV_API_KEY);
            std::env::remove_var(ENV_OPENROUTER_API_KEY);
            std::env::remove_var(ENV_OUTPUT_DIR);
        }
    }

    #[test]
    fn config_path_honors_env_override() {
        let s = Scope::new("path");
        assert_eq!(config_dir(), s.dir);
        assert_eq!(config_path(), s.dir.join("config.yaml"));
    }

    // SPEC-DIRS-001: config_dir precedence NABA_CONFIG_DIR > $XDG_CONFIG_HOME/naba > ~/.config/naba.
    #[test]
    fn config_dir_precedence_naba_then_xdg_then_default() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let prev_naba = std::env::var_os(ENV_CONFIG_DIR);
        let prev_xdg = std::env::var_os("XDG_CONFIG_HOME");
        let prev_home = std::env::var_os("HOME");

        // NABA_CONFIG_DIR wins over both XDG and HOME.
        std::env::set_var(ENV_CONFIG_DIR, "/explicit/naba-cfg");
        std::env::set_var("XDG_CONFIG_HOME", "/xdg");
        std::env::set_var("HOME", "/home/tester");
        assert_eq!(config_dir(), PathBuf::from("/explicit/naba-cfg"));

        // XDG_CONFIG_HOME/naba when NABA_CONFIG_DIR is unset.
        std::env::remove_var(ENV_CONFIG_DIR);
        assert_eq!(config_dir(), PathBuf::from("/xdg/naba"));

        // ~/.config/naba when both are unset — matches the cargo-dist installer default.
        std::env::remove_var("XDG_CONFIG_HOME");
        assert_eq!(config_dir(), PathBuf::from("/home/tester/.config/naba"));

        // Restore.
        match prev_naba {
            Some(v) => std::env::set_var(ENV_CONFIG_DIR, v),
            None => std::env::remove_var(ENV_CONFIG_DIR),
        }
        match prev_xdg {
            Some(v) => std::env::set_var("XDG_CONFIG_HOME", v),
            None => std::env::remove_var("XDG_CONFIG_HOME"),
        }
        match prev_home {
            Some(v) => std::env::set_var("HOME", v),
            None => std::env::remove_var("HOME"),
        }
    }

    #[test]
    fn load_missing_file_is_zero_config() {
        let _s = Scope::new("missing");
        let cfg = Config::load().unwrap();
        assert_eq!(cfg, Config::default());
    }

    #[test]
    fn save_then_load_round_trips_all_seven_keys() {
        let _s = Scope::new("roundtrip");
        let cfg = Config {
            api_key: "sk-abc".into(),
            model: "gemini-3-pro-image".into(),
            default_output_dir: "/tmp/out".into(),
            aspect: "16:9".into(),
            resolution: "2K".into(),
            quality: "high".into(),
            provider: "openrouter".into(),
        };
        cfg.save().unwrap();
        let loaded = Config::load().unwrap();
        assert_eq!(loaded, cfg);
        // provider specifically round-trips (the [NEW] key).
        assert_eq!(loaded.provider, "openrouter");
    }

    #[cfg(unix)]
    #[test]
    fn save_sets_dir_755_and_file_644() {
        use std::os::unix::fs::PermissionsExt;
        let _s = Scope::new("perms");
        Config {
            model: "m".into(),
            ..Default::default()
        }
        .save()
        .unwrap();
        let dmode = std::fs::metadata(config_dir())
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        let fmode = std::fs::metadata(config_path())
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(dmode, 0o755);
        assert_eq!(fmode, 0o644);
    }

    #[test]
    fn get_and_set_all_keys() {
        let mut cfg = Config::default();
        for key in VALID_KEYS {
            assert!(cfg.set(key, "v"), "set {key} should succeed");
            assert_eq!(cfg.get(key), "v");
        }
    }

    #[test]
    fn set_unknown_key_returns_false() {
        let mut cfg = Config::default();
        assert!(!cfg.set("bogus", "v"));
        assert_eq!(cfg.get("bogus"), "");
    }

    #[test]
    fn valid_keys_exact_set_and_order() {
        assert_eq!(
            valid_keys(),
            &[
                "api_key",
                "model",
                "provider",
                "default_output_dir",
                "aspect",
                "resolution",
                "quality"
            ]
        );
    }

    #[test]
    fn resolve_model_precedence() {
        // model beats quality.
        let cfg = Config {
            model: "explicit-model".into(),
            quality: "high".into(),
            ..Default::default()
        };
        assert_eq!(cfg.resolve_model().unwrap(), "explicit-model");
        // quality maps to tier when model unset.
        let cfg = Config {
            quality: "high".into(),
            ..Default::default()
        };
        assert_eq!(cfg.resolve_model().unwrap(), gemini::PRO_MODEL);
        let cfg = Config {
            quality: "fast".into(),
            ..Default::default()
        };
        assert_eq!(cfg.resolve_model().unwrap(), gemini::FLASH_MODEL);
        // neither → empty.
        assert_eq!(Config::default().resolve_model().unwrap(), "");
    }

    #[test]
    fn resolve_model_invalid_quality_is_err_007() {
        let cfg = Config {
            quality: "medium".into(),
            ..Default::default()
        };
        let err = cfg.resolve_model().unwrap_err();
        assert_eq!(
            err.message,
            "invalid quality \"medium\" in config (valid: fast, high)"
        );
    }

    #[test]
    fn resolve_api_key_env_over_config() {
        let _s = Scope::new("apikey");
        let cfg = Config {
            api_key: "config-key".into(),
            ..Default::default()
        };
        // No env → config value.
        assert_eq!(cfg.resolve_api_key(), "config-key");
        // Env wins.
        std::env::set_var(ENV_API_KEY, "env-key");
        assert_eq!(cfg.resolve_api_key(), "env-key");
    }

    #[test]
    fn resolve_openrouter_api_key_env_only() {
        let _s = Scope::new("orkey");
        let cfg = Config {
            api_key: "config-key".into(),
            ..Default::default()
        };
        assert_eq!(cfg.resolve_openrouter_api_key(), "");
        std::env::set_var(ENV_OPENROUTER_API_KEY, "or-env");
        assert_eq!(cfg.resolve_openrouter_api_key(), "or-env");
    }

    #[test]
    fn resolve_output_dir_precedence() {
        let _s = Scope::new("outdir");
        // config value used when no env.
        let cfg = Config {
            default_output_dir: "/cfg/out".into(),
            ..Default::default()
        };
        assert_eq!(cfg.resolve_output_dir(), "/cfg/out");
        // env wins.
        std::env::set_var(ENV_OUTPUT_DIR, "/env/out");
        assert_eq!(cfg.resolve_output_dir(), "/env/out");
        std::env::remove_var(ENV_OUTPUT_DIR);
        // no env, no config → XDG default (non-empty when HOME set).
        let cfg = Config::default();
        let got = cfg.resolve_output_dir();
        assert!(
            got.ends_with(".local/share/naba/images") || got.is_empty(),
            "got {got}"
        );
    }

    #[test]
    fn to_config_defaults_maps_provider_and_resolved_model() {
        // quality → resolved model tier; provider passed through.
        let cfg = Config {
            provider: "gemini".into(),
            quality: "high".into(),
            ..Default::default()
        };
        let defaults = cfg.to_config_defaults().unwrap();
        assert_eq!(defaults.provider.as_deref(), Some("gemini"));
        assert_eq!(defaults.model.as_deref(), Some(gemini::PRO_MODEL));
        // empty config → both None.
        let defaults = Config::default().to_config_defaults().unwrap();
        assert_eq!(defaults.provider, None);
        assert_eq!(defaults.model, None);
    }

    #[test]
    fn get_value_unset_is_err_008_exit_1() {
        let _s = Scope::new("get-unset");
        let err = get_value("model").unwrap_err();
        assert_eq!(err.code, exit::GENERAL);
        assert_eq!(err.message, "key \"model\" is not set\n\nValid keys: api_key, model, provider, default_output_dir, aspect, resolution, quality");
    }

    #[test]
    fn get_value_returns_set_value() {
        let _s = Scope::new("get-set");
        set_value("model", "gemini-3-pro-image").unwrap();
        assert_eq!(get_value("model").unwrap(), "gemini-3-pro-image");
    }

    #[test]
    fn set_value_unknown_key_is_err_009_exit_2() {
        let _s = Scope::new("set-unknown");
        let err = set_value("bogus", "v").unwrap_err();
        assert_eq!(err.code, exit::USAGE);
        assert_eq!(err.message, "unknown key \"bogus\"\n\nValid keys: api_key, model, provider, default_output_dir, aspect, resolution, quality");
    }

    #[test]
    fn set_value_then_get_value_round_trip_via_disk() {
        let _s = Scope::new("cmd-roundtrip");
        set_value("provider", "openrouter").unwrap();
        set_value("aspect", "16:9").unwrap();
        assert_eq!(get_value("provider").unwrap(), "openrouter");
        assert_eq!(get_value("aspect").unwrap(), "16:9");
    }

    // -------------------------------------------------------------------
    // Issue 3.2 — config auto-migration (SPEC-MIGRATE-001..004).
    // -------------------------------------------------------------------

    /// Write raw bytes to `config.yaml` in the current NABA_CONFIG_DIR (creating the dir).
    fn write_config_raw(contents: &str) -> PathBuf {
        let path = config_path();
        std::fs::create_dir_all(config_dir()).unwrap();
        std::fs::write(&path, contents.as_bytes()).unwrap();
        path
    }

    /// Synthetic "structural migration required" predicate for tests: fires while an old-shape
    /// `legacy_marker` key is present. The round-trip through [`Config`] drops that unknown
    /// key, so a second run sees the new shape and no-ops — exercising real idempotency.
    fn needs_legacy_marker(v: &serde_norway::Value) -> bool {
        v.get("legacy_marker").is_some()
    }

    // SPEC-MIGRATE-001: loading an old 6-key (pre-`provider`) config performs ZERO rewrite —
    // the on-disk file is byte-identical afterward, and absent keys resolve to defaults.
    #[test]
    fn load_old_six_key_config_is_byte_identical_and_defaults_resolve() {
        let _s = Scope::new("migrate-byte-identical");
        // Old config: the 6 pre-`provider` keys, NO `provider`, NO `model`, plus comments and
        // hand-authored formatting a serde round-trip would flatten.
        let original = "# naba config (hand-edited)\napi_key: sk-old-key   # inline comment\ndefault_output_dir: /tmp/naba-out\naspect: '16:9'\nresolution: 2K\nquality: high\n";
        let path = write_config_raw(original);
        let before = std::fs::read(&path).unwrap();

        let cfg = Config::load().unwrap();

        // Zero-rewrite: bytes unchanged, comments preserved, no `.bak` created.
        let after = std::fs::read(&path).unwrap();
        assert_eq!(before, after, "load() must not rewrite an old config");
        assert!(
            !backup_path(&path).exists(),
            "no .bak on a zero-rewrite load"
        );

        // Absent `provider`/`model` resolve to defaults on read.
        assert_eq!(cfg.provider, "");
        assert_eq!(cfg.model, "");
        assert_eq!(cfg.resolve_model().unwrap(), gemini::PRO_MODEL); // via quality: high
                                                                     // Present keys still loaded.
        assert_eq!(cfg.api_key, "sk-old-key");
        assert_eq!(cfg.quality, "high");
    }

    // SPEC-MIGRATE-001/003: a comment-bearing config survives a normal load untouched.
    #[test]
    fn config_with_comments_not_rewritten_on_normal_load() {
        let _s = Scope::new("migrate-comments");
        let original = "# top comment\nprovider: openrouter\nmodel: gemini-3-pro-image  # keep me\n# trailing comment\n";
        let path = write_config_raw(original);
        let before = std::fs::read(&path).unwrap();

        let _ = Config::load().unwrap();

        assert_eq!(
            std::fs::read(&path).unwrap(),
            before,
            "comments must survive a load"
        );
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("# keep me"), "inline comment preserved");
    }

    // SPEC-MIGRATE-002: when a structural rewrite IS triggered (synthetic), the engine backs
    // the original up to `<path>.bak`, rewrites via serde round-trip, and is idempotent.
    #[test]
    fn structural_migration_backs_up_rewrites_and_is_idempotent() {
        let _s = Scope::new("migrate-structural");
        let original = "# will be lost on rewrite (SPEC-MIGRATE-003)\nlegacy_marker: true\napi_key: sk-x\nmodel: gemini-3-pro-image\nquality: high\n";
        let path = write_config_raw(original);
        let orig_bytes = std::fs::read(&path).unwrap();

        // First run: rewrite happens.
        migrate_file(&path, needs_legacy_marker).unwrap();

        let bak = backup_path(&path);
        assert!(bak.exists(), ".bak must be written before the rewrite");
        assert_eq!(
            std::fs::read(&bak).unwrap(),
            orig_bytes,
            ".bak holds the original bytes"
        );

        let rewritten = std::fs::read_to_string(&path).unwrap();
        assert!(
            !rewritten.contains("legacy_marker"),
            "unknown old-shape key dropped"
        );
        assert!(
            !rewritten.contains("# will be lost"),
            "comments lost on rewrite (accepted)"
        );
        // Round-trip preserved the real config keys.
        let cfg: Config = serde_norway::from_str(&rewritten).unwrap();
        assert_eq!(cfg.api_key, "sk-x");
        assert_eq!(cfg.model, "gemini-3-pro-image");
        assert_eq!(cfg.quality, "high");

        // Second run: new shape has no marker → no-op, file unchanged, .bak not clobbered.
        let after_first = std::fs::read(&path).unwrap();
        migrate_file(&path, needs_legacy_marker).unwrap();
        assert_eq!(
            std::fs::read(&path).unwrap(),
            after_first,
            "second run is a no-op"
        );
        assert_eq!(
            std::fs::read(&bak).unwrap(),
            orig_bytes,
            ".bak preserved, not clobbered"
        );
    }

    // SPEC-MIGRATE-002: graceful on missing / empty / malformed / already-new inputs.
    #[test]
    fn structural_migration_is_graceful_on_edge_inputs() {
        let _s = Scope::new("migrate-graceful");
        let path = config_path();
        std::fs::create_dir_all(config_dir()).unwrap();

        // Missing file → no-op, no .bak, no crash.
        let _ = std::fs::remove_file(&path);
        migrate_file(&path, needs_legacy_marker).unwrap();
        assert!(!backup_path(&path).exists());
        assert!(!path.exists());

        // Empty file → no-op, still empty, no .bak.
        std::fs::write(&path, b"").unwrap();
        migrate_file(&path, needs_legacy_marker).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"");
        assert!(!backup_path(&path).exists());

        // Malformed YAML → no-op (skipped), file unchanged, no .bak.
        let bad = "api_key: [unterminated\nlegacy_marker: true\n";
        std::fs::write(&path, bad.as_bytes()).unwrap();
        migrate_file(&path, needs_legacy_marker).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), bad);
        assert!(!backup_path(&path).exists());

        // Already-new (no marker) → no-op even when a rewrite predicate is supplied.
        let good = "provider: gemini\nmodel: gemini-3-pro-image\n";
        std::fs::write(&path, good.as_bytes()).unwrap();
        migrate_file(&path, needs_legacy_marker).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), good);
        assert!(!backup_path(&path).exists());
    }

    // SPEC-MIGRATE-002: a rewrite preserves all 7 keys through the serde round-trip.
    #[test]
    fn structural_migration_round_trip_preserves_all_seven_keys() {
        let _s = Scope::new("migrate-seven-keys");
        let original = "legacy_marker: true\napi_key: sk-abc\nmodel: gemini-3-pro-image\nprovider: openrouter\ndefault_output_dir: /tmp/out\naspect: '16:9'\nresolution: 2K\nquality: high\n";
        let path = write_config_raw(original);

        migrate_file(&path, needs_legacy_marker).unwrap();

        let cfg: Config = serde_norway::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(
            cfg,
            Config {
                api_key: "sk-abc".into(),
                model: "gemini-3-pro-image".into(),
                provider: "openrouter".into(),
                default_output_dir: "/tmp/out".into(),
                aspect: "16:9".into(),
                resolution: "2K".into(),
                quality: "high".into(),
            }
        );
    }

    // The production predicate is a no-op: migrate_if_needed never rewrites the current schema.
    #[test]
    fn migrate_if_needed_is_zero_rewrite_for_current_schema() {
        let _s = Scope::new("migrate-noop");
        let original = "provider: gemini\nmodel: gemini-3-pro-image\n# comment\n";
        let path = write_config_raw(original);
        let before = std::fs::read(&path).unwrap();
        migrate_if_needed().unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), before);
        assert!(!backup_path(&path).exists());
    }
}
