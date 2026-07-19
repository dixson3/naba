# Implementation Guide: Configuration Management

## 1. Overview

Configuration in naba follows a layered resolution model: environment variables take precedence over config file values. The config file lives at `~/.config/naba/config.yaml` (overridable via `NABA_CONFIG_DIR` env var).

## 2. Use Cases

| ID | Name | Actor | Preconditions | Flow | Postconditions |
|----|------|-------|---------------|------|----------------|
| UC-009 | Set API key in config | CLI user | None | 1. User runs `naba config set api_key <key>` 2. Config loaded (or created) 3. Key validated against ValidKeys 4. Value written to YAML 5. Confirmation printed | Config file contains api_key; directory created if needed |
| UC-010 | Get config value | CLI user | Key previously set | 1. User runs `naba config get model` 2. Config loaded from disk 3. Value retrieved by key 4. Printed to stdout | Value on stdout; exit 1 if key not set |
| UC-011 | Override config dir | CI/testing | NABA_CONFIG_DIR env var set | 1. Set `NABA_CONFIG_DIR=/tmp/naba-test` 2. Config operations use overridden path | Config file at custom path |

## 3. Implementation Notes

### Config File Format

The schema is **nested per-provider**: a top-level `default_provider`, a `providers` map keyed
by provider name (each entry `{ model, api-key, api-key-envvar }`), plus the top-level image
defaults.

```yaml
default_provider: gemini
providers:
  gemini:
    model: gemini-3.1-flash-image
    api-key: your-gemini-api-key       # inline (optional; env var also works)
  openrouter:
    model: google/gemini-3.1-flash-image-preview
    api-key-envvar: MY_OPENROUTER_KEY  # read the key from a custom env var
default_output_dir: "~/images"
aspect: "16:9"
resolution: "2K"
quality: "high"
```

An **old flat** config (`api_key` / `model` / `provider` at the top level) is **auto-migrated**
to this nested shape on load: the original is backed up to `config.yaml.bak` first, then the file
is rewritten (comments are lost — see the `.bak` backup). Migration is idempotent.

### Valid Keys

`config get`/`config set` accept dotted addressing:

- `default-provider`
- `<provider>.model`, `<provider>.api-key`, `<provider>.api-key-envvar` — provider ∈ `gemini`,
  `openrouter`
- `default_output_dir`, `aspect`, `resolution`, `quality`

`aspect` / `resolution` are imageConfig defaults (see
[image-generation.md](image-generation.md) for valid values); `quality` is a model alias
(`fast` → flash, `high` → pro). The legacy flat keys `api_key`, `model`, `provider` still work
as aliases (`api_key` → `gemini.api-key`, `model` → the default provider's model, `provider` →
`default-provider`) for backward compatibility.

`config get` / `config set` accept `--json` and emit a `{status, key, value}` envelope; a piped
(non-TTY) invocation auto-enables it (SPEC-GLOBAL-003).

### Auth Resolution Order

Uniform across every provider (highest first):

1. Inline `providers.<provider>.api-key` in the config file
2. The env var **named by** `providers.<provider>.api-key-envvar`
3. The provider's conventional default env var (`GEMINI_API_KEY`, `OPENROUTER_API_KEY`)
4. Empty string (triggers ExitAuth error in commands)

Note this is inline-first: an inline config `api-key` beats the conventional env var.

### Model Resolution Order

Each provider designates its own default model; an absent `providers.<name>.model` resolves to
that provider's compiled-in default, so no provider is ever model-less. Precedence
(highest first):

1. `--model / -m` flag (requires `--provider` on the CLI)
2. `--quality` flag (`fast`/`high` alias → model id, Gemini)
3. `providers.<default_provider>.model` from config
4. `quality` from config (tier alias)
5. The selected provider's built-in default model (e.g. gemini `gemini-3.1-flash-image`,
   openrouter `google/gemini-3.1-flash-image-preview`)
