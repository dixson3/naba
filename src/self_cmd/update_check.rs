//! Update-check cache for `naba self` (SPEC-SELF-006, SPEC-PREFLIGHT).
//!
//! A small JSON cache at `~/.cache/naba/update-check.json` ([`crate::dirs::update_check_path`])
//! recording the last known latest release. It is populated **out-of-band** (by an explicit
//! `naba self update --check`, not shown here) and read on the hot path by:
//!
//! - the throttled upgrade [`nag`](super::nag), and
//! - the `skills preflight` binary-up-to-date axis (Epic C).
//!
//! The cache is **absent by default** on a fresh install (no release has been fetched yet), so
//! preflight treats an absent/stale cache as the non-blocking `unknown` tri-state.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::dirs;
use crate::error::{AppError, AppResult};

/// Default staleness horizon for the cache (24h). A cache older than this reads `unknown`.
pub const DEFAULT_TTL_SECS: u64 = 24 * 60 * 60;

/// The persisted update-check state.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateCheck {
    /// Unix seconds when the check last ran.
    #[serde(default)]
    pub checked_at: u64,
    /// The latest release version seen (e.g. `"0.2.0"`), empty if unknown.
    #[serde(default)]
    pub latest_version: String,
    /// The running version at check time (informational).
    #[serde(default)]
    pub current_version: String,
    /// Unix seconds when the upgrade nag last fired (throttle state).
    #[serde(default)]
    pub nagged_at: u64,
}

impl UpdateCheck {
    /// Load from the default cache path (`Ok(None)` when absent).
    pub fn load() -> AppResult<Option<Self>> {
        Self::load_from(&dirs::update_check_path())
    }

    /// Load from an explicit path (test seam). Missing → `Ok(None)`; a parse error → `Ok(None)`
    /// (a corrupt cache is treated as absent, never fatal on the hot path).
    pub fn load_from(path: &Path) -> AppResult<Option<Self>> {
        let data = match std::fs::read_to_string(path) {
            Ok(d) => d,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(AppError::file_io(format!("read update-check: {e}"))),
        };
        Ok(serde_json::from_str(&data).ok())
    }

    /// Persist to the default cache path (atomic temp+rename; creates the cache dir).
    pub fn save(&self) -> AppResult<()> {
        self.save_to(&dirs::update_check_path())
    }

    /// Persist to an explicit path (test seam), atomically.
    pub fn save_to(&self, path: &Path) -> AppResult<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| AppError::file_io(format!("mkdir cache: {e}")))?;
        }
        let data = serde_json::to_string_pretty(self)
            .map_err(|e| AppError::general(format!("serialize update-check: {e}")))?;
        let mut tmp = path.as_os_str().to_owned();
        tmp.push(format!(".tmp-{}", std::process::id()));
        let tmp = PathBuf::from(tmp);
        std::fs::write(&tmp, data.as_bytes())
            .map_err(|e| AppError::file_io(format!("write cache tmp: {e}")))?;
        std::fs::rename(&tmp, path).map_err(|e| {
            let _ = std::fs::remove_file(&tmp);
            AppError::file_io(format!("rename cache: {e}"))
        })?;
        Ok(())
    }

    /// Whether the cache is older than `ttl` seconds relative to `now` (or never checked).
    pub fn is_stale(&self, ttl: u64, now: u64) -> bool {
        self.checked_at == 0 || now.saturating_sub(self.checked_at) >= ttl
    }
}

/// Current wall-clock time in unix seconds (0 on the impossible pre-epoch error).
pub fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_missing_is_none() {
        let d = std::env::temp_dir().join(format!("naba-uc-miss-{}", std::process::id()));
        assert!(UpdateCheck::load_from(&d.join("update-check.json"))
            .unwrap()
            .is_none());
    }

    #[test]
    fn save_then_load_round_trips() {
        let d = std::env::temp_dir().join(format!("naba-uc-rt-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        let path = d.join("update-check.json");
        let uc = UpdateCheck {
            checked_at: 1000,
            latest_version: "0.2.0".into(),
            current_version: "0.1.0".into(),
            nagged_at: 500,
        };
        uc.save_to(&path).unwrap();
        assert_eq!(UpdateCheck::load_from(&path).unwrap().unwrap(), uc);
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn staleness() {
        let uc = UpdateCheck {
            checked_at: 1000,
            ..Default::default()
        };
        assert!(!uc.is_stale(100, 1050)); // within ttl
        assert!(uc.is_stale(100, 1200)); // beyond ttl
                                         // never-checked is always stale.
        assert!(UpdateCheck::default().is_stale(100, 50));
    }

    #[test]
    fn corrupt_cache_reads_as_absent() {
        let d = std::env::temp_dir().join(format!("naba-uc-bad-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        let path = d.join("update-check.json");
        std::fs::write(&path, b"{not json").unwrap();
        assert!(UpdateCheck::load_from(&path).unwrap().is_none());
        let _ = std::fs::remove_dir_all(&d);
    }
}
