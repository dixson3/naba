//! `naba skills preflight` — a fast skill-gate (SPEC-PREFLIGHT-001..).
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
//! ([`crate::skills::resolve_dest`]).

use crate::commands::Globals;
use crate::error::AppResult;

/// Resolved destination flags (mirror `skills`/`doctor`).
#[derive(Debug, Clone)]
pub struct Opts {
    pub scope: String,
    pub surface: String,
    pub target: String,
}

/// Run the preflight gate and report (JSON or human), setting the exit status. The axis
/// computation lands in Issue C.2; C.1 wires the surface + shared resolution.
pub fn run(opts: &Opts, globals: &Globals) -> AppResult<()> {
    let outcome = compute(opts);
    report(&outcome, globals.json)
}

/// The overall preflight outcome (axes filled in C.2).
struct Outcome {
    status: &'static str,
}

/// Compute the gate. C.1 skeleton: resolve the destination (shared with `skills`/`doctor`) and
/// return `ok`; C.2 fills the three axes.
fn compute(opts: &Opts) -> Outcome {
    let _dest = crate::skills::resolve_dest(&opts.scope, &opts.surface, &opts.target);
    Outcome { status: "ok" }
}

/// Emit the envelope (JSON or human) and return a non-zero error on a non-`ok` status.
fn report(outcome: &Outcome, json: bool) -> AppResult<()> {
    if json {
        let obj = serde_json::json!({
            "command": "skills preflight",
            "status": outcome.status,
        });
        println!("{}", serde_json::to_string_pretty(&obj).unwrap_or_default());
    } else {
        println!("skills preflight: {}", outcome.status);
    }
    if outcome.status != "ok" {
        return Err(crate::error::AppError::general(format!(
            "skills preflight: {}",
            outcome.status
        )));
    }
    Ok(())
}
