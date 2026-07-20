//! `naba skills preflight` — a fast skill-gate (SPEC-PREFLIGHT-001..004).
//!
//! Mirrors yoshiko-flow's `yf preflight <skill>` but with an added **auth axis** (naba's
//! deliberate divergence from yf, which validates no API keys). A skill invocation calls
//! `naba skills preflight --json` at trigger time to confirm the environment is ready:
//!
//! 1. **auth** — the effective provider's key is present (offline; no network on the hot path).
//! 2. **skills up-to-date** — the on-disk embedded skills match this binary (`embed::skill_status`).
//! 3. **binary up-to-date** — a **tri-state** (`up_to_date | update_available | unknown`) read from
//!    the `~/.cache/naba/update-check.json` cache. Absent/stale → `unknown`, which is
//!    **non-blocking** (a fresh install has no cache yet, so preflight must still pass).
//!
//! Overall `status` is `ok` unless auth or skills fails; the binary axis never blocks. Exit code
//! is non-zero on any non-`ok` status (doctor/preflight convention).
//!
//! The scope/surface/target destination resolution is shared with `skills`/`doctor`
//! ([`crate::skills::resolve_dest`]); the provider resolution is shared with `doctor`
//! ([`crate::doctor::resolve_provider`]).

use crate::commands::Globals;
use crate::config::Config;
use crate::embed::{self, SkillStatus};
use crate::error::{AppError, AppResult};
use crate::self_cmd::update::is_newer;
use crate::self_cmd::update_check::{now_secs, UpdateCheck, DEFAULT_TTL_SECS};
use crate::version;

/// Resolved destination flags (mirror `skills`/`doctor`).
#[derive(Debug, Clone)]
pub struct Opts {
    pub scope: String,
    pub harness: String,
    pub target: String,
}

// ---- axes ---------------------------------------------------------------------------------

/// The offline auth axis: provider key-present.
struct AuthAxis {
    ok: bool,
    provider: String,
    detail: String,
}

/// The skills-up-to-date axis over every embedded skill.
struct SkillsAxis {
    ok: bool,
    detail: String,
    per_skill: Vec<(String, SkillStatus)>,
}

/// The tri-state binary-up-to-date axis.
struct BinaryAxis {
    /// `up_to_date | update_available | unknown`.
    status: &'static str,
    detail: String,
    latest: Option<String>,
}

/// Auth axis from resolved inputs (pure). `ok` iff the key is non-empty.
fn auth_axis(provider: &str, key: &str, key_name: &str) -> AuthAxis {
    if key.is_empty() {
        AuthAxis {
            ok: false,
            provider: provider.to_string(),
            detail: format!("{key_name} not set (env or config); generation will fail"),
        }
    } else {
        AuthAxis {
            ok: true,
            provider: provider.to_string(),
            detail: "present".to_string(),
        }
    }
}

/// Skills axis from per-skill statuses (pure). `ok` iff every skill is installed, up-to-date,
/// complete, and unmodified.
fn skills_axis(per_skill: Vec<(String, SkillStatus)>) -> SkillsAxis {
    let mut ok = true;
    let mut detail = "installed, up-to-date, complete, unmodified".to_string();
    for (name, st) in &per_skill {
        if !st.installed {
            ok = false;
            detail = format!("{name}: not installed (run: naba skills install)");
            break;
        } else if !st.up_to_date {
            ok = false;
            detail = format!("{name}: outdated vs this binary (run: naba skills upgrade)");
            break;
        } else if !st.complete {
            ok = false;
            detail = format!("{name}: missing files (run: naba skills upgrade)");
            break;
        } else if !st.unmodified {
            ok = false;
            detail = format!("{name}: modified since install (run: naba skills upgrade)");
            break;
        }
    }
    if per_skill.is_empty() {
        ok = false;
        detail = "no embedded skills".to_string();
    }
    SkillsAxis {
        ok,
        detail,
        per_skill,
    }
}

/// Binary axis from the update-check cache (pure). Absent/stale → non-blocking `unknown`.
fn binary_axis(cache: Option<&UpdateCheck>, current: &str, now: u64, ttl: u64) -> BinaryAxis {
    match cache {
        None => BinaryAxis {
            status: "unknown",
            detail: "no update-check cache yet (non-blocking)".to_string(),
            latest: None,
        },
        Some(c) if c.is_stale(ttl, now) => BinaryAxis {
            status: "unknown",
            detail: "update-check cache is stale (non-blocking)".to_string(),
            latest: (!c.latest_version.is_empty()).then(|| c.latest_version.clone()),
        },
        Some(c) if is_newer(&c.latest_version, current) => BinaryAxis {
            status: "update_available",
            detail: format!("{current} -> {} (run `naba self update`)", c.latest_version),
            latest: Some(c.latest_version.clone()),
        },
        Some(c) => BinaryAxis {
            status: "up_to_date",
            detail: "binary is current".to_string(),
            latest: (!c.latest_version.is_empty()).then(|| c.latest_version.clone()),
        },
    }
}

/// Derive the overall status from the axes. Auth and skills block; the binary axis never does.
fn overall_status(auth: &AuthAxis, skills: &SkillsAxis) -> &'static str {
    if !auth.ok {
        "auth_missing"
    } else if !skills.ok {
        "skills_outdated"
    } else {
        "ok"
    }
}

// ---- run ----------------------------------------------------------------------------------

/// Run the preflight gate and report (JSON or human), setting the exit status.
pub fn run(opts: &Opts, globals: &Globals) -> AppResult<()> {
    // Auth axis (offline, provider-aware — shared with doctor).
    let cfg = Config::load().unwrap_or_default();
    let provider = crate::doctor::resolve_provider(globals.provider.as_deref(), &cfg);
    let key = crate::doctor::provider_api_key(&provider, &cfg);
    let key_name = crate::doctor::provider_key_name(&provider);
    let auth = auth_axis(&provider, &key, key_name);

    // Skills axis (embed status against the resolved dest).
    let per_skill = match crate::skills::resolve_dest(&opts.scope, &opts.harness, &opts.target) {
        Ok(dest) => embed::skill_names()
            .into_iter()
            .map(|name| {
                let st = embed::skill_status(&name, &dest.join(&name));
                (name, st)
            })
            .collect(),
        Err(_) => Vec::new(),
    };
    let skills = skills_axis(per_skill);

    // Binary axis (tri-state from the update-check cache; absent/stale → unknown, non-blocking).
    let cache = UpdateCheck::load().unwrap_or(None);
    let binary = binary_axis(
        cache.as_ref(),
        version::VERSION,
        now_secs(),
        DEFAULT_TTL_SECS,
    );

    let status = overall_status(&auth, &skills);
    report(status, &auth, &skills, &binary, globals.json)?;
    if status != "ok" {
        return Err(AppError::general(format!("skills preflight: {status}")));
    }
    Ok(())
}

/// Emit the envelope (JSON or human).
fn report(
    status: &str,
    auth: &AuthAxis,
    skills: &SkillsAxis,
    binary: &BinaryAxis,
    json: bool,
) -> AppResult<()> {
    if json {
        let skills_list: Vec<serde_json::Value> = skills
            .per_skill
            .iter()
            .map(|(name, st)| {
                serde_json::json!({
                    "name": name,
                    "installed": st.installed,
                    "up_to_date": st.up_to_date,
                    "complete": st.complete,
                    "unmodified": st.unmodified,
                })
            })
            .collect();
        let obj = serde_json::json!({
            "command": "skills preflight",
            "status": status,
            "axes": {
                "auth": { "ok": auth.ok, "provider": auth.provider, "detail": auth.detail },
                "skills": { "ok": skills.ok, "detail": skills.detail, "skills": skills_list },
                "binary": { "status": binary.status, "detail": binary.detail, "latest": binary.latest },
            },
        });
        println!("{}", serde_json::to_string_pretty(&obj).unwrap_or_default());
    } else {
        println!("skills preflight: {status}");
        println!("  auth ({}): {}", auth.provider, auth.detail);
        println!("  skills: {}", skills.detail);
        println!("  binary [{}]: {}", binary.status, binary.detail);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn st(installed: bool, up: bool, complete: bool, unmod: bool) -> SkillStatus {
        SkillStatus {
            installed,
            up_to_date: up,
            complete,
            unmodified: unmod,
        }
    }

    #[test]
    fn auth_axis_key_present_vs_missing() {
        assert!(auth_axis("gemini", "sk-x", "GEMINI_API_KEY").ok);
        let a = auth_axis("gemini", "", "GEMINI_API_KEY");
        assert!(!a.ok);
        assert!(a.detail.contains("GEMINI_API_KEY not set"));
    }

    #[test]
    fn skills_axis_all_good_is_ok() {
        let s = skills_axis(vec![("naba".into(), st(true, true, true, true))]);
        assert!(s.ok);
    }

    #[test]
    fn skills_axis_reports_first_failure() {
        let s = skills_axis(vec![("naba".into(), st(true, false, true, true))]);
        assert!(!s.ok);
        assert!(s.detail.contains("outdated"));
        // empty set is not ok.
        assert!(!skills_axis(vec![]).ok);
    }

    // SPEC-PREFLIGHT: absent cache → binary axis "unknown", non-blocking.
    #[test]
    fn binary_axis_absent_cache_is_unknown() {
        let b = binary_axis(None, "0.1.0", 100, DEFAULT_TTL_SECS);
        assert_eq!(b.status, "unknown");
        assert!(b.latest.is_none());
    }

    #[test]
    fn binary_axis_stale_cache_is_unknown() {
        let c = UpdateCheck {
            checked_at: 1,
            latest_version: "0.2.0".into(),
            ..Default::default()
        };
        let b = binary_axis(
            Some(&c),
            "0.1.0",
            1 + DEFAULT_TTL_SECS + 1,
            DEFAULT_TTL_SECS,
        );
        assert_eq!(b.status, "unknown");
    }

    #[test]
    fn binary_axis_fresh_cache_reports_update_or_current() {
        let now = 1000;
        let fresh = |latest: &str| UpdateCheck {
            checked_at: now,
            latest_version: latest.into(),
            ..Default::default()
        };
        assert_eq!(
            binary_axis(Some(&fresh("0.2.0")), "0.1.0", now, DEFAULT_TTL_SECS).status,
            "update_available"
        );
        assert_eq!(
            binary_axis(Some(&fresh("0.1.0")), "0.1.0", now, DEFAULT_TTL_SECS).status,
            "up_to_date"
        );
    }

    // SPEC-PREFLIGHT: an absent cache (binary unknown) keeps the overall status ok when auth +
    // skills pass.
    #[test]
    fn overall_ok_when_auth_and_skills_pass_regardless_of_binary() {
        let auth = auth_axis("gemini", "sk-x", "GEMINI_API_KEY");
        let skills = skills_axis(vec![("naba".into(), st(true, true, true, true))]);
        assert_eq!(overall_status(&auth, &skills), "ok");
        // The binary axis being unknown does not change the overall status.
        let binary = binary_axis(None, "0.1.0", 100, DEFAULT_TTL_SECS);
        assert_eq!(binary.status, "unknown");
    }

    #[test]
    fn overall_blocks_on_auth_then_skills() {
        let no_auth = auth_axis("gemini", "", "GEMINI_API_KEY");
        let good_skills = skills_axis(vec![("naba".into(), st(true, true, true, true))]);
        assert_eq!(overall_status(&no_auth, &good_skills), "auth_missing");

        let ok_auth = auth_axis("gemini", "sk", "GEMINI_API_KEY");
        let bad_skills = skills_axis(vec![("naba".into(), st(false, false, false, false))]);
        assert_eq!(overall_status(&ok_auth, &bad_skills), "skills_outdated");
    }
}
