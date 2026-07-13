//! Build-injected version fields (SPEC-VERSION-BUILD-001).
//!
//! Both formats are exposed so callers can render them as SPEC requires:
//! - `version` subcommand (SPEC-VERSION-001): `naba <V> (commit: <C>, built: <D>)`
//! - `doctor` version check (SPEC-VERSION-002): `naba <V> (commit <C>, built <D>)`

/// `git describe --tags --always --dirty`, fallback `dev`.
pub const VERSION: &str = env!("NABA_VERSION");
/// `git rev-parse --short HEAD`, fallback `none`.
pub const COMMIT: &str = env!("NABA_COMMIT");
/// UTC build time.
pub const DATE: &str = env!("NABA_DATE");
/// The compile target triple (e.g. `aarch64-apple-darwin`), set from `$TARGET` in build.rs.
/// `naba self update` matches this against dist-manifest artifact `target_triples`.
pub const HOST_TRIPLE: &str = env!("NABA_HOST_TRIPLE");

/// SPEC-VERSION-001 format (with colons) — used by the `version` subcommand.
pub fn version_line() -> String {
    format!("naba {VERSION} (commit: {COMMIT}, built: {DATE})")
}

/// SPEC-VERSION-002 format (no colons) — used by the `doctor` version check.
pub fn doctor_version_line() -> String {
    format!("naba {VERSION} (commit {COMMIT}, built {DATE})")
}
