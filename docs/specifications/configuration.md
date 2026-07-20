# naba — Configuration Specification

Clause IDs (`SPEC-<AREA>-NNN`) are stable and are never renumbered; append only.

## §6 Config schema & precedence (SPEC-CFGSCHEMA)

- **SPEC-CFGSCHEMA-001** [PINNED+NEW] Config file `config.yaml` at `NABA_CONFIG_DIR` (if set)
  else `<home>/.config/naba`. The schema is **nested per-provider** (Epic 1). Top level (YAML,
  all omitempty): `default_provider`, a `providers` map keyed by provider name, and the image
  defaults `default_output_dir`, `aspect`, `resolution`, `quality`. Each `providers.<name>`
  entry carries `model`, `api-key`, `api-key-envvar` (all omitempty). The `config get`/`config
  set` addressable key set (order pinned, drives the `Valid keys:` error lines): `default-provider`,
  each known provider's `<provider>.model` / `<provider>.api-key` / `<provider>.api-key-envvar`
  (providers: `gemini`, `openrouter`, `bedrock`), then `default_output_dir`, `aspect`, `resolution`,
  `quality`. The legacy flat keys `api_key`, `model`, `provider` remain accepted as **aliases**
  (`api_key` → `gemini.api-key`, `model` → the default provider's `model`, `provider` →
  `default-provider`) for backward compatibility, but are not advertised in the valid-keys list.
- **SPEC-CFGSCHEMA-002** [PINNED] Missing config file → zero-value config, no error. `Save()`
  mkdir `0o755`, file `0o644`.
- **SPEC-CFGSCHEMA-003** [PINNED] **Uniform api-key precedence** (one resolver for every
  provider, Epic 1): inline `providers.<name>.api-key` > the env var named by
  `providers.<name>.api-key-envvar` > the provider's conventional default env var
  (`GEMINI_API_KEY` for gemini, `OPENROUTER_API_KEY` for openrouter, `AWS_BEARER_TOKEN_BEDROCK`
  for bedrock's api-key path — SPEC-PROVIDER-013). The conventional env-var
  names are centralized in one place. Note this **reverses the old flat env-over-config order**:
  an inline config `api-key` now beats the conventional env var (an explicit per-provider
  credential is the most specific source). OpenRouter is no longer special-cased — it has a
  first-class inline `api-key`/`api-key-envvar` like every provider (the old "no
  `openrouter_api_key` config key" carve-out is dropped).
- **SPEC-CFGSCHEMA-004** [PINNED] Output-dir precedence: `NABA_OUTPUT_DIR` env > config
  `default_output_dir` > XDG default `<home>/.local/share/naba/images`.
- **SPEC-CFGSCHEMA-005** [PINNED] **CLI-vs-MCP output-dir asymmetry.** The **CLI** image
  commands do **NOT** consult `NABA_OUTPUT_DIR`/`default_output_dir`/the XDG default — they
  write to `-o` (file or dir) or auto-name in **CWD**. `NABA_OUTPUT_DIR` and the XDG default
  are consumed **only by the MCP server**. Preserve this asymmetry exactly.
- **SPEC-CFGSCHEMA-006** [PINNED] **Per-provider default model** (Epic 1). Each provider
  designates its own default model; when `providers.<name>.model` is absent the selector
  resolves it to that provider's compiled-in default (`gemini::DEFAULT_MODEL`,
  `openrouter::DEFAULT_MODEL`; later providers register their own) — no provider is ever
  model-less. Config model precedence for the default provider (`ResolveModel`):
  `providers.<default>.model` > `quality`→model tier > unset. Invalid config `quality` →
  `"invalid quality %q in config (valid: fast, high)"`. Full CLI model precedence: `--model`
  (set, non-empty) > `--quality` (set) > config `ResolveModel` > provider default.

---

## §10 Config migration (SPEC-MIGRATE)

- **SPEC-MIGRATE-001** [NEW] The flat→nested schema change (Epic 1) is a **STRUCTURAL**
  migration, applied automatically on load. The old flat shape (top-level
  `api_key`/`model`/`provider`) is detected and rewritten into the nested schema. Per-key
  mapping: `api_key` → `providers.gemini.api-key` (its historical Gemini scope, regardless of
  the old `provider` value); `model` → `providers.<default>.model`, where `<default>` is the
  old `provider` value or `gemini` when absent; `provider` → `default_provider`;
  `aspect`/`resolution`/`quality`/`default_output_dir` → preserved as the top-level image
  defaults. An image-defaults-only config (no `api_key`/`model`/`provider`) is already
  schema-valid and is **not** rewritten (byte-identical, comments intact).
- **SPEC-MIGRATE-002** [NEW] The structural rewrite writes a `config.yaml.bak` backup with the
  **original bytes** first, then transforms + rewrites `config.yaml`. It is **idempotent** (a
  migrated file has `providers`/`default_provider` and no flat keys, so a second load no-ops
  and the `.bak` is never clobbered) and graceful on empty/missing/malformed/already-nested
  inputs (no data loss, no crash).
- **SPEC-MIGRATE-003** [NEW/ACCEPTED] The structural rewrite **loses YAML comments** (serde
  round-trip does not preserve them). This is an accepted loss, mitigated by the `.bak` backup.
  Documented here so it is not a surprise.
- **SPEC-MIGRATE-004** [NEW] YAML crate: use `serde_norway`/`yaml_serde`. **`serde_yml` is
  forbidden** (RUSTSEC-2025-0068).
