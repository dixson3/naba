//! Provider-selector factory (Issue 2.5) — resolves *which* provider + model to use from
//! CLI flags, config defaults, and env-key presence, then constructs the right [`Provider`]
//! impl. This is the single home for the SPEC-PROVIDER-007 precedence chain.
//!
//! # Precedence (SPEC-PROVIDER-007 / SPEC-CFGSCHEMA-006)
//!
//! **Provider**: CLI `--provider` > config `provider` > env-key autodetect > built-in fallback.
//! * Env autodetect: only `GEMINI_API_KEY` → gemini; only `OPENROUTER_API_KEY` → openrouter;
//!   BOTH present and no CLI/config provider → **openrouter** with the default slug
//!   `google/gemini-3.1-flash-image-preview` (SPEC-PROVIDER-008, never `auto`); NEITHER present
//!   → **gemini** (the built-in fallback). The missing-key case does NOT error here — it surfaces
//!   at call time (see "Missing-key timing" below).
//!
//! **Model** (per the chosen provider):
//! * Gemini: CLI `--model` > CLI `--quality` (→ [`model_for_quality`]) > config `model` >
//!   [`gemini::DEFAULT_MODEL`]. This ordering matches SPEC-CFGSCHEMA-006 (`--model` > `--quality`
//!   > config `ResolveModel` > provider default) — an explicit `--model` overrides `--quality`.
//! * OpenRouter: CLI `--model` > config `model` > [`openrouter::DEFAULT_MODEL`]. `--quality` does
//!   **NOT** swap the model (SPEC-PROVIDER-005); it flows through raw as the request's `quality`
//!   param. The resolved `--quality` is carried on [`Selection::quality`] for both providers (for
//!   Gemini it is already baked into the model and is ignored by the provider at request time).
//!
//! # `--model` requires `--provider` (SPEC-PROVIDER-007 / SPEC-ERR-016)
//!
//! A CLI `--model` with no CLI `--provider` is a usage error (exit 2): a bare model name is
//! ambiguous across providers. This is a **CLI-flags-only** rule — config `model` without config
//! `provider` is fine (config `model` is scoped by whatever provider autodetect/config resolves),
//! matching the operator-stated rule and SPEC §9 SPEC-ERR-016.
//!
//! # `auto` guard (SPEC-PROVIDER-006)
//!
//! The selector rejects a resolved OpenRouter model of `auto` / `openrouter/auto` **early**, at
//! selection time (exit 2), rather than deferring to the OpenRouterProvider's own call-time guard.
//! The default slug is never `auto`, so this can only trigger via an explicit `--model auto` (with
//! `--provider openrouter`) or a config `model: auto`. Failing fast at selection is cleaner UX than
//! constructing a provider that is guaranteed to reject every request. The provider keeps its own
//! call-time guard as defence-in-depth (belt and suspenders).
//!
//! # Missing-key timing (match Go)
//!
//! The selector does **not** error when the chosen provider's API key is absent. It constructs the
//! provider with whatever key is present (possibly empty). Go performs the SPEC-ERR-001
//! "`<KEY> not set`" preflight (exit 3) in the command layer right before the API call, so the port
//! matches: the command layer (Issue 4.1) calls [`missing_key_error`] at call time when the
//! resolved key is empty. Keeping this out of the selector means selection stays pure and testable
//! and non-image code paths (which need no key) are never blocked.
//!
//! # Config seam (Issue 3.1 is separate)
//!
//! This module takes **already-resolved** config values as a plain [`ConfigDefaults`] struct — it
//! does NOT parse `config.yaml` (that is Issue 3.1). Issue 3.1's config loader will resolve
//! `provider` and the `model`/`quality`→model chain (SPEC-CFGSCHEMA-006 `ResolveModel`) and hand
//! the results in here as `ConfigDefaults { provider, model }`.
//!
//! # Command-layer wiring seam (Issue 4.1)
//!
//! An image command builds the three input structs and calls [`select_provider`]:
//! ```ignore
//! let inputs = SelectionInputs {
//!     provider: globals.provider.clone(),   // CLI --provider
//!     model:    globals.model.clone(),      // CLI --model
//!     quality:  Some(args.image.quality.clone()), // CLI --quality (empty string = unset)
//! };
//! let cfg = ConfigDefaults { provider: cfg_provider, model: cfg_model }; // from Issue 3.1
//! let env = EnvKeys::from_env();
//! let selection = resolve_selection(&inputs, &cfg, &env)?; // resolved provider/model/key/quality
//! if selection.api_key.is_empty() {
//!     return Err(missing_key_error(&selection.provider)); // SPEC-ERR-001, exit 3, at call time
//! }
//! let provider = build_provider(&selection);
//! // Build the GenerateRequest with the RESOLVED model so the provider uses it verbatim:
//! let req = GenerateRequest {
//!     model: selection.model.clone(),   // resolved; Gemini uses it directly (quality ignored)
//!     quality: selection.quality.clone(), // raw --quality (OpenRouter's native param)
//!     ..
//! };
//! ```
//! Setting `req.model` to the *resolved* model is important: it makes the Gemini provider use the
//! selector's decision verbatim (so an explicit `--model` truly overrides `--quality`) instead of
//! re-deriving the model from `req.quality`.

use std::collections::BTreeMap;

use crate::error::AppError;
use crate::provider::registry::{self, ProviderSpec};
use crate::provider::{gemini, openrouter, Provider};

/// Stable provider identifiers (match `Provider::name`). Retained as named constants for the
/// call sites (config, doctor) that reference a provider by name; the registry is the source of
/// truth for the *set* of providers.
pub const PROVIDER_GEMINI: &str = "gemini";
pub const PROVIDER_OPENROUTER: &str = "openrouter";

/// CLI-flag inputs to the selector (`--provider` / `--model` / `--quality`). Empty strings are
/// treated as unset (clap defaults `--quality` to `""`).
#[derive(Debug, Clone, Default)]
pub struct SelectionInputs {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub quality: Option<String>,
}

/// Already-resolved config defaults (Issue 3.1 seam — this module never parses YAML).
#[derive(Debug, Clone, Default)]
pub struct ConfigDefaults {
    pub provider: Option<String>,
    pub model: Option<String>,
}

/// Injectable, N-provider env-key presence/values (Issue 2.1). Holds a resolved credential per
/// provider name; a provider counts as present only when its key is `Some(non-empty)`. Use
/// [`EnvKeys::from_env`] in production, or [`EnvKeys::from_resolved`] to inject config-merged
/// creds (commands / MCP) or hand-built creds (tests), avoiding process-global env races.
#[derive(Debug, Clone, Default)]
pub struct EnvKeys {
    /// Resolved credential per provider name (empty values are dropped at construction).
    keys: BTreeMap<String, String>,
}

impl EnvKeys {
    /// Read every registered provider's conventional key env var from the process environment.
    /// The names live on the registry (SPEC-CFGSCHEMA-003) so config and the selector never
    /// diverge — no literal env-var strings live here.
    pub fn from_env() -> Self {
        let pairs = registry::registry().iter().filter_map(|spec| {
            let name = spec.conventional_env_var?;
            let val = std::env::var(name).ok()?;
            Some((spec.name.to_string(), val))
        });
        Self::from_resolved(pairs)
    }

    /// Build from `(provider, resolved-key)` pairs; empty keys are dropped so `present`/`get`
    /// treat them as absent.
    pub fn from_resolved<I>(pairs: I) -> Self
    where
        I: IntoIterator<Item = (String, String)>,
    {
        let keys = pairs.into_iter().filter(|(_, v)| !v.is_empty()).collect();
        Self { keys }
    }

    /// The resolved key for `provider`, if present and non-empty.
    pub fn get(&self, provider: &str) -> Option<&str> {
        self.keys
            .get(provider)
            .map(String::as_str)
            .filter(|s| !s.is_empty())
    }

    /// Whether `provider` has a resolvable, non-empty credential.
    pub fn present(&self, provider: &str) -> bool {
        self.get(provider).is_some()
    }
}

/// The resolved decision: which provider, which model, which API key, and the raw `--quality` to
/// carry on the request. Exposed (vs. only the boxed provider) so the precedence logic is unit
/// testable — a `Box<dyn Provider>` hides the resolved model behind private fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    pub provider: String,
    pub model: String,
    pub api_key: String,
    /// Raw `--quality` value to place on the request (`None` when unset). For OpenRouter this is
    /// the native quality param; for Gemini it is redundant (already baked into `model`).
    pub quality: Option<String>,
}

/// `Some(&str)` only when the option holds a non-empty string.
fn non_empty(s: Option<&String>) -> Option<&str> {
    s.map(String::as_str).filter(|v| !v.is_empty())
}

/// Validate a caller-supplied provider name (from CLI or config) against the registry.
fn validate_provider(name: &str) -> Result<&'static ProviderSpec, AppError> {
    registry::find(name).ok_or_else(|| {
        AppError::usage(format!(
            "unknown provider {name:?}\n\nValid values: {}",
            registry::names().join(", ")
        ))
    })
}

/// Autodetect the provider from env-key presence over the registry (SPEC-PROVIDER-007/008/009).
/// Only reached when no CLI/config provider was given. Among providers with resolvable creds the
/// one appearing **latest** in the declared registry order wins (generalizing SPEC-PROVIDER-008 —
/// adding a newer provider's key reroutes to it); with no creds the fallback is the first
/// registered provider (gemini). For the two-provider case this reproduces the legacy behavior
/// exactly: only-gemini→gemini, only-openrouter→openrouter, both→openrouter, neither→gemini. The
/// missing-key error for the fallback surfaces at call time (SPEC-ERR-001).
fn autodetect(env: &EnvKeys) -> &'static ProviderSpec {
    let names = registry::names();
    let name = autodetect_name(&names, |p| env.present(p), registry::fallback());
    registry::find(name).expect("autodetect yields a registered provider")
}

/// The autodetected provider name for `env` (SPEC-PROVIDER-009). Public so `doctor` / `preflight`
/// resolve the effective provider identically to the image selector, over the same registry.
pub fn autodetect_provider(env: &EnvKeys) -> &'static str {
    autodetect(env).name
}

/// Pure autodetect scan (SPEC-PROVIDER-009): given the declared provider `order` (oldest→newest),
/// the **latest** provider with resolvable creds wins; with none, `fallback`. Factored out of
/// [`autodetect`] so the N-provider precedence — including a hypothetical third provider slotted
/// after gemini/openrouter — is unit-testable without mutating the real registry.
fn autodetect_name<'a>(
    order: &[&'a str],
    present: impl Fn(&str) -> bool,
    fallback: &'a str,
) -> &'a str {
    order
        .iter()
        .rev()
        .copied()
        .find(|p| present(p))
        .unwrap_or(fallback)
}

/// Resolve the full [`Selection`] from CLI flags, config defaults, and env keys, applying the
/// SPEC-PROVIDER-007 precedence chain. Pure (no HTTP, no provider construction) so every branch is
/// unit testable. [`select_provider`] wraps this to build the boxed provider.
pub fn resolve_selection(
    inputs: &SelectionInputs,
    cfg: &ConfigDefaults,
    env: &EnvKeys,
) -> Result<Selection, AppError> {
    let cli_provider = non_empty(inputs.provider.as_ref());
    let cli_model = non_empty(inputs.model.as_ref());
    let cli_quality = non_empty(inputs.quality.as_ref());

    // SPEC-PROVIDER-007 / SPEC-ERR-016: CLI `--model` without CLI `--provider` is ambiguous → 2.
    // CLI-flags-only rule: config `model` without config `provider` is intentionally allowed.
    if cli_model.is_some() && cli_provider.is_none() {
        return Err(AppError::usage(
            "--model requires --provider\n\nSpecify --provider gemini or --provider openrouter",
        ));
    }

    // Provider: CLI > config > autodetect > fallback. Every branch resolves to a registry spec.
    let spec = if let Some(p) = cli_provider {
        validate_provider(p)?
    } else if let Some(p) = non_empty(cfg.provider.as_ref()) {
        validate_provider(p)?
    } else {
        autodetect(env)
    };

    let cfg_model = non_empty(cfg.model.as_ref());
    let model = resolve_model_for(spec, cli_model, cli_quality, cfg_model)?;

    // SPEC-PROVIDER-006: a provider that rejects the `auto` router must never back an image path —
    // reject early (exit 2), registry-declared so it is not a hardcoded openrouter check here.
    if spec.rejects_auto_router && openrouter::is_auto_router(&model) {
        return Err(AppError::usage(format!(
            "model {model:?} cannot generate images: openrouter/auto is a text-only router\n\nSet an image model, e.g. --model {}",
            openrouter::DEFAULT_MODEL
        )));
    }

    let api_key = env.get(spec.name).unwrap_or_default().to_string();

    Ok(Selection {
        provider: spec.name.to_string(),
        model,
        api_key,
        quality: cli_quality.map(str::to_string),
    })
}

/// Resolve the model for the chosen provider (SPEC-CFGSCHEMA-006 CLI precedence): CLI `--model` >
/// (for a `quality_selects_model` provider) CLI `--quality` tier > config `model` > the
/// provider's compiled-in default. The `--quality`→tier step is registry-declared (Gemini maps
/// it to a model; OpenRouter carries it raw on the request instead), so this is not a hardcoded
/// per-provider branch.
fn resolve_model_for(
    spec: &ProviderSpec,
    cli_model: Option<&str>,
    cli_quality: Option<&str>,
    cfg_model: Option<&str>,
) -> Result<String, AppError> {
    if let Some(m) = cli_model {
        return Ok(m.to_string());
    }
    if spec.quality_selects_model {
        if let Some(q) = cli_quality {
            return model_for_quality_owned(q);
        }
    }
    if let Some(m) = cfg_model {
        return Ok(m.to_string());
    }
    Ok(spec.default_model.to_string())
}

/// [`gemini::model_for_quality`] returning an owned `String` (its `&'static str` doesn't unify with
/// the other owned branches in one `if/else`).
fn model_for_quality_owned(quality: &str) -> Result<String, AppError> {
    gemini::model_for_quality(quality).map(str::to_string)
}

/// Build the concrete [`Provider`] from a resolved [`Selection`] via the registry builder. The
/// provider is constructed with the resolved model and the provider-appropriate API key.
pub fn build_provider(sel: &Selection) -> Box<dyn Provider> {
    registry::find(&sel.provider)
        .expect("validated provider name")
        .build(&sel.api_key, &sel.model)
}

/// The 2.5 factory entry point: resolve precedence and construct the provider (SPEC-PROVIDER-007).
/// Missing-key handling is the caller's job at call time (see module docs / [`missing_key_error`]).
pub fn select_provider(
    inputs: &SelectionInputs,
    cfg: &ConfigDefaults,
    env: &EnvKeys,
) -> Result<Box<dyn Provider>, AppError> {
    let selection = resolve_selection(inputs, cfg, env)?;
    Ok(build_provider(&selection))
}

/// The SPEC-ERR-001 "API key unset" error (exit 3), naming the selected provider's env key
/// ([DIVERGENCE] under multi-provider). The command layer (Issue 4.1) calls this at call time when
/// [`Selection::api_key`] is empty — the selector never raises it (Go errors at call time).
pub fn missing_key_error(provider: &str) -> AppError {
    let key = registry::conventional_env_var(provider).unwrap_or(crate::config::ENV_API_KEY);
    AppError::auth(format!(
        "{key} not set.\n\nSet it with: export {key}=<your-key>\nOr run: naba config set api_key <your-key>"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::exit;

    fn inputs(
        provider: Option<&str>,
        model: Option<&str>,
        quality: Option<&str>,
    ) -> SelectionInputs {
        SelectionInputs {
            provider: provider.map(str::to_string),
            model: model.map(str::to_string),
            quality: quality.map(str::to_string),
        }
    }

    fn cfg(provider: Option<&str>, model: Option<&str>) -> ConfigDefaults {
        ConfigDefaults {
            provider: provider.map(str::to_string),
            model: model.map(str::to_string),
        }
    }

    fn env(gemini: Option<&str>, openrouter: Option<&str>) -> EnvKeys {
        let mut pairs = Vec::new();
        if let Some(g) = gemini {
            pairs.push((PROVIDER_GEMINI.to_string(), g.to_string()));
        }
        if let Some(o) = openrouter {
            pairs.push((PROVIDER_OPENROUTER.to_string(), o.to_string()));
        }
        EnvKeys::from_resolved(pairs)
    }

    /// Build an [`EnvKeys`] from arbitrary `(provider, key)` pairs — used to model an N-provider
    /// autodetect (including a hypothetical third provider not in the registry, which the scan
    /// simply never selects because the registry drives the scan).
    fn env_pairs(pairs: &[(&str, &str)]) -> EnvKeys {
        EnvKeys::from_resolved(pairs.iter().map(|(p, k)| (p.to_string(), k.to_string())))
    }

    fn resolve(i: SelectionInputs, c: ConfigDefaults, e: EnvKeys) -> Selection {
        resolve_selection(&i, &c, &e).unwrap()
    }

    // ---- Provider precedence ----

    #[test]
    fn cli_provider_wins_over_config_and_autodetect() {
        // CLI gemini beats config openrouter and an openrouter-only env.
        let sel = resolve(
            inputs(Some("gemini"), None, None),
            cfg(Some("openrouter"), None),
            env(None, Some("or-key")),
        );
        assert_eq!(sel.provider, PROVIDER_GEMINI);
        assert_eq!(sel.model, gemini::DEFAULT_MODEL);
    }

    #[test]
    fn config_provider_wins_over_autodetect() {
        // Config gemini beats an openrouter-only env (SPEC-PROVIDER-008 mitigation).
        let sel = resolve(
            inputs(None, None, None),
            cfg(Some("gemini"), None),
            env(Some("g-key"), Some("or-key")),
        );
        assert_eq!(sel.provider, PROVIDER_GEMINI);
        assert_eq!(sel.api_key, "g-key");
        assert_eq!(sel.model, gemini::DEFAULT_MODEL);
    }

    #[test]
    fn autodetect_only_gemini_key() {
        let sel = resolve(
            inputs(None, None, None),
            cfg(None, None),
            env(Some("g-key"), None),
        );
        assert_eq!(sel.provider, PROVIDER_GEMINI);
        assert_eq!(sel.api_key, "g-key");
        assert_eq!(sel.model, gemini::DEFAULT_MODEL);
    }

    #[test]
    fn autodetect_only_openrouter_key() {
        let sel = resolve(
            inputs(None, None, None),
            cfg(None, None),
            env(None, Some("or-key")),
        );
        assert_eq!(sel.provider, PROVIDER_OPENROUTER);
        assert_eq!(sel.api_key, "or-key");
        assert_eq!(sel.model, openrouter::DEFAULT_MODEL);
    }

    #[test]
    fn autodetect_both_keys_no_config_is_openrouter_default_slug() {
        // SPEC-PROVIDER-007/008: both keys + no CLI/config provider → openrouter + default slug.
        let sel = resolve(
            inputs(None, None, None),
            cfg(None, None),
            env(Some("g-key"), Some("or-key")),
        );
        assert_eq!(sel.provider, PROVIDER_OPENROUTER);
        assert_eq!(sel.model, "google/gemini-3.1-flash-image-preview");
        assert_eq!(sel.api_key, "or-key");
    }

    #[test]
    fn autodetect_no_keys_falls_back_to_gemini_with_empty_key() {
        // NEITHER present → gemini fallback; empty key (missing-key error is deferred to call time).
        let sel = resolve(inputs(None, None, None), cfg(None, None), env(None, None));
        assert_eq!(sel.provider, PROVIDER_GEMINI);
        assert_eq!(sel.model, gemini::DEFAULT_MODEL);
        assert_eq!(sel.api_key, "");
    }

    // ---- N-provider autodetect precedence (SPEC-PROVIDER-009) ----

    #[test]
    fn autodetect_name_latest_registered_with_creds_wins() {
        // Two-provider parity: both keys → openrouter (latest registered), matching the legacy
        // SPEC-PROVIDER-008 reroute.
        let order = ["gemini", "openrouter"];
        let present = |set: &'static [&'static str]| move |p: &str| set.contains(&p);
        assert_eq!(
            autodetect_name(&order, present(&["gemini", "openrouter"]), "gemini"),
            "openrouter"
        );
        assert_eq!(
            autodetect_name(&order, present(&["gemini"]), "gemini"),
            "gemini"
        );
        assert_eq!(
            autodetect_name(&order, present(&["openrouter"]), "gemini"),
            "openrouter"
        );
        assert_eq!(autodetect_name(&order, |_| false, "gemini"), "gemini");
    }

    #[test]
    fn autodetect_name_hypothetical_third_provider() {
        // A registry that later gains a third provider (e.g. bedrock) slots it last; the latest
        // provider with creds wins, generalizing the reroute rule.
        let order = ["gemini", "openrouter", "bedrock"];
        let present = |set: &'static [&'static str]| move |p: &str| set.contains(&p);
        // All three present → bedrock (latest).
        assert_eq!(
            autodetect_name(
                &order,
                present(&["gemini", "openrouter", "bedrock"]),
                "gemini"
            ),
            "bedrock"
        );
        // gemini + bedrock (no openrouter) → bedrock.
        assert_eq!(
            autodetect_name(&order, present(&["gemini", "bedrock"]), "gemini"),
            "bedrock"
        );
        // gemini + openrouter (bedrock absent) → openrouter (unchanged from the 2-provider case).
        assert_eq!(
            autodetect_name(&order, present(&["gemini", "openrouter"]), "gemini"),
            "openrouter"
        );
        // Only bedrock → bedrock.
        assert_eq!(
            autodetect_name(&order, present(&["bedrock"]), "gemini"),
            "bedrock"
        );
    }

    #[test]
    fn autodetect_ignores_creds_for_unregistered_providers() {
        // A credential for a provider not in the registry never wins — the scan is registry-driven,
        // so a stray `bedrock` cred (before bedrock is registered) falls back to autodetect over
        // the real registry (gemini/openrouter only).
        let sel = resolve(
            inputs(None, None, None),
            cfg(None, None),
            env_pairs(&[("bedrock", "b-key"), ("gemini", "g-key")]),
        );
        assert_eq!(sel.provider, PROVIDER_GEMINI);
        assert_eq!(sel.api_key, "g-key");
    }

    #[test]
    fn empty_string_keys_are_treated_as_absent() {
        // Some("") must not count as present.
        let sel = resolve(
            inputs(None, None, None),
            cfg(None, None),
            env(Some(""), Some("or-key")),
        );
        assert_eq!(sel.provider, PROVIDER_OPENROUTER);
    }

    // ---- --model requires --provider (SPEC-ERR-016) ----

    #[test]
    fn cli_model_without_cli_provider_is_usage_error() {
        let err = resolve_selection(
            &inputs(None, Some("some-model"), None),
            &cfg(None, None),
            &env(Some("g-key"), None),
        )
        .unwrap_err();
        assert_eq!(err.code, exit::USAGE);
        assert!(err.message.starts_with("--model requires --provider"));
    }

    #[test]
    fn config_model_without_config_provider_is_allowed() {
        // CLI-flags-only rule: config model alone is fine (scoped by autodetected provider).
        let sel = resolve(
            inputs(None, None, None),
            cfg(None, Some("gemini-custom")),
            env(Some("g-key"), None),
        );
        assert_eq!(sel.provider, PROVIDER_GEMINI);
        assert_eq!(sel.model, "gemini-custom");
    }

    #[test]
    fn cli_model_with_cli_provider_is_ok() {
        let sel = resolve(
            inputs(Some("openrouter"), Some("openai/gpt-image-1"), None),
            cfg(None, None),
            env(None, Some("or-key")),
        );
        assert_eq!(sel.provider, PROVIDER_OPENROUTER);
        assert_eq!(sel.model, "openai/gpt-image-1");
    }

    // ---- Model resolution ----

    #[test]
    fn gemini_quality_high_maps_to_pro_model() {
        let sel = resolve(
            inputs(Some("gemini"), None, Some("high")),
            cfg(None, None),
            env(Some("g-key"), None),
        );
        assert_eq!(sel.provider, PROVIDER_GEMINI);
        assert_eq!(sel.model, gemini::PRO_MODEL);
        assert_eq!(sel.quality.as_deref(), Some("high"));
    }

    #[test]
    fn gemini_quality_fast_maps_to_flash_model() {
        let sel = resolve(
            inputs(Some("gemini"), None, Some("fast")),
            cfg(None, None),
            env(Some("g-key"), None),
        );
        assert_eq!(sel.model, gemini::FLASH_MODEL);
    }

    #[test]
    fn gemini_cli_model_overrides_quality() {
        // SPEC-PROVIDER-005: explicit --model overrides --quality.
        let sel = resolve(
            inputs(Some("gemini"), Some("gemini-3-pro-image"), Some("fast")),
            cfg(None, None),
            env(Some("g-key"), None),
        );
        assert_eq!(sel.model, "gemini-3-pro-image");
        // quality still carried raw (ignored by the Gemini provider once model is set).
        assert_eq!(sel.quality.as_deref(), Some("fast"));
    }

    #[test]
    fn gemini_quality_beats_config_model() {
        // SPEC-CFGSCHEMA-006: --quality > config model.
        let sel = resolve(
            inputs(Some("gemini"), None, Some("high")),
            cfg(None, Some("gemini-config-model")),
            env(Some("g-key"), None),
        );
        assert_eq!(sel.model, gemini::PRO_MODEL);
    }

    #[test]
    fn gemini_invalid_quality_is_usage_error() {
        let err = resolve_selection(
            &inputs(Some("gemini"), None, Some("medium")),
            &cfg(None, None),
            &env(Some("g-key"), None),
        )
        .unwrap_err();
        assert_eq!(err.code, exit::USAGE);
        assert_eq!(
            err.message,
            "invalid quality \"medium\"\n\nValid values: fast, high"
        );
    }

    #[test]
    fn openrouter_quality_does_not_change_model_and_is_carried() {
        // SPEC-PROVIDER-005: --quality high on OpenRouter keeps the slug, carries quality raw.
        let sel = resolve(
            inputs(Some("openrouter"), None, Some("high")),
            cfg(None, None),
            env(None, Some("or-key")),
        );
        assert_eq!(sel.provider, PROVIDER_OPENROUTER);
        assert_eq!(sel.model, openrouter::DEFAULT_MODEL);
        assert_eq!(sel.quality.as_deref(), Some("high"));
    }

    #[test]
    fn openrouter_config_model_used_when_no_cli_model() {
        let sel = resolve(
            inputs(None, None, None),
            cfg(Some("openrouter"), Some("bytedance-seed/seedream-4.5")),
            env(None, Some("or-key")),
        );
        assert_eq!(sel.model, "bytedance-seed/seedream-4.5");
    }

    // ---- auto guard (SPEC-PROVIDER-006) ----

    #[test]
    fn openrouter_auto_model_rejected_early() {
        let err = resolve_selection(
            &inputs(Some("openrouter"), Some("openrouter/auto"), None),
            &cfg(None, None),
            &env(None, Some("or-key")),
        )
        .unwrap_err();
        assert_eq!(err.code, exit::USAGE);
        assert!(err.message.contains("cannot generate images"));
    }

    #[test]
    fn openrouter_bare_auto_model_rejected_early() {
        let err = resolve_selection(
            &inputs(Some("openrouter"), Some("auto"), None),
            &cfg(None, None),
            &env(None, Some("or-key")),
        )
        .unwrap_err();
        assert_eq!(err.code, exit::USAGE);
    }

    #[test]
    fn config_auto_model_also_rejected() {
        let err = resolve_selection(
            &inputs(None, None, None),
            &cfg(Some("openrouter"), Some("openrouter/auto")),
            &env(None, Some("or-key")),
        )
        .unwrap_err();
        assert_eq!(err.code, exit::USAGE);
    }

    // ---- Unknown provider ----

    #[test]
    fn unknown_cli_provider_is_usage_error() {
        let err = resolve_selection(
            &inputs(Some("dalle"), None, None),
            &cfg(None, None),
            &env(None, None),
        )
        .unwrap_err();
        assert_eq!(err.code, exit::USAGE);
        assert_eq!(
            err.message,
            "unknown provider \"dalle\"\n\nValid values: gemini, openrouter"
        );
    }

    // ---- build_provider / select_provider produce the right concrete provider ----

    #[test]
    fn build_provider_names_match_selection() {
        let g = build_provider(&Selection {
            provider: PROVIDER_GEMINI.to_string(),
            model: gemini::DEFAULT_MODEL.to_string(),
            api_key: "k".to_string(),
            quality: None,
        });
        assert_eq!(g.name(), "gemini");

        let o = build_provider(&Selection {
            provider: PROVIDER_OPENROUTER.to_string(),
            model: openrouter::DEFAULT_MODEL.to_string(),
            api_key: "k".to_string(),
            quality: None,
        });
        assert_eq!(o.name(), "openrouter");
    }

    #[test]
    fn select_provider_constructs_from_precedence() {
        let p = select_provider(
            &inputs(Some("openrouter"), None, None),
            &cfg(None, None),
            &env(None, Some("or-key")),
        )
        .unwrap();
        assert_eq!(p.name(), "openrouter");
    }

    // ---- missing-key helper (SPEC-ERR-001) ----

    #[test]
    fn missing_key_error_names_provider_key() {
        let g = missing_key_error(PROVIDER_GEMINI);
        assert_eq!(g.code, exit::AUTH);
        assert!(g.message.starts_with("GEMINI_API_KEY not set."));

        let o = missing_key_error(PROVIDER_OPENROUTER);
        assert_eq!(o.code, exit::AUTH);
        assert!(o.message.starts_with("OPENROUTER_API_KEY not set."));
    }
}
