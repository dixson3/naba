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
model: "gemini-2.0-flash-exp"
default_output_dir: "~/images"
```

### Valid Keys

Defined in `internal/config/config.go` `ValidKeys()`: `api_key`, `model`, `default_output_dir`.

### Auth Resolution Order

1. `GEMINI_API_KEY` environment variable
2. `api_key` from config file
3. Empty string (triggers ExitAuth error in commands)

### Model Resolution Order

1. `--model / -m` flag
2. `model` from config file
3. Empty string (uses `defaultModel` constant in client.go: `gemini-2.0-flash-exp-image-generation`)
