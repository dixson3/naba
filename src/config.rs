//! Config load/save, env/path resolution, and `config get`/`config set` backing.
//!
//! The schema, key set, path resolution, and precedence rules are pinned by SPEC §6
//! (SPEC-CFGSCHEMA-001..006), §3.8 (SPEC-CONFIG-001..003), §9 (SPEC-ERR-007..009), and §10
//! (SPEC-MIGRATE-001..004).
//!
//! # Nested per-provider schema (Epic 1 — SPEC-CFGSCHEMA-001)
//!
//! The config is **nested**, not flat: a top-level `default_provider`, a `providers` map keyed
//! by provider name (each entry `{ model, api-key, api-key-envvar }`), plus the top-level image
//! defaults (`aspect`, `resolution`, `quality`, `default_output_dir`). Every field is optional
//! (`omitempty` via `skip_serializing_if`); a zero [`Config`] serializes to an empty document.
//!
//! # Uniform api-key resolution (Epic 1 — SPEC-CFGSCHEMA-003)
//!
//! One resolver, [`Config::resolve_api_key_for`], serves every provider with a single
//! precedence: inline `providers.<name>.api-key` → the custom env var named by
//! `providers.<name>.api-key-envvar` → the provider's conventional default env var
//! ([`conventional_env_var`], e.g. `GEMINI_API_KEY` / `OPENROUTER_API_KEY`). The env-var names
//! live in ONE place ([`conventional_env_var`] / the `ENV_*` constants); the selector's
//! [`crate::provider::select::EnvKeys::from_env`] reads the same constants.
//!
//! # Per-provider default model (Epic 1 — SPEC-CFGSCHEMA-006)
//!
//! Each provider designates its own default model. When `providers.<name>.model` is absent, the
//! provider selector ([`crate::provider::select`]) resolves it to that provider's compiled-in
//! default (`gemini::DEFAULT_MODEL`, `openrouter::DEFAULT_MODEL`) — no provider is ever
//! model-less. The registry is left extensible for later providers (e.g. Bedrock).
//!
//! # YAML crate (SPEC-MIGRATE-004)
//!
//! Uses **`serde_norway`** — a maintained fork of `serde_yaml`. **`serde_yml` is forbidden**
//! (RUSTSEC-2025-0068). `serde_norway::to_string` omits fields marked
//! `skip_serializing_if`, reproducing Go's `omitempty`.
//!
//! # Home directory
//!
//! Resolved by reading `$HOME` directly (matching Go's `os.UserHomeDir`, which on unix is
//! `$HOME`) — no `dirs`/`home` crate dependency. An unset `$HOME` yields an empty base.
//!
//! # Config auto-migration (Epic 1 — SPEC-MIGRATE-001..004)
//!
//! [`Config::load`] runs [`migrate_if_needed`] before the read/parse. The flat→nested schema
//! change is a **STRUCTURAL** migration: [`needs_structural_migration`] detects the old flat
//! shape (a top-level `api_key`/`model`/`provider` with no `providers`/`default_provider`), and
//! [`migrate_file`] backs the original bytes up to `<path>.bak`, then rewrites the document into
//! the nested schema. It is idempotent (a migrated file has no flat keys, so a second load
//! no-ops and the `.bak` is never clobbered) and graceful on empty/missing/malformed inputs. A
//! structural rewrite **loses YAML comments** (serde round-trip does not preserve them) — an
//! accepted loss mitigated by the `.bak` backup (SPEC-MIGRATE-003).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::provider::gemini;
use crate::provider::select::{ConfigDefaults, PROVIDER_GEMINI, PROVIDER_OPENROUTER};

const CONFIG_FILE_NAME: &str = "config.yaml";

/// `GEMINI_API_KEY` — the Gemini provider's conventional default key env var
/// (SPEC-CFGSCHEMA-003). Centralized here (the selector's `EnvKeys::from_env` reads it too).
pub const ENV_API_KEY: &str = "GEMINI_API_KEY";
/// `OPENROUTER_API_KEY` — the OpenRouter provider's conventional default key env var
/// (SPEC-CFGSCHEMA-003).
pub const ENV_OPENROUTER_API_KEY: &str = "OPENROUTER_API_KEY";
/// `NABA_OUTPUT_DIR` — output-dir override, consumed by MCP only (SPEC-CFGSCHEMA-004/005).
pub const ENV_OUTPUT_DIR: &str = "NABA_OUTPUT_DIR";
/// `NABA_CONFIG_DIR` — config-dir override (SPEC-CFGSCHEMA-001).
pub const ENV_CONFIG_DIR: &str = "NABA_CONFIG_DIR";

/// The providers naba knows about (Epic 1). The order is load-bearing: it drives the
/// `Valid keys:` error lines and the per-provider dotted-key surface. Bedrock joins this list
/// in a later epic — the registry is intentionally extensible.
pub const KNOWN_PROVIDERS: [&str; 2] = [PROVIDER_GEMINI, PROVIDER_OPENROUTER];

/// The provider's conventional default key env var (SPEC-CFGSCHEMA-003). The single source of
/// truth for the env-var names — [`Config::resolve_api_key_for`] and the selector both defer
/// here. Returns `None` for a provider with no conventional key env var.
pub fn conventional_env_var(provider: &str) -> Option<&'static str> {
    match provider {
        PROVIDER_GEMINI => Some(ENV_API_KEY),
        PROVIDER_OPENROUTER => Some(ENV_OPENROUTER_API_KEY),
        _ => None,
    }
}

/// Per-provider config entry (SPEC-CFGSCHEMA-001): a default `model` plus an api-key source
/// (`api-key` inline, or `api-key-envvar` naming a custom env var). All fields optional.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// This provider's default model. Absent → the provider's compiled-in default
    /// (resolved by the selector, SPEC-CFGSCHEMA-006).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub model: String,
    /// Inline api-key (highest precedence in [`Config::resolve_api_key_for`]).
    #[serde(rename = "api-key", default, skip_serializing_if = "String::is_empty")]
    pub api_key: String,
    /// Name of a custom env var to read the api-key from (second precedence).
    #[serde(
        rename = "api-key-envvar",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub api_key_envvar: String,
}

impl ProviderConfig {
    /// Whether every field is empty (an entry safe to drop from the map so it does not
    /// serialize as a bare `name: {}`).
    fn is_empty(&self) -> bool {
        self.model.is_empty() && self.api_key.is_empty() && self.api_key_envvar.is_empty()
    }
}

/// The naba configuration (SPEC-CFGSCHEMA-001). Nested per-provider schema. All keys are
/// optional (`omitempty` via `skip_serializing_if`); a zero `Config` serializes to an empty
/// document.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    /// The provider used when no CLI `--provider` is given (SPEC-PROVIDER-007). Absent → the
    /// selector's env-key autodetect decides.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub default_provider: String,
    /// Per-provider entries keyed by provider name.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub providers: BTreeMap<String, ProviderConfig>,
    /// MCP output-dir default (SPEC-CFGSCHEMA-004/005). Top-level image/output default.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub default_output_dir: String,
    /// imageConfig aspect default; a per-call `--aspect` flag overrides it.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub aspect: String,
    /// imageConfig resolution default; a per-call `--resolution` flag overrides it.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub resolution: String,
    /// Model quality alias (fast/high); a configured per-provider `model` beats it
    /// ([`Config::resolve_model`]).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub quality: String,
}

/// Addressable per-provider field for the dotted `config get`/`set` surface.
#[derive(Debug, Clone, Copy)]
enum PField {
    Model,
    ApiKey,
    ApiKeyEnvvar,
}

/// Whether `name` is a provider naba knows about.
fn is_known_provider(name: &str) -> bool {
    KNOWN_PROVIDERS.contains(&name)
}

/// Parse a dotted `<provider>.<field>` key into its parts, or `None` when the provider is
/// unknown or the field is not one of `model`/`api-key`/`api-key-envvar`.
fn parse_provider_key(key: &str) -> Option<(&str, PField)> {
    let (provider, field) = key.split_once('.')?;
    if !is_known_provider(provider) {
        return None;
    }
    let field = match field {
        "model" => PField::Model,
        "api-key" => PField::ApiKey,
        "api-key-envvar" => PField::ApiKeyEnvvar,
        _ => return None,
    };
    Some((provider, field))
}

/// The full set of addressable config keys (SPEC-CFGSCHEMA-001), in the pinned order that
/// `config get`/`config set` join into their `Valid keys:` error lines (SPEC-ERR-008/009):
/// `default-provider`, each provider's `model`/`api-key`/`api-key-envvar`, then the top-level
/// image defaults. Legacy flat keys (`api_key`, `model`, `provider`) remain accepted as aliases
/// for backward compatibility but are not advertised here.
pub fn valid_keys() -> Vec<String> {
    let mut keys = vec!["default-provider".to_string()];
    for p in KNOWN_PROVIDERS {
        keys.push(format!("{p}.model"));
        keys.push(format!("{p}.api-key"));
        keys.push(format!("{p}.api-key-envvar"));
    }
    for k in ["default_output_dir", "aspect", "resolution", "quality"] {
        keys.push(k.to_string());
    }
    keys
}

/// `valid_keys()` joined with `", "` for the `Valid keys: <list>` error lines.
fn valid_keys_joined() -> String {
    valid_keys().join(", ")
}

/// The home directory from `$HOME`, or empty when unset (matches Go's error-swallowing
/// `UserHomeDir`).
fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default()
}

/// Config directory (SPEC-CFGSCHEMA-001, SPEC-DIRS-001): `NABA_CONFIG_DIR` >
/// `$XDG_CONFIG_HOME/naba` > `<home>/.config/naba`.
///
/// This is the **single source of truth** for the config dir — `self`/`preflight`
/// ([`crate::dirs`]) defer to it so they never diverge from `config`.
pub fn config_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os(ENV_CONFIG_DIR) {
        if !dir.is_empty() {
            return PathBuf::from(dir);
        }
    }
    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
        if !xdg.is_empty() {
            return PathBuf::from(xdg).join("naba");
        }
    }
    home_dir().join(".config").join("naba")
}

/// Full path to `config.yaml` (SPEC-CFGSCHEMA-001).
pub fn config_path() -> PathBuf {
    config_dir().join(CONFIG_FILE_NAME)
}

/// The MCP XDG-default output dir `<home>/.local/share/naba/images` (SPEC-CFGSCHEMA-004).
/// Empty when `$HOME` is unset (matches Go).
pub fn default_output_dir() -> String {
    let home = home_dir();
    if home.as_os_str().is_empty() {
        return String::new();
    }
    home.join(".local")
        .join("share")
        .join("naba")
        .join("images")
        .to_string_lossy()
        .into_owned()
}

impl Config {
    /// Load `config.yaml`. A missing file → zero-value [`Config`], no error
    /// (SPEC-CFGSCHEMA-002). A read error other than not-found, or a YAML parse error,
    /// surfaces as `ExitGeneral` (1).
    ///
    /// [`migrate_if_needed`] runs ahead of the read: an old flat config is transformed into
    /// the nested schema (with a `.bak` backup) before it is parsed (SPEC-MIGRATE-002).
    pub fn load() -> AppResult<Config> {
        let path = config_path();
        migrate_if_needed()?;
        let data = match std::fs::read_to_string(&path) {
            Ok(data) => data,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Config::default()),
            Err(e) => return Err(AppError::general(e.to_string())),
        };
        serde_norway::from_str(&data).map_err(|e| AppError::general(e.to_string()))
    }

    /// Write `config.yaml`: mkdir `0o755`, file `0o644` (SPEC-CFGSCHEMA-002). Save failures
    /// surface as `ExitFileIO` (10) so the `config set` layer reports `save config: %v`.
    pub fn save(&self) -> AppResult<()> {
        let dir = config_dir();
        std::fs::create_dir_all(&dir).map_err(|e| AppError::file_io(e.to_string()))?;
        set_mode(&dir, 0o755);

        let data = serde_norway::to_string(self).map_err(|e| AppError::file_io(e.to_string()))?;
        let path = config_path();
        std::fs::write(&path, data.as_bytes()).map_err(|e| AppError::file_io(e.to_string()))?;
        set_mode(&path, 0o644);
        Ok(())
    }

    /// The provider whose entry backs the legacy single `model`/`config get model` view and
    /// [`resolve_model`](Self::resolve_model): the configured `default_provider`, else gemini
    /// (the historical default). This preserves flat-config behavior where a bare `model`
    /// applied to the default/gemini provider.
    fn model_source_provider(&self) -> &str {
        if self.default_provider.is_empty() {
            PROVIDER_GEMINI
        } else {
            &self.default_provider
        }
    }

    /// Value for `key`, or `""` for an unset field or an unknown key (SPEC-CONFIG-002 — the
    /// command layer distinguishes unset-vs-unknown; this accessor mirrors Go's `Get`).
    ///
    /// Accepts dotted `<provider>.<model|api-key|api-key-envvar>` keys, `default-provider`, the
    /// top-level image defaults, and the legacy flat aliases (`api_key` → `gemini.api-key`,
    /// `model` → the default provider's model, `provider` → `default-provider`).
    pub fn get(&self, key: &str) -> String {
        match key {
            "default-provider" | "provider" => self.default_provider.clone(),
            "default_output_dir" => self.default_output_dir.clone(),
            "aspect" => self.aspect.clone(),
            "resolution" => self.resolution.clone(),
            "quality" => self.quality.clone(),
            // Legacy flat aliases (backward compatibility).
            "api_key" => self.provider_field_get(PROVIDER_GEMINI, PField::ApiKey),
            "model" => {
                let p = self.model_source_provider().to_string();
                self.provider_field_get(&p, PField::Model)
            }
            _ => match parse_provider_key(key) {
                Some((provider, field)) => self.provider_field_get(provider, field),
                None => String::new(),
            },
        }
    }

    /// Read a per-provider field, or `""` when the provider has no entry.
    fn provider_field_get(&self, provider: &str, field: PField) -> String {
        match self.providers.get(provider) {
            Some(pc) => match field {
                PField::Model => pc.model.clone(),
                PField::ApiKey => pc.api_key.clone(),
                PField::ApiKeyEnvvar => pc.api_key_envvar.clone(),
            },
            None => String::new(),
        }
    }

    /// Set `key` to `value`; returns `false` on an unknown key (SPEC-CONFIG-003 — the command
    /// layer turns `false` into the `unknown key` usage error, exit 2). Accepts the same key
    /// vocabulary as [`get`](Self::get), including the legacy flat aliases.
    #[must_use]
    pub fn set(&mut self, key: &str, value: &str) -> bool {
        match key {
            "default-provider" | "provider" => self.default_provider = value.to_string(),
            "default_output_dir" => self.default_output_dir = value.to_string(),
            "aspect" => self.aspect = value.to_string(),
            "resolution" => self.resolution = value.to_string(),
            "quality" => self.quality = value.to_string(),
            // Legacy flat aliases (backward compatibility).
            "api_key" => self.provider_field_set(PROVIDER_GEMINI, PField::ApiKey, value),
            "model" => {
                let p = self.model_source_provider().to_string();
                self.provider_field_set(&p, PField::Model, value);
            }
            _ => match parse_provider_key(key) {
                Some((provider, field)) => {
                    let provider = provider.to_string();
                    self.provider_field_set(&provider, field, value);
                }
                None => return false,
            },
        }
        true
    }

    /// Write a per-provider field, creating the entry as needed and pruning an entry that ends
    /// up fully empty (so it never serializes as a bare `name: {}`).
    fn provider_field_set(&mut self, provider: &str, field: PField, value: &str) {
        let entry = self.providers.entry(provider.to_string()).or_default();
        match field {
            PField::Model => entry.model = value.to_string(),
            PField::ApiKey => entry.api_key = value.to_string(),
            PField::ApiKeyEnvvar => entry.api_key_envvar = value.to_string(),
        }
        if entry.is_empty() {
            self.providers.remove(provider);
        }
    }

    /// The config-implied default model for the default provider (SPEC-CFGSCHEMA-006):
    /// `providers.<default>.model` > `quality`→model tier > `""` (unset). An invalid `quality`
    /// yields SPEC-ERR-007 `invalid quality %q in config (valid: fast, high)`. The tier mapping
    /// stays in lockstep with [`gemini`]'s model constants.
    ///
    /// This is the single-model view the pre-registry selector adapter consumes; the selector
    /// still applies the per-provider compiled-in default when this is empty.
    pub fn resolve_model(&self) -> AppResult<String> {
        let p = self.model_source_provider();
        if let Some(pc) = self.providers.get(p) {
            if !pc.model.is_empty() {
                return Ok(pc.model.clone());
            }
        }
        if !self.quality.is_empty() {
            return match self.quality.as_str() {
                "fast" => Ok(gemini::FLASH_MODEL.to_string()),
                "high" => Ok(gemini::PRO_MODEL.to_string()),
                other => Err(AppError::general(format!(
                    "invalid quality {other:?} in config (valid: fast, high)"
                ))),
            };
        }
        Ok(String::new())
    }

    /// Uniform api-key resolution for any provider (SPEC-CFGSCHEMA-003), highest precedence
    /// first: inline `providers.<provider>.api-key`, then the env var named by
    /// `providers.<provider>.api-key-envvar`, then the provider's conventional default env var
    /// ([`conventional_env_var`]). Empty when none resolve. This is the single resolver every
    /// provider uses — the old `resolve_api_key`/`resolve_openrouter_api_key` split is gone.
    pub fn resolve_api_key_for(&self, provider: &str) -> String {
        if let Some(pc) = self.providers.get(provider) {
            if !pc.api_key.is_empty() {
                return pc.api_key.clone();
            }
            if !pc.api_key_envvar.is_empty() {
                if let Ok(v) = std::env::var(&pc.api_key_envvar) {
                    if !v.is_empty() {
                        return v;
                    }
                }
            }
        }
        if let Some(name) = conventional_env_var(provider) {
            if let Ok(v) = std::env::var(name) {
                if !v.is_empty() {
                    return v;
                }
            }
        }
        String::new()
    }

    /// Output dir (MCP only — the CLI ignores this, SPEC-CFGSCHEMA-005): `NABA_OUTPUT_DIR` env
    /// beats config `default_output_dir`, which beats the XDG default
    /// `<home>/.local/share/naba/images` (SPEC-CFGSCHEMA-004).
    pub fn resolve_output_dir(&self) -> String {
        if let Ok(dir) = std::env::var(ENV_OUTPUT_DIR) {
            if !dir.is_empty() {
                return dir;
            }
        }
        if !self.default_output_dir.is_empty() {
            return self.default_output_dir.clone();
        }
        default_output_dir()
    }

    /// The selector seam (SPEC-PROVIDER-007): produce the [`ConfigDefaults`] the provider
    /// selector consumes. `provider` is `default_provider` (None when unset); `model` is the
    /// **resolved** config model (SPEC-CFGSCHEMA-006), so the selector never re-derives the
    /// quality tier. An invalid config `quality` surfaces SPEC-ERR-007 here.
    pub fn to_config_defaults(&self) -> AppResult<ConfigDefaults> {
        let model = self.resolve_model()?;
        Ok(ConfigDefaults {
            provider: opt(&self.default_provider),
            model: opt(&model),
        })
    }
}

// ---------------------------------------------------------------------------
// Config auto-migration (Epic 1 — SPEC-MIGRATE-001..004): flat → nested.
// ---------------------------------------------------------------------------

/// Filename suffix for the pre-migration backup (SPEC-MIGRATE-002): `config.yaml.bak`.
const MIGRATION_BACKUP_EXT: &str = "bak";

/// Whether the on-disk YAML is in the OLD **flat** shape and needs a structural rewrite
/// (SPEC-MIGRATE-001/002).
///
/// The flat schema had top-level `api_key`/`model`/`provider`; the nested schema has
/// `providers`/`default_provider`. Migration fires when a flat-only key is present AND no nested
/// key is (so a fresh nested file, or one already migrated, never re-migrates — the idempotency
/// contract). The shared image-default keys (`aspect`/`resolution`/`quality`/`default_output_dir`)
/// exist in both shapes and never, on their own, trigger a migration.
fn needs_structural_migration(parsed: &serde_norway::Value) -> bool {
    let has_flat = ["api_key", "model", "provider"]
        .iter()
        .any(|k| parsed.get(*k).is_some());
    let has_nested = parsed.get("providers").is_some() || parsed.get("default_provider").is_some();
    has_flat && !has_nested
}

/// Transform a parsed OLD flat document into the nested [`Config`] (SPEC-MIGRATE-002).
///
/// Per-key mapping (explicit):
/// - `api_key` → `providers.gemini.api-key` — its historical Gemini-scoped meaning, regardless
///   of the old `provider` value (so a stray key on an openrouter-default config still lands
///   under gemini);
/// - `model` → `providers.<default>.model`, where `<default>` is the old `provider` value, or
///   gemini when `provider` is absent (the explicit fallback);
/// - `provider` → `default_provider`;
/// - `aspect`/`resolution`/`quality`/`default_output_dir` → preserved as the top-level image
///   defaults.
fn migrate_flat_value(parsed: &serde_norway::Value) -> Config {
    let get = |k: &str| {
        parsed
            .get(k)
            .and_then(serde_norway::Value::as_str)
            .unwrap_or("")
            .to_string()
    };

    let provider = get("provider");
    let mut cfg = Config {
        default_provider: provider.clone(),
        aspect: get("aspect"),
        resolution: get("resolution"),
        quality: get("quality"),
        default_output_dir: get("default_output_dir"),
        ..Default::default()
    };

    // api_key is historically Gemini-scoped, always → providers.gemini.api-key.
    let api_key = get("api_key");
    if !api_key.is_empty() {
        cfg.providers
            .entry(PROVIDER_GEMINI.to_string())
            .or_default()
            .api_key = api_key;
    }
    // model → the resolved default provider's entry (old `provider`, else gemini).
    let model = get("model");
    if !model.is_empty() {
        let target = if provider.is_empty() {
            PROVIDER_GEMINI.to_string()
        } else {
            provider.clone()
        };
        cfg.providers.entry(target).or_default().model = model;
    }
    cfg
}

/// `<path>.bak` — the backup sibling (full filename plus `.bak`, so `config.yaml` →
/// `config.yaml.bak`, not `config.bak`).
fn backup_path(path: &Path) -> PathBuf {
    let mut s = path.as_os_str().to_owned();
    s.push(".");
    s.push(MIGRATION_BACKUP_EXT);
    PathBuf::from(s)
}

/// Run the flat→nested structural migration against `config.yaml` (SPEC-MIGRATE-001/002).
/// Called by [`Config::load`] ahead of the read.
pub fn migrate_if_needed() -> AppResult<()> {
    migrate_file(&config_path())
}

/// Core migration engine (SPEC-MIGRATE-002/003).
///
/// Contract:
/// - **missing file** → no-op (`Ok`), no `.bak`;
/// - **empty / whitespace-only** → no-op;
/// - **malformed YAML** → no-op (skipped — [`Config::load`] surfaces the parse error);
/// - **already-nested** ([`needs_structural_migration`] == `false`) → no-op, file untouched;
/// - **flat shape** → write `<path>.bak` with the **original bytes** first, then transform the
///   document into the nested schema ([`migrate_flat_value`]) and rewrite `<path>`.
///
/// **Idempotency** (SPEC-MIGRATE-002): a migrated file has `providers`/`default_provider` and no
/// flat keys, so the second run sees the nested shape and no-ops; `.bak` is written **at most
/// once** and never clobbered — the backup always holds the true pre-migration original.
///
/// **Comment loss** (SPEC-MIGRATE-003): the serde round-trip does not preserve YAML comments;
/// this is an accepted loss, mitigated by the `.bak` backup.
fn migrate_file(path: &Path) -> AppResult<()> {
    let data = match std::fs::read_to_string(path) {
        Ok(d) => d,
        // Missing file: nothing to migrate (matches load()'s not-found → zero Config).
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        // Any other hard read error: best-effort skip — load() re-reads and surfaces it.
        Err(_) => return Ok(()),
    };
    // Empty / whitespace-only file: no document to migrate.
    if data.trim().is_empty() {
        return Ok(());
    }
    // Malformed YAML: skip the rewrite; load()'s parse re-surfaces the error, so we neither
    // crash nor destroy the user's (repairable) file.
    let value: serde_norway::Value = match serde_norway::from_str(&data) {
        Ok(v) => v,
        Err(_) => return Ok(()),
    };
    // Already nested / nothing flat to migrate: leave the file byte-identical.
    if !needs_structural_migration(&value) {
        return Ok(());
    }
    // Flat shape: back the original up, then transform + rewrite into the nested schema.
    let bak = backup_path(path);
    std::fs::write(&bak, data.as_bytes()).map_err(|e| AppError::file_io(e.to_string()))?;
    set_mode(&bak, 0o644);
    let migrated = migrate_flat_value(&value);
    let rewritten =
        serde_norway::to_string(&migrated).map_err(|e| AppError::file_io(e.to_string()))?;
    std::fs::write(path, rewritten.as_bytes()).map_err(|e| AppError::file_io(e.to_string()))?;
    set_mode(path, 0o644);
    Ok(())
}

/// `Some(s)` for a non-empty string, else `None`.
fn opt(s: &str) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}

/// Set unix mode on a path; best-effort (a perms failure never fails the save on its own).
#[cfg(unix)]
fn set_mode(path: &std::path::Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;
    let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode));
}

#[cfg(not(unix))]
fn set_mode(_path: &std::path::Path, _mode: u32) {}

// ---------------------------------------------------------------------------
// config get / config set command backing (SPEC-CONFIG-002/003, SPEC-ERR-008/009).
// ---------------------------------------------------------------------------

/// Backing for `config get <key>` (SPEC-CONFIG-002): returns the value to print. Load error →
/// `ExitGeneral` `load config: %v`; unset key → `ExitGeneral` `key %q is not set` (SPEC-ERR-008).
/// An unknown key is also "not set" (Go's `Get` returns `""` for both).
pub fn get_value(key: &str) -> AppResult<String> {
    let cfg = Config::load().map_err(|e| AppError::general(format!("load config: {e}")))?;
    let value = cfg.get(key);
    if value.is_empty() {
        return Err(AppError::general(format!(
            "key {key:?} is not set\n\nValid keys: {}",
            valid_keys_joined()
        )));
    }
    Ok(value)
}

/// Backing for `config set <key> <value>` (SPEC-CONFIG-003): loads, sets, saves. Load error →
/// `ExitGeneral` `load config: %v`; unknown key → `ExitUsage` `unknown key %q` (SPEC-ERR-009,
/// exit 2); save error → `ExitFileIO` `save config: %v` (exit 10).
pub fn set_value(key: &str, value: &str) -> AppResult<()> {
    let mut cfg = Config::load().map_err(|e| AppError::general(format!("load config: {e}")))?;
    if !cfg.set(key, value) {
        return Err(AppError::usage(format!(
            "unknown key {key:?}\n\nValid keys: {}",
            valid_keys_joined()
        )));
    }
    cfg.save()
        .map_err(|e| AppError::file_io(format!("save config: {e}")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::exit;
    use crate::provider::{gemini, openrouter};
    use std::sync::{Mutex, MutexGuard};

    // Config path + several resolvers read process-global env (NABA_CONFIG_DIR, HOME,
    // GEMINI_API_KEY, ...). Serialize env-touching tests so they don't race.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    /// A test scope with an isolated NABA_CONFIG_DIR tempdir (no real home is touched) and the
    /// env lock held. Cleans the env vars it sets on drop.
    struct Scope {
        _guard: MutexGuard<'static, ()>,
        dir: PathBuf,
    }

    impl Scope {
        fn new(name: &str) -> Self {
            let guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
            let mut dir = std::env::temp_dir();
            dir.push(format!("naba-cfg-test-{}-{}", std::process::id(), name));
            let _ = std::fs::remove_dir_all(&dir);
            std::env::set_var(ENV_CONFIG_DIR, &dir);
            // Clear key/output env so tests control precedence explicitly.
            std::env::remove_var(ENV_API_KEY);
            std::env::remove_var(ENV_OPENROUTER_API_KEY);
            std::env::remove_var(ENV_OUTPUT_DIR);
            Scope { _guard: guard, dir }
        }
    }

    impl Drop for Scope {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.dir);
            std::env::remove_var(ENV_CONFIG_DIR);
            std::env::remove_var(ENV_API_KEY);
            std::env::remove_var(ENV_OPENROUTER_API_KEY);
            std::env::remove_var(ENV_OUTPUT_DIR);
        }
    }

    /// Write raw bytes to `config.yaml` in the current NABA_CONFIG_DIR (creating the dir).
    fn write_config_raw(contents: &str) -> PathBuf {
        let path = config_path();
        std::fs::create_dir_all(config_dir()).unwrap();
        std::fs::write(&path, contents.as_bytes()).unwrap();
        path
    }

    #[test]
    fn config_path_honors_env_override() {
        let s = Scope::new("path");
        assert_eq!(config_dir(), s.dir);
        assert_eq!(config_path(), s.dir.join("config.yaml"));
    }

    // SPEC-DIRS-001: config_dir precedence NABA_CONFIG_DIR > $XDG_CONFIG_HOME/naba > ~/.config/naba.
    #[test]
    fn config_dir_precedence_naba_then_xdg_then_default() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let prev_naba = std::env::var_os(ENV_CONFIG_DIR);
        let prev_xdg = std::env::var_os("XDG_CONFIG_HOME");
        let prev_home = std::env::var_os("HOME");

        std::env::set_var(ENV_CONFIG_DIR, "/explicit/naba-cfg");
        std::env::set_var("XDG_CONFIG_HOME", "/xdg");
        std::env::set_var("HOME", "/home/tester");
        assert_eq!(config_dir(), PathBuf::from("/explicit/naba-cfg"));

        std::env::remove_var(ENV_CONFIG_DIR);
        assert_eq!(config_dir(), PathBuf::from("/xdg/naba"));

        std::env::remove_var("XDG_CONFIG_HOME");
        assert_eq!(config_dir(), PathBuf::from("/home/tester/.config/naba"));

        match prev_naba {
            Some(v) => std::env::set_var(ENV_CONFIG_DIR, v),
            None => std::env::remove_var(ENV_CONFIG_DIR),
        }
        match prev_xdg {
            Some(v) => std::env::set_var("XDG_CONFIG_HOME", v),
            None => std::env::remove_var("XDG_CONFIG_HOME"),
        }
        match prev_home {
            Some(v) => std::env::set_var("HOME", v),
            None => std::env::remove_var("HOME"),
        }
    }

    #[test]
    fn load_missing_file_is_zero_config() {
        let _s = Scope::new("missing");
        let cfg = Config::load().unwrap();
        assert_eq!(cfg, Config::default());
    }

    // A nested config round-trips through save/load with per-provider entries preserved.
    #[test]
    fn save_then_load_round_trips_nested_schema() {
        let _s = Scope::new("roundtrip");
        let mut cfg = Config {
            default_provider: "openrouter".into(),
            default_output_dir: "/tmp/out".into(),
            aspect: "16:9".into(),
            resolution: "2K".into(),
            quality: "high".into(),
            ..Default::default()
        };
        cfg.providers.insert(
            "gemini".into(),
            ProviderConfig {
                model: "gemini-3-pro-image".into(),
                api_key: "sk-gem".into(),
                ..Default::default()
            },
        );
        cfg.providers.insert(
            "openrouter".into(),
            ProviderConfig {
                api_key_envvar: "MY_OR_KEY".into(),
                ..Default::default()
            },
        );
        cfg.save().unwrap();
        let loaded = Config::load().unwrap();
        assert_eq!(loaded, cfg);
        // The api-key rename is honored on the wire.
        let text = std::fs::read_to_string(config_path()).unwrap();
        assert!(text.contains("api-key: sk-gem"), "got:\n{text}");
        assert!(text.contains("api-key-envvar: MY_OR_KEY"), "got:\n{text}");
        assert!(
            text.contains("default_provider: openrouter"),
            "got:\n{text}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn save_sets_dir_755_and_file_644() {
        use std::os::unix::fs::PermissionsExt;
        let _s = Scope::new("perms");
        let mut cfg = Config::default();
        assert!(cfg.set("gemini.model", "m"));
        cfg.save().unwrap();
        let dmode = std::fs::metadata(config_dir())
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        let fmode = std::fs::metadata(config_path())
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(dmode, 0o755);
        assert_eq!(fmode, 0o644);
    }

    // Dotted per-provider keys + top-level defaults + default-provider all get/set round-trip.
    #[test]
    fn get_and_set_dotted_and_toplevel_keys() {
        let mut cfg = Config::default();
        for key in [
            "default-provider",
            "gemini.model",
            "gemini.api-key",
            "gemini.api-key-envvar",
            "openrouter.model",
            "openrouter.api-key",
            "openrouter.api-key-envvar",
            "default_output_dir",
            "aspect",
            "resolution",
            "quality",
        ] {
            assert!(cfg.set(key, "v"), "set {key} should succeed");
            assert_eq!(cfg.get(key), "v", "get {key}");
        }
    }

    // Legacy flat aliases still work: api_key → gemini.api-key, model → default provider's model,
    // provider → default-provider (SPEC-CONFIG backward compatibility).
    #[test]
    fn legacy_flat_aliases_map_onto_nested() {
        let mut cfg = Config::default();
        assert!(cfg.set("api_key", "sk-legacy"));
        assert_eq!(cfg.get("gemini.api-key"), "sk-legacy");
        assert_eq!(cfg.get("api_key"), "sk-legacy");

        assert!(cfg.set("model", "legacy-model"));
        // default_provider unset → the alias targets gemini.
        assert_eq!(cfg.get("gemini.model"), "legacy-model");
        assert_eq!(cfg.get("model"), "legacy-model");

        assert!(cfg.set("provider", "openrouter"));
        assert_eq!(cfg.get("default-provider"), "openrouter");
        // Now `model` alias tracks the new default provider.
        assert!(cfg.set("model", "or-model"));
        assert_eq!(cfg.get("openrouter.model"), "or-model");
    }

    #[test]
    fn set_unknown_key_returns_false() {
        let mut cfg = Config::default();
        assert!(!cfg.set("bogus", "v"));
        assert!(!cfg.set("bedrock.model", "v")); // unknown provider (not yet registered)
        assert!(!cfg.set("gemini.bogus", "v")); // unknown field
        assert_eq!(cfg.get("bogus"), "");
    }

    #[test]
    fn setting_empty_value_prunes_empty_provider_entry() {
        let mut cfg = Config::default();
        assert!(cfg.set("gemini.model", ""));
        // A fully-empty entry is dropped so it never serializes as `gemini: {}`.
        assert!(!cfg.providers.contains_key("gemini"));
    }

    #[test]
    fn valid_keys_exact_set_and_order() {
        assert_eq!(
            valid_keys(),
            vec![
                "default-provider",
                "gemini.model",
                "gemini.api-key",
                "gemini.api-key-envvar",
                "openrouter.model",
                "openrouter.api-key",
                "openrouter.api-key-envvar",
                "default_output_dir",
                "aspect",
                "resolution",
                "quality",
            ]
        );
    }

    #[test]
    fn resolve_model_precedence() {
        // Per-provider model beats quality.
        let mut cfg = Config {
            default_provider: "gemini".into(),
            quality: "high".into(),
            ..Default::default()
        };
        cfg.providers.insert(
            "gemini".into(),
            ProviderConfig {
                model: "explicit-model".into(),
                ..Default::default()
            },
        );
        assert_eq!(cfg.resolve_model().unwrap(), "explicit-model");
        // quality maps to tier when no per-provider model.
        let cfg = Config {
            quality: "high".into(),
            ..Default::default()
        };
        assert_eq!(cfg.resolve_model().unwrap(), gemini::PRO_MODEL);
        let cfg = Config {
            quality: "fast".into(),
            ..Default::default()
        };
        assert_eq!(cfg.resolve_model().unwrap(), gemini::FLASH_MODEL);
        // neither → empty.
        assert_eq!(Config::default().resolve_model().unwrap(), "");
    }

    #[test]
    fn resolve_model_invalid_quality_is_err_007() {
        let cfg = Config {
            quality: "medium".into(),
            ..Default::default()
        };
        let err = cfg.resolve_model().unwrap_err();
        assert_eq!(
            err.message,
            "invalid quality \"medium\" in config (valid: fast, high)"
        );
    }

    // SPEC-CFGSCHEMA-003: uniform api-key resolution — inline > custom envvar > conventional env.
    #[test]
    fn resolve_api_key_uniform_precedence() {
        let _s = Scope::new("apikey");
        // Inline wins over everything (including the conventional env var).
        let mut cfg = Config::default();
        cfg.providers.insert(
            "gemini".into(),
            ProviderConfig {
                api_key: "inline-key".into(),
                ..Default::default()
            },
        );
        std::env::set_var(ENV_API_KEY, "env-key");
        assert_eq!(cfg.resolve_api_key_for("gemini"), "inline-key");

        // No inline → the custom envvar named by api-key-envvar.
        let mut cfg = Config::default();
        cfg.providers.insert(
            "gemini".into(),
            ProviderConfig {
                api_key_envvar: "CUSTOM_GEMINI".into(),
                ..Default::default()
            },
        );
        std::env::set_var("CUSTOM_GEMINI", "custom-env-key");
        assert_eq!(cfg.resolve_api_key_for("gemini"), "custom-env-key");
        std::env::remove_var("CUSTOM_GEMINI");

        // No inline, no custom → the conventional default env var.
        let cfg = Config::default();
        std::env::set_var(ENV_API_KEY, "conventional-key");
        assert_eq!(cfg.resolve_api_key_for("gemini"), "conventional-key");
        std::env::remove_var(ENV_API_KEY);
        // Nothing set → empty.
        assert_eq!(cfg.resolve_api_key_for("gemini"), "");
    }

    // SPEC-CFGSCHEMA-003: openrouter now has a first-class inline api-key (no special-case).
    #[test]
    fn resolve_api_key_openrouter_inline_and_env() {
        let _s = Scope::new("orkey");
        let mut cfg = Config::default();
        cfg.providers.insert(
            "openrouter".into(),
            ProviderConfig {
                api_key: "or-inline".into(),
                ..Default::default()
            },
        );
        assert_eq!(cfg.resolve_api_key_for("openrouter"), "or-inline");
        // env fallback when no inline.
        let cfg = Config::default();
        assert_eq!(cfg.resolve_api_key_for("openrouter"), "");
        std::env::set_var(ENV_OPENROUTER_API_KEY, "or-env");
        assert_eq!(cfg.resolve_api_key_for("openrouter"), "or-env");
    }

    #[test]
    fn resolve_output_dir_precedence() {
        let _s = Scope::new("outdir");
        let cfg = Config {
            default_output_dir: "/cfg/out".into(),
            ..Default::default()
        };
        assert_eq!(cfg.resolve_output_dir(), "/cfg/out");
        std::env::set_var(ENV_OUTPUT_DIR, "/env/out");
        assert_eq!(cfg.resolve_output_dir(), "/env/out");
        std::env::remove_var(ENV_OUTPUT_DIR);
        let cfg = Config::default();
        let got = cfg.resolve_output_dir();
        assert!(
            got.ends_with(".local/share/naba/images") || got.is_empty(),
            "got {got}"
        );
    }

    #[test]
    fn to_config_defaults_maps_default_provider_and_resolved_model() {
        // default_provider + quality → resolved model tier; provider passed through.
        let cfg = Config {
            default_provider: "gemini".into(),
            quality: "high".into(),
            ..Default::default()
        };
        let defaults = cfg.to_config_defaults().unwrap();
        assert_eq!(defaults.provider.as_deref(), Some("gemini"));
        assert_eq!(defaults.model.as_deref(), Some(gemini::PRO_MODEL));
        // empty config → both None.
        let defaults = Config::default().to_config_defaults().unwrap();
        assert_eq!(defaults.provider, None);
        assert_eq!(defaults.model, None);
    }

    #[test]
    fn get_value_unset_is_err_008_exit_1() {
        let _s = Scope::new("get-unset");
        let err = get_value("aspect").unwrap_err();
        assert_eq!(err.code, exit::GENERAL);
        assert_eq!(
            err.message,
            format!(
                "key \"aspect\" is not set\n\nValid keys: {}",
                valid_keys_joined()
            )
        );
    }

    #[test]
    fn get_value_returns_set_value() {
        let _s = Scope::new("get-set");
        set_value("gemini.model", "gemini-3-pro-image").unwrap();
        assert_eq!(get_value("gemini.model").unwrap(), "gemini-3-pro-image");
        // Legacy alias reads back the same value (default provider = gemini).
        assert_eq!(get_value("model").unwrap(), "gemini-3-pro-image");
    }

    #[test]
    fn set_value_unknown_key_is_err_009_exit_2() {
        let _s = Scope::new("set-unknown");
        let err = set_value("bogus", "v").unwrap_err();
        assert_eq!(err.code, exit::USAGE);
        assert_eq!(
            err.message,
            format!(
                "unknown key \"bogus\"\n\nValid keys: {}",
                valid_keys_joined()
            )
        );
    }

    #[test]
    fn set_value_then_get_value_round_trip_via_disk() {
        let _s = Scope::new("cmd-roundtrip");
        set_value("default-provider", "openrouter").unwrap();
        set_value("aspect", "16:9").unwrap();
        set_value("openrouter.model", "bytedance-seed/seedream-4.5").unwrap();
        assert_eq!(get_value("default-provider").unwrap(), "openrouter");
        assert_eq!(get_value("aspect").unwrap(), "16:9");
        assert_eq!(
            get_value("openrouter.model").unwrap(),
            "bytedance-seed/seedream-4.5"
        );
    }

    // -------------------------------------------------------------------
    // Epic 1 — flat→nested auto-migration (SPEC-MIGRATE-001..004).
    // -------------------------------------------------------------------

    // SPEC-MIGRATE-002: a flat config is backed up to `.bak` (original bytes) and rewritten into
    // the nested schema; comments are lost (accepted, SPEC-MIGRATE-003); a second run is a no-op.
    #[test]
    fn flat_config_migrates_backs_up_and_is_idempotent() {
        let _s = Scope::new("migrate-structural");
        let original = "# hand-edited (comment lost on rewrite, SPEC-MIGRATE-003)\napi_key: sk-x   # inline\nmodel: gemini-3-pro-image\nprovider: gemini\nquality: high\naspect: '16:9'\n";
        let path = write_config_raw(original);
        let orig_bytes = std::fs::read(&path).unwrap();

        migrate_file(&path).unwrap();

        // .bak holds the true original bytes.
        let bak = backup_path(&path);
        assert!(bak.exists(), ".bak must be written before the rewrite");
        assert_eq!(std::fs::read(&bak).unwrap(), orig_bytes, ".bak = original");

        // The rewritten file is nested + comment-free.
        let rewritten = std::fs::read_to_string(&path).unwrap();
        assert!(
            !rewritten.contains("# hand-edited"),
            "comments lost (accepted)"
        );
        assert!(!rewritten.contains("api_key:"), "flat api_key key gone");
        let cfg: Config = serde_norway::from_str(&rewritten).unwrap();
        assert_eq!(cfg.default_provider, "gemini");
        assert_eq!(cfg.get("gemini.api-key"), "sk-x");
        assert_eq!(cfg.get("gemini.model"), "gemini-3-pro-image");
        assert_eq!(cfg.aspect, "16:9");
        assert_eq!(cfg.quality, "high");

        // Second run: nested shape has no flat key → no-op, .bak not clobbered.
        let after_first = std::fs::read(&path).unwrap();
        migrate_file(&path).unwrap();
        assert_eq!(
            std::fs::read(&path).unwrap(),
            after_first,
            "idempotent no-op"
        );
        assert_eq!(std::fs::read(&bak).unwrap(), orig_bytes, ".bak preserved");
    }

    // SPEC-MIGRATE-002 (the plan's required case): an openrouter-default flat config with a stray
    // gemini `api_key` — the key must land under gemini, NOT openrouter.
    #[test]
    fn migrate_openrouter_default_with_stray_gemini_api_key() {
        let _s = Scope::new("migrate-stray");
        let original = "provider: openrouter\napi_key: sk-stray\n";
        let path = write_config_raw(original);

        migrate_file(&path).unwrap();

        let cfg: Config = serde_norway::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(cfg.default_provider, "openrouter");
        // The stray key lands under gemini (historical scope), not openrouter.
        assert_eq!(cfg.get("gemini.api-key"), "sk-stray");
        assert_eq!(cfg.get("openrouter.api-key"), "");
        assert!(!cfg.providers.contains_key("openrouter"));
    }

    // A flat `model` with no `provider` maps under the gemini fallback entry.
    #[test]
    fn migrate_flat_model_without_provider_targets_gemini() {
        let _s = Scope::new("migrate-model-only");
        let original = "model: cfg-model\n";
        let path = write_config_raw(original);
        migrate_file(&path).unwrap();
        let cfg: Config = serde_norway::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(cfg.default_provider, "");
        assert_eq!(cfg.get("gemini.model"), "cfg-model");
    }

    // SPEC-MIGRATE-002: graceful on missing / empty / malformed / already-nested inputs.
    #[test]
    fn migration_is_graceful_on_edge_inputs() {
        let _s = Scope::new("migrate-graceful");
        let path = config_path();
        std::fs::create_dir_all(config_dir()).unwrap();

        // Missing file → no-op, no .bak, no crash.
        let _ = std::fs::remove_file(&path);
        migrate_file(&path).unwrap();
        assert!(!backup_path(&path).exists());
        assert!(!path.exists());

        // Empty file → no-op, still empty, no .bak.
        std::fs::write(&path, b"").unwrap();
        migrate_file(&path).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"");
        assert!(!backup_path(&path).exists());

        // Malformed YAML → no-op (skipped), file unchanged, no .bak.
        let bad = "api_key: [unterminated\nprovider: gemini\n";
        std::fs::write(&path, bad.as_bytes()).unwrap();
        migrate_file(&path).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), bad);
        assert!(!backup_path(&path).exists());

        // Already-nested → no-op even though it has provider entries.
        let good =
            "default_provider: gemini\nproviders:\n  gemini:\n    model: gemini-3-pro-image\n";
        std::fs::write(&path, good.as_bytes()).unwrap();
        migrate_file(&path).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), good);
        assert!(!backup_path(&path).exists());
    }

    // SPEC-MIGRATE-001/003: an image-defaults-only config (aspect/resolution/quality) is NOT flat
    // — those keys exist in the nested schema too, so a normal load leaves it byte-identical.
    #[test]
    fn image_defaults_only_config_not_migrated() {
        let _s = Scope::new("migrate-image-defaults");
        let original = "# keep me\naspect: '16:9'\nresolution: 2K\nquality: high\n";
        let path = write_config_raw(original);
        let before = std::fs::read(&path).unwrap();
        Config::load().unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), before, "byte-identical load");
        assert!(!backup_path(&path).exists(), "no .bak on a non-flat load");
    }

    // End-to-end: loading a flat config migrates on disk AND resolves correctly through the
    // nested schema (the value a command would see).
    #[test]
    fn load_migrates_flat_and_resolves_nested() {
        let _s = Scope::new("migrate-load-e2e");
        let original = "api_key: sk-old\nmodel: cfg-model\n";
        write_config_raw(original);
        let cfg = Config::load().unwrap();
        assert_eq!(cfg.get("gemini.api-key"), "sk-old");
        assert_eq!(cfg.resolve_api_key_for("gemini"), "sk-old");
        assert_eq!(cfg.resolve_model().unwrap(), "cfg-model");
        // The on-disk file is now nested.
        let text = std::fs::read_to_string(config_path()).unwrap();
        assert!(text.contains("providers:"), "migrated on disk:\n{text}");
        assert!(backup_path(&config_path()).exists(), ".bak written");
    }

    #[test]
    fn conventional_env_var_is_provider_aware() {
        assert_eq!(conventional_env_var("gemini"), Some(ENV_API_KEY));
        assert_eq!(
            conventional_env_var("openrouter"),
            Some(ENV_OPENROUTER_API_KEY)
        );
        assert_eq!(conventional_env_var("bedrock"), None);
        // Sanity: the openrouter default model constant is reachable (registry extensibility).
        assert!(!openrouter::DEFAULT_MODEL.is_empty());
    }
}
