//! Provider registry (Issue 2.1) — the single source of truth for the set of providers naba
//! knows about, replacing the ~9 hardcoded provider match sites that used to live in
//! [`select`](crate::provider::select), [`crate::config`], and [`crate::doctor`].
//!
//! # What the registry declares
//!
//! Each [`ProviderSpec`] carries a provider's identity plus everything the rest of the codebase
//! needs to treat providers uniformly:
//!
//! * `name` — the stable identifier (matches [`Provider::name`]).
//! * `conventional_env_var` — the provider's conventional default key env var (e.g.
//!   `GEMINI_API_KEY`); the single source the api-key resolver and the selector both read.
//! * `default_model` — the provider's compiled-in default model (SPEC-CFGSCHEMA-006), so no
//!   provider is ever model-less.
//! * `quality_selects_model` — whether `--quality` maps to a *model* (Gemini's Flash/Pro tier)
//!   or flows through as a native request parameter (OpenRouter). Declared per provider so the
//!   selector never hardcodes the distinction (SPEC-PROVIDER-005).
//! * `rejects_auto_router` — whether the provider rejects the `auto`/`openrouter/auto` sentinel
//!   as an image model (SPEC-PROVIDER-006).
//! * a `builder` — constructs the concrete [`Provider`] from a resolved key + model.
//!
//! # Declared order + N-provider autodetect (SPEC-PROVIDER-007/009)
//!
//! [`REGISTRY`] is an **ordered** list, oldest→newest. That single ordering drives:
//!
//! * the `config` valid-keys surface and the `naba provider`/`naba models` listing (forward), and
//! * env-key **autodetect precedence** (see [`autodetect`]): among providers with resolvable
//!   creds, the one appearing **latest** in the declared order wins — a generalization of
//!   SPEC-PROVIDER-008 (adding a newer provider's key reroutes to it). When *no* provider has
//!   creds the fallback is the **first** registered provider (gemini). This preserves the legacy
//!   two-provider behavior exactly: only-gemini→gemini, only-openrouter→openrouter,
//!   both→openrouter, neither→gemini.
//!
//! Adding a provider (e.g. Bedrock in a later epic) is a **single new [`ProviderSpec`] entry**
//! here — the selector, config, doctor, and the `provider`/`models` commands all pick it up with
//! no further edits.

use crate::provider::{gemini, openrouter, GeminiProvider, OpenRouterProvider, Provider};

/// A registered provider (see the module docs). All fields are `'static` so the whole registry
/// is a compile-time `static`.
pub struct ProviderSpec {
    /// Stable provider identifier (matches [`Provider::name`]).
    pub name: &'static str,
    /// The provider's conventional default key env var, or `None` when it has none.
    pub conventional_env_var: Option<&'static str>,
    /// The provider's compiled-in default model (SPEC-CFGSCHEMA-006).
    pub default_model: &'static str,
    /// `--quality` selects the model for this provider (Gemini tier) vs. flows through as a
    /// native request parameter (OpenRouter). SPEC-PROVIDER-005.
    pub quality_selects_model: bool,
    /// The provider rejects the `auto` router sentinel as an image model (SPEC-PROVIDER-006).
    pub rejects_auto_router: bool,
    /// Builds the concrete provider from a resolved api key + model.
    builder: fn(&str, &str) -> Box<dyn Provider>,
}

impl ProviderSpec {
    /// Construct the concrete [`Provider`] with a resolved api key + model.
    pub fn build(&self, api_key: &str, model: &str) -> Box<dyn Provider> {
        (self.builder)(api_key, model)
    }
}

/// The declared provider registry (see the module docs for the ordering contract). Adding a
/// provider is a single new entry here.
static REGISTRY: &[ProviderSpec] = &[
    ProviderSpec {
        name: "gemini",
        conventional_env_var: Some(crate::config::ENV_API_KEY),
        default_model: gemini::DEFAULT_MODEL,
        quality_selects_model: true,
        rejects_auto_router: false,
        builder: |k, m| Box::new(GeminiProvider::new(k, m)),
    },
    ProviderSpec {
        name: "openrouter",
        conventional_env_var: Some(crate::config::ENV_OPENROUTER_API_KEY),
        default_model: openrouter::DEFAULT_MODEL,
        quality_selects_model: false,
        rejects_auto_router: true,
        builder: |k, m| Box::new(OpenRouterProvider::new(k, m)),
    },
];

/// The registry as an ordered slice.
pub fn registry() -> &'static [ProviderSpec] {
    REGISTRY
}

/// The registered provider spec for `name`, or `None` when the provider is unknown.
pub fn find(name: &str) -> Option<&'static ProviderSpec> {
    REGISTRY.iter().find(|s| s.name == name)
}

/// Whether `name` is a registered provider.
pub fn is_known(name: &str) -> bool {
    find(name).is_some()
}

/// The registered provider names, in declared order.
pub fn names() -> Vec<&'static str> {
    REGISTRY.iter().map(|s| s.name).collect()
}

/// The conventional default key env var for `provider`, or `None`.
pub fn conventional_env_var(provider: &str) -> Option<&'static str> {
    find(provider).and_then(|s| s.conventional_env_var)
}

/// The first (fallback) registered provider (SPEC-PROVIDER-007).
pub fn fallback() -> &'static str {
    REGISTRY[0].name
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declared_order_is_gemini_then_openrouter() {
        assert_eq!(names(), vec!["gemini", "openrouter"]);
        assert_eq!(fallback(), "gemini");
    }

    #[test]
    fn find_and_env_vars() {
        assert_eq!(find("gemini").unwrap().default_model, gemini::DEFAULT_MODEL);
        assert_eq!(
            conventional_env_var("gemini"),
            Some(crate::config::ENV_API_KEY)
        );
        assert_eq!(
            conventional_env_var("openrouter"),
            Some(crate::config::ENV_OPENROUTER_API_KEY)
        );
        assert_eq!(conventional_env_var("bedrock"), None);
        assert!(!is_known("bedrock"));
    }

    #[test]
    fn provider_capabilities_are_declared() {
        let g = find("gemini").unwrap();
        assert!(g.quality_selects_model);
        assert!(!g.rejects_auto_router);
        let o = find("openrouter").unwrap();
        assert!(!o.quality_selects_model);
        assert!(o.rejects_auto_router);
    }

    #[test]
    fn build_produces_named_provider() {
        assert_eq!(find("gemini").unwrap().build("k", "m").name(), "gemini");
        assert_eq!(
            find("openrouter").unwrap().build("k", "m").name(),
            "openrouter"
        );
    }
}
