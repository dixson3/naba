//! `doctor` command (SPEC-DOCTOR-001..006, §3.9): environment health checks.
//!
//! Runs the checks in a fixed order (version, config, api_key, [model_config], api_live,
//! model_reachable, skills:<name>), builds a `Vec<DoctorCheck>`, and reports either the
//! human line format or the JSON envelope (SPEC-JSON-004). Exits 1 if any check fails
//! (SPEC-DOCTOR-005). Ports Go's `internal/cli/doctor.go`.
//!
//! # Provider-aware checks (SPEC-DOCTOR-006)
//!
//! The `api_key` / `api_live` / `model_reachable` checks resolve the *effective* provider
//! (CLI `--provider` > config `provider` > env-key autodetect) and probe that provider's
//! key + `list_models`. For the default gemini case the detail strings match Go verbatim;
//! the parity suite pins status + envelope shape (not exact wording) for the provider-aware
//! checks, so the openrouter analog can differ.

use crate::commands::Globals;
use crate::config::{self, Config};
use crate::embed;
use crate::error::{exit, AppError, AppResult};
use crate::output::{status, DoctorCheck, DoctorEnvelope};
use crate::provider::{self, build_provider, gemini, openrouter, Selection};
use crate::version;

/// Resolved destination flags for the doctor skills checks (mirror `skills`' scope/surface/target).
#[derive(Debug, Clone)]
pub struct Opts {
    pub scope: String,
    pub surface: String,
    pub target: String,
}

/// Entry point: run the checks, then report (human/JSON) and set the exit status.
pub async fn run(opts: &Opts, globals: &Globals) -> AppResult<()> {
    let checks = checks(opts, globals).await;
    // Throttled, offline upgrade nudge (SPEC-SELF-006) before the report; no-op unless a vendor
    // install has a cached newer release. Honors NABA_NO_UPDATE_CHECK/CI.
    crate::self_cmd::nag::maybe_nag();
    report(&checks, globals.json)
}

/// Run every health check in order and collect the results (Go's `doctorChecks`).
async fn checks(opts: &Opts, globals: &Globals) -> Vec<DoctorCheck> {
    let mut checks: Vec<DoctorCheck> = Vec::new();
    let mut add = |name: &str, st: &str, detail: String| {
        checks.push(DoctorCheck::new(name, st, detail));
    };

    // 1. Binary version (informational, SPEC-VERSION-002 — no colons).
    add("version", status::PASS, version::doctor_version_line());

    // 2. Config parseable.
    let cfg = match Config::load() {
        Ok(cfg) => {
            add(
                "config",
                status::PASS,
                format!("parseable ({})", config::config_path().display()),
            );
            cfg
        }
        Err(e) => {
            add("config", status::FAIL, format!("config not parseable: {e}"));
            Config::default()
        }
    };

    // Resolve the effective provider (SPEC-DOCTOR-006).
    let provider = resolve_provider(globals.provider.as_deref(), &cfg);
    let api_key = provider_api_key(&provider, &cfg);

    // 3. API key present (provider-aware).
    if api_key.is_empty() {
        add(
            "api_key",
            status::FAIL,
            format!(
                "{} not set (env or config); generation will fail",
                provider_key_name(&provider)
            ),
        );
    } else {
        add("api_key", status::PASS, "present".to_string());
    }

    // Resolve the model that would be used by default (config model/quality, else provider default).
    let model = match cfg.resolve_model() {
        Ok(m) if !m.is_empty() => m,
        Ok(_) => default_model(&provider),
        Err(e) => {
            add("model_config", status::FAIL, e.to_string());
            default_model(&provider)
        }
    };

    // 4 & 5. Live key check + model reachability (a single models.list call, no image cost).
    if !api_key.is_empty() {
        let sel = Selection {
            provider: provider.clone(),
            model: model.clone(),
            api_key: api_key.clone(),
            quality: None,
        };
        let client = build_provider(&sel);
        match client.list_models().await {
            Ok(available) => {
                add(
                    "api_live",
                    status::PASS,
                    "key validated via models.list".to_string(),
                );
                if model_reachable(&provider, &model, &available) {
                    add(
                        "model_reachable",
                        status::PASS,
                        format!("{model:?} is available"),
                    );
                } else {
                    add(
                        "model_reachable",
                        status::FAIL,
                        format!(
                            "configured model {model:?} is not in models.list (retired or wrong id)"
                        ),
                    );
                }
            }
            Err(e) if e.code == exit::AUTH => {
                add("api_live", status::FAIL, format!("key rejected: {e}"));
            }
            Err(e) => {
                // Network/transient error: degrade to presence-only rather than failing hard.
                add(
                    "api_live",
                    status::WARN,
                    format!("could not reach API (offline?): {e}"),
                );
                add(
                    "model_reachable",
                    status::WARN,
                    "skipped (API unreachable)".to_string(),
                );
            }
        }
    }

    // 6. Skills installed and matching the embedded binary.
    match crate::skills::resolve_dest(&opts.scope, &opts.surface, &opts.target) {
        Err(e) => add(
            "skills",
            status::FAIL,
            format!("cannot resolve skills destination: {e}"),
        ),
        Ok(dest) => {
            for name in embed::skill_names() {
                let st = embed::skill_status(&name, &dest.join(&name));
                let check = format!("skills:{name}");
                if !st.installed {
                    add(
                        &check,
                        status::FAIL,
                        format!(
                            "not installed at {} (run: naba skills install)",
                            dest.display()
                        ),
                    );
                } else if !st.up_to_date {
                    add(
                        &check,
                        status::FAIL,
                        "installed copy is outdated vs this binary (run: naba skills upgrade)"
                            .to_string(),
                    );
                } else if !st.complete {
                    add(
                        &check,
                        status::FAIL,
                        "installed copy is missing files (run: naba skills upgrade)".to_string(),
                    );
                } else if !st.unmodified {
                    add(
                        &check,
                        status::FAIL,
                        "installed copy was modified since install (run: naba skills upgrade)"
                            .to_string(),
                    );
                } else {
                    add(
                        &check,
                        status::PASS,
                        format!(
                            "installed, up-to-date, complete, unmodified ({})",
                            dest.display()
                        ),
                    );
                }
            }
        }
    }

    checks
}

/// Effective provider: CLI `--provider` > config `provider` > env-key autodetect.
///
/// `pub(crate)` shared surface: `skills preflight` (Epic C) reuses this and
/// [`provider_api_key`]/[`provider_key_name`] for its offline auth axis, so the two commands
/// resolve the provider identically (SPEC-DIRS/SPEC-PREFLIGHT).
pub(crate) fn resolve_provider(cli_provider: Option<&str>, cfg: &Config) -> String {
    if let Some(p) = cli_provider.filter(|s| !s.is_empty()) {
        return p.to_string();
    }
    if !cfg.provider.is_empty() {
        return cfg.provider.clone();
    }
    // Autodetect from resolved key presence (matches select::autodetect).
    let gemini = !cfg.resolve_api_key().is_empty();
    let openrouter = !cfg.resolve_openrouter_api_key().is_empty();
    match (gemini, openrouter) {
        (_, true) => provider::select::PROVIDER_OPENROUTER.to_string(),
        _ => provider::select::PROVIDER_GEMINI.to_string(),
    }
}

/// The resolved API key for the effective provider. `pub(crate)` shared surface (see
/// [`resolve_provider`]).
pub(crate) fn provider_api_key(provider: &str, cfg: &Config) -> String {
    if provider == provider::select::PROVIDER_OPENROUTER {
        cfg.resolve_openrouter_api_key()
    } else {
        cfg.resolve_api_key()
    }
}

/// The env-var name doctor reports for a missing provider key. `pub(crate)` shared surface (see
/// [`resolve_provider`]).
pub(crate) fn provider_key_name(provider: &str) -> &'static str {
    if provider == provider::select::PROVIDER_OPENROUTER {
        "OPENROUTER_API_KEY"
    } else {
        "GEMINI_API_KEY"
    }
}

/// The provider's default model.
fn default_model(provider: &str) -> String {
    if provider == provider::select::PROVIDER_OPENROUTER {
        openrouter::DEFAULT_MODEL.to_string()
    } else {
        gemini::DEFAULT_MODEL.to_string()
    }
}

/// Provider-aware model reachability against a `list_models` response. Gemini normalizes the
/// `models/` prefix (Go's `ModelReachable`); OpenRouter compares slugs directly.
fn model_reachable(provider: &str, model: &str, available: &[provider::ModelInfo]) -> bool {
    if provider == provider::select::PROVIDER_OPENROUTER {
        available.iter().any(|m| m.id == model)
    } else {
        gemini::model_reachable(model, available)
    }
}

/// Print the checks (JSON envelope when `--json`/piped, else human) and return a non-zero
/// error if any check failed (Go's `reportDoctor`).
fn report(checks: &[DoctorCheck], json: bool) -> AppResult<()> {
    let failed = checks.iter().filter(|c| c.status == status::FAIL).count();

    if json {
        let env = DoctorEnvelope::from_checks(checks.to_vec());
        println!("{}", env.to_json());
    } else {
        for c in checks {
            println!("[{}] {}: {}", symbol(&c.status), c.name, c.detail);
        }
        if failed == 0 {
            println!("\nAll checks passed.");
        } else {
            println!("\n{failed} check(s) failed.");
        }
    }

    if failed > 0 {
        return Err(AppError::general(format!(
            "doctor: {failed} check(s) failed"
        )));
    }
    Ok(())
}

/// Human status symbol: pass → ✓, warn → !, fail → ✗ (Go's `doctorSymbol`).
fn symbol(st: &str) -> &'static str {
    match st {
        status::PASS => "\u{2713}", // ✓
        status::WARN => "!",
        _ => "\u{2717}", // ✗
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symbol_mapping() {
        assert_eq!(symbol(status::PASS), "\u{2713}");
        assert_eq!(symbol(status::WARN), "!");
        assert_eq!(symbol(status::FAIL), "\u{2717}");
    }

    #[test]
    fn resolve_provider_autodetect_defaults_gemini() {
        let cfg = Config::default();
        // No keys, no config provider → gemini.
        let prev_g = std::env::var_os("GEMINI_API_KEY");
        let prev_o = std::env::var_os("OPENROUTER_API_KEY");
        std::env::remove_var("GEMINI_API_KEY");
        std::env::remove_var("OPENROUTER_API_KEY");
        assert_eq!(resolve_provider(None, &cfg), "gemini");
        // CLI provider wins.
        assert_eq!(resolve_provider(Some("openrouter"), &cfg), "openrouter");
        if let Some(v) = prev_g {
            std::env::set_var("GEMINI_API_KEY", v);
        }
        if let Some(v) = prev_o {
            std::env::set_var("OPENROUTER_API_KEY", v);
        }
    }

    #[test]
    fn provider_key_name_is_provider_aware() {
        assert_eq!(provider_key_name("gemini"), "GEMINI_API_KEY");
        assert_eq!(provider_key_name("openrouter"), "OPENROUTER_API_KEY");
    }

    #[test]
    fn report_fails_when_a_check_fails() {
        let checks = vec![
            DoctorCheck::new("version", status::PASS, "v"),
            DoctorCheck::new("api_key", status::FAIL, "missing"),
        ];
        let err = report(&checks, true).unwrap_err();
        assert_eq!(err.code, exit::GENERAL);
        assert_eq!(err.message, "doctor: 1 check(s) failed");
    }

    #[test]
    fn report_ok_when_all_pass() {
        let checks = vec![DoctorCheck::new("version", status::PASS, "v")];
        assert!(report(&checks, true).is_ok());
    }
}
