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

```yaml
api_key: "your-gemini-api-key"
model: "gemini-3.1-flash-image"
default_output_dir: "~/images"
aspect: "16:9"
resolution: "2K"
quality: "high"
```

### Valid Keys

Defined in `internal/config/config.go` `ValidKeys()`: `api_key`, `model`,
`default_output_dir`, `aspect`, `resolution`, `quality`.

- `aspect` / `resolution` are imageConfig defaults (see
  [image-generation.md](image-generation.md) for valid values).
- `quality` is a model alias (`fast` → `gemini-3.1-flash-image`, `high` →
  `gemini-3-pro-image`).

### Auth Resolution Order

1. `GEMINI_API_KEY` environment variable
2. `api_key` from config file
3. Empty string (triggers ExitAuth error in commands)

### Model Resolution Order

Precedence (highest first):

1. `--model / -m` flag
2. `--quality` flag (`fast`/`high` alias → model id)
3. `model` from config file
4. `quality` from config file (intra-config tiebreak: config `model` beats config `quality`)
5. Built-in `DefaultModel` constant in `internal/gemini/client.go`: `gemini-3.1-flash-image`

Explicit flags are detected with cobra `Changed()` (not empty-string sentinels). The prior
default `gemini-2.0-flash-exp-image-generation` was retired upstream (2025-11-14); the
built-in default is now the current GA model so a fresh install with no config works.
