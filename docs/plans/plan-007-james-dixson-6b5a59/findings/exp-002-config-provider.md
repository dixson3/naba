# Exp 002 — config schema, migration, provider registry

## Config (`src/config.rs`)

- **Flat, all-`String` struct** (`config.rs:58-79`): `api_key` (Gemini-only), `model`,
  `default_output_dir`, `aspect`, `resolution`, `quality`, `provider`. `serde` `omitempty`-style skip.
- **`VALID_KEYS`** 7 entries (`config.rs:85-93`); order is load-bearing (drives error lines). Hand-written
  `get`/`set` (`config.rs:194-222`), `get_value`/`set_value` (`config.rs:404-431`, SPEC-ERR-008/009).
- **On-disk:** YAML via **`serde_norway`** (serde_yaml fork; `serde_yml` forbidden RUSTSEC-2025-0068).
  `config.yaml` at `config_dir()` = `NABA_CONFIG_DIR` > `$XDG_CONFIG_HOME/naba` > `~/.config/naba`.
- **`save()`** re-serializes the whole struct → **drops comments/formatting** (comment preservation only
  on the no-rewrite load path). `load()` runs `migrate_if_needed()` first.

## Migration seam (already built, `config.rs:288-375`)

- Two-function seam: `needs_structural_migration(&Value) -> bool` — **currently hardcoded `false`** (the
  extension point) — and `migrate_file(path, needs)` — the engine.
- `migrate_file`: missing/empty/malformed → no-op; if `needs`==true → **write `config.yaml.bak` with the
  ORIGINAL bytes first**, then round-trip. **Idempotent** (`.bak` written at most once), `.bak` machinery done.
- **Gap for flat→nested:** `migrate_file` currently round-trips through the SAME `Config` type, so a genuine
  schema change needs a real transform (custom flat→nested map, or a versioned deserialize), not just a
  re-serialize. Implement `needs_structural_migration` to detect the flat shape.

## Resolution + env

- `resolve_api_key` (GEMINI_API_KEY env > config `api_key`), `resolve_openrouter_api_key` (env-only, no
  config key), `resolve_model` (model > quality→tier). Env constants `config.rs:45-54`.
- **Duplication risk:** `EnvKeys::from_env` (`select.rs:115-120`) hardcodes the literal env-var names again
  (not the config.rs constants) — the new uniform api-key resolution should centralize this.

## Provider "registry" = ~9 hardcoded edit sites (no abstraction)

Adding **Bedrock** touches (all in `src/provider/select.rs` unless noted):
1. `PROVIDER_BEDROCK` const; 2. `validate_provider` match + error string; 3. `EnvKeys` + `from_env` +
`*_present`; 4. `autodetect` tuple match (2-bool → 3-key); 5. `resolve_selection` per-provider arm;
6. `build_provider` match; 7. `missing_key_error`; 8. new `src/provider/bedrock.rs` impl `Provider` +
`pub mod bedrock` in `mod.rs`; 9. config env constant (+ optional Config field/VALID_KEYS).
→ **This nested-schema work should introduce a real provider registry abstraction** to avoid the 9-site
shotgun for every future provider.

- **`Provider` trait** (`mod.rs:154-165`): `name`, async `generate`, async `list_models`; `ModelInfo{id}`.
  gemini/openrouter `list_models` already implemented (GET models endpoint) → `naba models` has a foundation.
  Constructors take `(api_key, model)` — Bedrock should mirror.

## SPEC docs to update

- Root **`SPEC.md`**: §5 SPEC-PROVIDER (001 "two providers"→three; 007/008 precedence/autodetect),
  §6 SPEC-CFGSCHEMA (001 flat 7-key set + order; 003 "no openrouter_api_key key" — both rewritten by nested
  schema), §10 SPEC-MIGRATE (001 "additive-optional" → reclassify flat→nested as **structural**; 002/003
  `.bak` path already spec'd), §3.8 SPEC-CONFIG.
- **`docs/specifications/IG/configuration.md`**: format example YAML + valid-keys list (still lists Go's 6
  keys) + auth/model resolution order — all need the nested schema.

## Design implications

- (a) Nested `{default_provider, providers:{name:{model, api-key|api-key-envvar}}, image defaults}` needs a
  new struct + either dotted-key `config get/set` addressing (`config set gemini.model X`) or a redesigned
  command surface; rework `to_config_defaults`/`resolve_*` (assume single global provider/model today).
- (b) Auto-migration seam exists + tested; implement the predicate + a real transform.
- (c) Introduce a provider-registry abstraction as part of this work (not 9-site edits per provider).
