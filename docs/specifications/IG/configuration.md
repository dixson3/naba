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
  bedrock:
    model: amazon.nova-canvas-v1:0
    api-key-envvar: AWS_BEARER_TOKEN_BEDROCK  # api-key (bearer) path; AWS profile/SigV4 also works
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
  `openrouter`, `bedrock`
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
3. The provider's conventional default env var (`GEMINI_API_KEY`, `OPENROUTER_API_KEY`,
   `AWS_BEARER_TOKEN_BEDROCK`)
4. Empty string (triggers ExitAuth error in commands)

Note this is inline-first: an inline config `api-key` beats the conventional env var.

### AWS Bedrock provider (SPEC-PROVIDER-012/013)

Bedrock is a thin `reqwest` client over the Bedrock Runtime `InvokeModel` REST call — it does
**not** pull in the full `aws-sdk`. Two model families are supported (raw per-model JSON bodies,
both returning base64 images): the **Amazon** family (`amazon.*` — Nova Canvas, Titan Image v1/v2)
and the **Stability** family (`stability.*` — Stable Image Core / Ultra / SD 3.5). `naba models
--provider bedrock` lists the curated set.

Bedrock has **two auth modes**, chosen automatically (bearer preferred when a token is resolvable,
else the profile/SigV4 path):

1. **api-key bearer** — `Authorization: Bearer <token>`, resolved via the uniform api-key order
   above (`providers.bedrock.api-key` / `api-key-envvar` / `AWS_BEARER_TOKEN_BEDROCK`).
2. **AWS profile / SigV4** — the request is signed with `aws-sigv4` using credentials from the
   environment (`AWS_ACCESS_KEY_ID` / `AWS_SECRET_ACCESS_KEY` / `AWS_SESSION_TOKEN`) or a named
   `~/.aws/credentials` profile (`AWS_PROFILE`). SSO-token / IMDS resolution is out of scope.

Region defaults to **`us-east-1`** and is read from `AWS_REGION` > `AWS_DEFAULT_REGION` > the
default. The endpoint host is `https://bedrock-runtime.<region>.amazonaws.com` (override via
`BEDROCK_BASE_URL` for testing).

### Model Resolution Order

Each provider designates its own default model; an absent `providers.<name>.model` resolves to
that provider's compiled-in default, so no provider is ever model-less. Precedence
(highest first):

1. `--model / -m` flag (requires `--provider` on the CLI)
2. `--quality` flag (`fast`/`high` alias → model id, Gemini)
3. `providers.<default_provider>.model` from config
4. `quality` from config (tier alias)
5. The selected provider's built-in default model (e.g. gemini `gemini-3.1-flash-image`,
   openrouter `google/gemini-3.1-flash-image-preview`, bedrock `amazon.nova-canvas-v1:0`)
