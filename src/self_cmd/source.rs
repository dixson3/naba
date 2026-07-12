//! Install-source classification for `naba self` (SPEC-SELF-002).
//!
//! Ports yoshiko-flow's `source.rs`. The authoritative signal is **path-primary**: the
//! canonicalized `current_exe()`, not the receipt. The receipt only supplies the vendor prefix.
//!
//! # Precedence: Homebrew > FromBuild > Vendor > Unknown
//!
//! - **Homebrew** — the exe path contains a `Cellar` component (macOS taps) or a `.linuxbrew`
//!   component (Linuxbrew). Homebrew installs are managed by `brew`; `self update` always
//!   refuses them.
//! - **FromBuild** — naba's own `naba-from-build.json` marker is present (written by
//!   `naba self install --from-build`).
//! - **Vendor** — the canonicalized exe path is under the canonicalized vendor prefix (from the
//!   cargo-dist receipt, typically `~/.local/bin`). This is the only auto-updatable source.
//! - **Unknown** — none of the above (e.g. a `cargo run` binary, a manually-copied binary).
//!
//! [`classify`] is pure over its inputs (exe path, vendor prefix, marker-present flag) so every
//! branch is unit-testable without touching the filesystem. The fs-composing detection (reading
//! `current_exe`, the receipt prefix, and the marker) lives in [`super::update`].

use std::path::Path;

/// How the running `naba` binary was installed (SPEC-SELF-002).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// Under the cargo-dist vendor prefix (`~/.local/bin`). Auto-updatable.
    Vendor,
    /// Managed by Homebrew (`Cellar`/`.linuxbrew` in the exe path). `brew upgrade` only.
    Homebrew,
    /// A `naba self install --from-build` install (marker present).
    FromBuild,
    /// Origin could not be determined.
    Unknown,
}

impl Source {
    /// The lowercase tag used in `--json` envelopes (`"vendor"`, `"homebrew"`, …).
    pub fn tag(self) -> &'static str {
        match self {
            Source::Vendor => "vendor",
            Source::Homebrew => "homebrew",
            Source::FromBuild => "from-build",
            Source::Unknown => "unknown",
        }
    }
}

/// Classify an install source from its raw signals (path-primary, SPEC-SELF-002).
///
/// - `exe` — the (ideally canonicalized) running-binary path.
/// - `vendor_prefix` — the canonicalized cargo-dist install prefix from the receipt, if any.
/// - `from_build` — whether the `naba-from-build.json` marker is present.
///
/// Precedence Homebrew > FromBuild > Vendor > Unknown.
pub fn classify(exe: &Path, vendor_prefix: Option<&Path>, from_build: bool) -> Source {
    if is_homebrew_path(exe) {
        return Source::Homebrew;
    }
    if from_build {
        return Source::FromBuild;
    }
    if let Some(prefix) = vendor_prefix {
        if !prefix.as_os_str().is_empty() && exe.starts_with(prefix) {
            return Source::Vendor;
        }
    }
    Source::Unknown
}

/// A path is a Homebrew install when it has a `Cellar` component (macOS taps live under
/// `<prefix>/Cellar/<formula>/…`) or a `.linuxbrew` component (Linuxbrew).
fn is_homebrew_path(exe: &Path) -> bool {
    exe.components().any(|c| {
        let s = c.as_os_str().to_string_lossy();
        s == "Cellar" || s == ".linuxbrew"
    })
}

/// Whether `self update` may auto-swap this source without `--force` (SPEC-SELF-003). Only
/// [`Source::Vendor`].
pub fn auto_updatable(source: Source) -> bool {
    matches!(source, Source::Vendor)
}

/// Whether the throttled upgrade nag should fire for this source (SPEC-SELF-006). Only
/// [`Source::Vendor`] — Homebrew/from-build/unknown users manage their own upgrades.
pub fn nag_eligible(source: Source) -> bool {
    matches!(source, Source::Vendor)
}

/// Human guidance for a refused `self update` (SPEC-SELF-003), by source.
pub fn refusal_guidance(source: Source) -> String {
    match source {
        Source::Homebrew => {
            "this naba was installed via Homebrew; run `brew upgrade naba` instead".to_string()
        }
        Source::FromBuild => {
            "this naba is a from-build install; re-run `naba self install --from-build` from an \
             updated checkout, or `naba self update --force` to pull the latest release"
                .to_string()
        }
        Source::Unknown => {
            "naba's install source could not be determined; re-run with `--force` to update anyway"
                .to_string()
        }
        Source::Vendor => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn homebrew_wins_over_everything() {
        // macOS Homebrew Cellar path — even with a from-build marker and a vendor prefix.
        let exe = PathBuf::from("/opt/homebrew/Cellar/naba/0.1.0/bin/naba");
        assert_eq!(
            classify(&exe, Some(Path::new("/opt/homebrew/Cellar")), true),
            Source::Homebrew
        );
        // Linuxbrew.
        let linux = PathBuf::from("/home/u/.linuxbrew/Cellar/naba/0.1.0/bin/naba");
        assert_eq!(classify(&linux, None, false), Source::Homebrew);
    }

    #[test]
    fn from_build_wins_over_vendor() {
        let exe = PathBuf::from("/home/u/.local/bin/naba");
        // Marker present → FromBuild even though the exe is under the vendor prefix.
        assert_eq!(
            classify(&exe, Some(Path::new("/home/u/.local/bin")), true),
            Source::FromBuild
        );
    }

    #[test]
    fn vendor_when_under_prefix_and_no_marker() {
        let exe = PathBuf::from("/home/u/.local/bin/naba");
        assert_eq!(
            classify(&exe, Some(Path::new("/home/u/.local/bin")), false),
            Source::Vendor
        );
    }

    #[test]
    fn unknown_when_outside_prefix_no_marker() {
        let exe = PathBuf::from("/tmp/scratch/naba");
        assert_eq!(
            classify(&exe, Some(Path::new("/home/u/.local/bin")), false),
            Source::Unknown
        );
        // No prefix at all → Unknown.
        assert_eq!(classify(&exe, None, false), Source::Unknown);
        // Empty prefix is ignored (not a match).
        assert_eq!(classify(&exe, Some(Path::new("")), false), Source::Unknown);
    }

    #[test]
    fn auto_updatable_and_nag_only_vendor() {
        for s in [Source::Homebrew, Source::FromBuild, Source::Unknown] {
            assert!(!auto_updatable(s));
            assert!(!nag_eligible(s));
        }
        assert!(auto_updatable(Source::Vendor));
        assert!(nag_eligible(Source::Vendor));
    }

    #[test]
    fn refusal_guidance_is_source_specific() {
        assert!(refusal_guidance(Source::Homebrew).contains("brew upgrade naba"));
        assert!(refusal_guidance(Source::FromBuild).contains("--from-build"));
        assert!(refusal_guidance(Source::Unknown).contains("--force"));
        assert!(refusal_guidance(Source::Vendor).is_empty());
    }

    #[test]
    fn tags_are_stable() {
        assert_eq!(Source::Vendor.tag(), "vendor");
        assert_eq!(Source::Homebrew.tag(), "homebrew");
        assert_eq!(Source::FromBuild.tag(), "from-build");
        assert_eq!(Source::Unknown.tag(), "unknown");
    }
}
