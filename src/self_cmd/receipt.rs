//! Install receipt + from-build marker for `naba self` (SPEC-SELF-001, SPEC-DIRS-001).
//!
//! Two on-disk artifacts, both under the config dir ([`crate::dirs::config_dir`]):
//!
//! - **The cargo-dist receipt** (`naba-receipt.json`) — written by the vendor `curl|sh`
//!   installer, never by naba. naba only **reads** it, and only for the load-bearing
//!   `install_prefix` (the vendor prefix the source classifier needs). All other cargo-dist
//!   fields are tolerated and ignored (`serde` skips unknown keys by default) — in particular
//!   naba **never** branches on the receipt's `source` repo descriptor.
//! - **naba's own from-build marker** (`naba-from-build.json`) — written *only* by
//!   `naba self install --from-build` and removed by `self uninstall` / `self update --force`.
//!   Atomic temp+rename write.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::dirs;
use crate::error::{AppError, AppResult};

/// The subset of the cargo-dist receipt naba reads (SPEC-SELF-001). Unknown keys — `binaries`,
/// `binary_aliases`, `modify_path`, `provider`, `source`, `install_layout`, … — are tolerated
/// and ignored. `#[serde(default)]` makes every read field optional so a schema drift never
/// fails the parse.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Receipt {
    /// The vendor install prefix, e.g. `~/.local/bin` (load-bearing for classification).
    #[serde(default)]
    pub install_prefix: String,
    /// The installed version string (informational).
    #[serde(default)]
    pub version: String,
}

impl Receipt {
    /// Load the receipt from the default path ([`crate::dirs::receipt_path`]). A missing file is
    /// `Ok(None)` (no vendor install); a read or JSON error surfaces as [`AppError`].
    pub fn load() -> AppResult<Option<Receipt>> {
        Self::load_from(&dirs::receipt_path())
    }

    /// Load the receipt from an explicit path (test seam). Missing → `Ok(None)`.
    pub fn load_from(path: &Path) -> AppResult<Option<Receipt>> {
        let data = match std::fs::read_to_string(path) {
            Ok(d) => d,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(AppError::file_io(format!("read receipt: {e}"))),
        };
        let receipt: Receipt = serde_json::from_str(&data)
            .map_err(|e| AppError::general(format!("parse receipt: {e}")))?;
        Ok(Some(receipt))
    }

    /// The canonicalized vendor install prefix, or `None` when `install_prefix` is empty.
    ///
    /// Expands a leading `~/` (or bare `~`) against `$HOME`, then `std::fs::canonicalize` so the
    /// classifier compares two canonical paths (symlink-safe, SPEC-SELF-002). Canonicalization is
    /// best-effort: if the prefix does not resolve on disk, the expanded (un-canonicalized) path
    /// is returned so classification still has a prefix to test.
    pub fn canonical_install_prefix(&self) -> Option<PathBuf> {
        if self.install_prefix.is_empty() {
            return None;
        }
        let expanded = expand_tilde(&self.install_prefix);
        Some(std::fs::canonicalize(&expanded).unwrap_or(expanded))
    }
}

/// naba's from-build marker (SPEC-SELF-001), written only by `self install --from-build`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FromBuildMarker {
    /// Always `"from-build"` — the discriminator.
    pub source: String,
    /// The version that was installed (`crate::version::VERSION` at install time).
    pub version: String,
    /// The build profile (e.g. `"release"`, `"debug"`).
    pub profile: String,
}

impl FromBuildMarker {
    /// A marker for the given version/profile with `source = "from-build"`.
    pub fn new(version: &str, profile: &str) -> Self {
        Self {
            source: "from-build".to_string(),
            version: version.to_string(),
            profile: profile.to_string(),
        }
    }

    /// Whether the from-build marker exists at the default path.
    pub fn exists() -> bool {
        dirs::from_build_marker_path().is_file()
    }

    /// Read the marker from the default path (`Ok(None)` when absent).
    pub fn load() -> AppResult<Option<FromBuildMarker>> {
        Self::load_from(&dirs::from_build_marker_path())
    }

    /// Read the marker from an explicit path (test seam).
    pub fn load_from(path: &Path) -> AppResult<Option<FromBuildMarker>> {
        let data = match std::fs::read_to_string(path) {
            Ok(d) => d,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(AppError::file_io(format!("read from-build marker: {e}"))),
        };
        let marker = serde_json::from_str(&data)
            .map_err(|e| AppError::general(format!("parse from-build marker: {e}")))?;
        Ok(Some(marker))
    }

    /// Write the marker to the default path, atomically (temp + rename).
    pub fn write(&self) -> AppResult<()> {
        self.write_to(&dirs::from_build_marker_path())
    }

    /// Write the marker to an explicit path, atomically (test seam). Creates the parent dir.
    pub fn write_to(&self, path: &Path) -> AppResult<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| AppError::file_io(format!("mkdir for marker: {e}")))?;
        }
        let data = serde_json::to_string_pretty(self)
            .map_err(|e| AppError::general(format!("serialize marker: {e}")))?;
        // Atomic: write a sibling temp file then rename over the target.
        let tmp = tmp_sibling(path);
        std::fs::write(&tmp, data.as_bytes())
            .map_err(|e| AppError::file_io(format!("write marker tmp: {e}")))?;
        std::fs::rename(&tmp, path).map_err(|e| {
            let _ = std::fs::remove_file(&tmp);
            AppError::file_io(format!("rename marker: {e}"))
        })?;
        Ok(())
    }

    /// Remove the marker at the default path (idempotent — absent is `Ok`).
    pub fn remove() -> AppResult<()> {
        Self::remove_from(&dirs::from_build_marker_path())
    }

    /// Remove the marker at an explicit path (idempotent).
    pub fn remove_from(path: &Path) -> AppResult<()> {
        match std::fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(AppError::file_io(format!("remove marker: {e}"))),
        }
    }
}

/// Expand a leading `~/` or bare `~` against `$HOME`. Other paths pass through unchanged.
fn expand_tilde(p: &str) -> PathBuf {
    if p == "~" {
        return home();
    }
    if let Some(rest) = p.strip_prefix("~/") {
        return home().join(rest);
    }
    PathBuf::from(p)
}

fn home() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default()
}

/// A `<path>.tmp-<pid>` sibling for the atomic-rename write.
fn tmp_sibling(path: &Path) -> PathBuf {
    let mut s = path.as_os_str().to_owned();
    s.push(format!(".tmp-{}", std::process::id()));
    PathBuf::from(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("naba-receipt-test-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    // A real-shape cargo-dist receipt: naba reads install_prefix + version and tolerates the rest.
    const REAL_RECEIPT: &str = r#"{
  "binaries": ["naba"],
  "binary_aliases": {},
  "install_prefix": "PREFIX_PLACEHOLDER",
  "install_layout": "flat",
  "modify_path": true,
  "provider": { "source": "cargo-dist", "version": "0.32.0" },
  "source": { "app_name": "naba", "name": "naba", "owner": "dixson3", "release_type": "github" },
  "version": "0.1.0"
}"#;

    #[test]
    fn load_missing_receipt_is_none() {
        let d = scratch("missing");
        assert!(Receipt::load_from(&d.join("naba-receipt.json")).unwrap().is_none());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn parse_real_shape_receipt_reads_prefix_and_version() {
        let d = scratch("real");
        let prefix = d.join("bin");
        std::fs::create_dir_all(&prefix).unwrap();
        let json = REAL_RECEIPT.replace("PREFIX_PLACEHOLDER", prefix.to_str().unwrap());
        let path = d.join("naba-receipt.json");
        std::fs::write(&path, json).unwrap();

        let r = Receipt::load_from(&path).unwrap().unwrap();
        assert_eq!(r.install_prefix, prefix.to_str().unwrap());
        assert_eq!(r.version, "0.1.0");
        // canonical prefix resolves to the (existing) real dir.
        let canon = r.canonical_install_prefix().unwrap();
        assert_eq!(canon, std::fs::canonicalize(&prefix).unwrap());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn empty_prefix_yields_no_canonical() {
        let r = Receipt {
            install_prefix: String::new(),
            version: "x".into(),
        };
        assert!(r.canonical_install_prefix().is_none());
    }

    #[test]
    fn tilde_prefix_expands_against_home() {
        let prev = std::env::var_os("HOME");
        let d = scratch("tilde");
        std::env::set_var("HOME", &d);
        let sub = d.join(".local/bin");
        std::fs::create_dir_all(&sub).unwrap();
        let r = Receipt {
            install_prefix: "~/.local/bin".into(),
            version: String::new(),
        };
        let canon = r.canonical_install_prefix().unwrap();
        assert_eq!(canon, std::fs::canonicalize(&sub).unwrap());
        match prev {
            Some(v) => std::env::set_var("HOME", v),
            None => std::env::remove_var("HOME"),
        }
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn nonexistent_prefix_falls_back_to_expanded() {
        let r = Receipt {
            install_prefix: "/no/such/prefix/naba-bin".into(),
            version: String::new(),
        };
        // Canonicalize fails → expanded path returned unchanged.
        assert_eq!(
            r.canonical_install_prefix().unwrap(),
            PathBuf::from("/no/such/prefix/naba-bin")
        );
    }

    #[test]
    fn from_build_marker_atomic_round_trip() {
        let d = scratch("marker");
        let path = d.join("sub/naba-from-build.json"); // parent created by write_to
        let m = FromBuildMarker::new("0.2.0", "release");
        m.write_to(&path).unwrap();
        assert!(path.is_file());
        // No stray temp file left behind.
        let leftovers: Vec<_> = std::fs::read_dir(path.parent().unwrap())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains(".tmp-"))
            .collect();
        assert!(leftovers.is_empty(), "atomic write left a temp file");

        let read = FromBuildMarker::load_from(&path).unwrap().unwrap();
        assert_eq!(read, m);
        assert_eq!(read.source, "from-build");

        // Remove is idempotent.
        FromBuildMarker::remove_from(&path).unwrap();
        assert!(!path.exists());
        FromBuildMarker::remove_from(&path).unwrap(); // second remove: still Ok
        let _ = std::fs::remove_dir_all(&d);
    }
}
