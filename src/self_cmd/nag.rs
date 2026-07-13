//! Throttled upgrade nag for `naba self` (SPEC-SELF-006).
//!
//! A best-effort, offline, one-line stderr nudge fired from `version` / `doctor` when the
//! cached update-check says a newer release exists **and** the running binary is a nag-eligible
//! (vendor) install. No network on this path — it only reads the [`super::update_check`] cache.
//!
//! Suppressed entirely when `NABA_NO_UPDATE_CHECK` or `CI` is set. Throttled to at most once per
//! [`NAG_INTERVAL_SECS`] via the cache's `nagged_at` field.

use super::source;
use super::update::{detect_source, is_newer};
use super::update_check::{now_secs, UpdateCheck};
use crate::version;

/// Minimum interval between nags (24h).
pub const NAG_INTERVAL_SECS: u64 = 24 * 60 * 60;

/// Whether the nag is suppressed by environment (`NABA_NO_UPDATE_CHECK` or `CI` set non-empty).
fn suppressed() -> bool {
    non_empty_env("NABA_NO_UPDATE_CHECK") || non_empty_env("CI")
}

fn non_empty_env(k: &str) -> bool {
    std::env::var_os(k).map(|v| !v.is_empty()).unwrap_or(false)
}

/// Pure throttle+availability decision (SPEC-SELF-006): nag iff a newer release is cached and the
/// throttle interval has elapsed.
pub fn should_nag(cache: &UpdateCheck, current: &str, now: u64, interval: u64) -> bool {
    is_newer(&cache.latest_version, current) && now.saturating_sub(cache.nagged_at) >= interval
}

/// Fire the nag if eligible (best-effort; any error is swallowed). Called from `version`/`doctor`.
pub fn maybe_nag() {
    if suppressed() {
        return;
    }
    // Only vendor installs are nagged.
    match detect_source() {
        Ok(s) if source::nag_eligible(s) => {}
        _ => return,
    }
    let Ok(Some(mut cache)) = UpdateCheck::load() else {
        return;
    };
    let now = now_secs();
    if !should_nag(&cache, version::VERSION, now, NAG_INTERVAL_SECS) {
        return;
    }
    eprintln!(
        "A new naba release is available: {} -> {}. Run `naba self update`.",
        version::VERSION,
        cache.latest_version
    );
    cache.nagged_at = now;
    let _ = cache.save();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cache(latest: &str, nagged_at: u64) -> UpdateCheck {
        UpdateCheck {
            checked_at: 1,
            latest_version: latest.into(),
            current_version: "0.1.0".into(),
            nagged_at,
        }
    }

    #[test]
    fn nags_when_newer_and_interval_elapsed() {
        // latest > current, last nag long ago → nag.
        assert!(should_nag(
            &cache("0.2.0", 0),
            "0.1.0",
            100_000,
            NAG_INTERVAL_SECS
        ));
    }

    #[test]
    fn no_nag_when_up_to_date() {
        assert!(!should_nag(
            &cache("0.1.0", 0),
            "0.1.0",
            100_000,
            NAG_INTERVAL_SECS
        ));
        assert!(!should_nag(
            &cache("", 0),
            "0.1.0",
            100_000,
            NAG_INTERVAL_SECS
        ));
    }

    #[test]
    fn throttled_within_interval() {
        // Newer available but nagged recently → suppressed.
        let now = 100_000;
        assert!(!should_nag(
            &cache("0.2.0", now - 10),
            "0.1.0",
            now,
            NAG_INTERVAL_SECS
        ));
        // Just past the interval → nag again.
        assert!(should_nag(
            &cache("0.2.0", now - NAG_INTERVAL_SECS),
            "0.1.0",
            now,
            NAG_INTERVAL_SECS
        ));
    }
}
