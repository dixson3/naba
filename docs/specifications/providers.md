# naba — Providers Specification

Clause IDs (`SPEC-<AREA>-NNN`) are stable and are never renumbered; append only.

## §5 Provider layer (SPEC-PROVIDER)

- **SPEC-PROVIDER-001** [NEW] naba supports multiple providers: **gemini** (current),
  **openrouter** (new), and **AWS bedrock** (Epic 3). Every image path (`generate`, `edit`,
  `restore`, and the composite commands) routes through the selected provider. The provider set
  is the registry (SPEC-PROVIDER-009); the count is not fixed.
- **SPEC-PROVIDER-002** [PINNED] **Gemini** provider (port of the Go client): base URL
  `https://generativelanguage.googleapis.com/v1beta` (override via `GEMINI_BASE_URL`);
  endpoint `{base}/models/{model}:generateContent`, POST, headers `Content-Type:
  application/json` + `x-goog-api-key: <key>`; `generationConfig.responseModalities =
  ["TEXT","IMAGE"]` always; `imageConfig` under `generationConfig.imageConfig` with
  `aspectRatio`/`imageSize` (omitempty). Default model `gemini-3.1-flash-image`.
- **SPEC-PROVIDER-003** [PINNED] Gemini model constants: `DefaultModel = FlashModel =
  "gemini-3.1-flash-image"`, `ProModel = "gemini-3-pro-image"`.
- **SPEC-PROVIDER-004** [NEW] **OpenRouter** provider: bespoke client against the dedicated
  **`POST /api/v1/images`** Unified Image API. Bearer auth (`Authorization: Bearer <key>`).
  Base URL `https://openrouter.ai/api/v1` with a **`OPENROUTER_BASE_URL`** override
  (mirroring `GEMINI_BASE_URL`, for mockable tests). Request fields map from naba's
  imageConfig: `aspect_ratio` ← aspect, `resolution` ← imageSize (`512`/`1K`/`2K`/`4K`),
  native `quality` ← quality, and `input_references[]` for edit/restore image input.
  Response images are base64 in `data[].b64_json`. **CONFIRMED by the Issue 2.6 live-key
  smoke (2026-07-12):** the success envelope is `{ "created", "data": [{ "b64_json",
  "media_type" }], "usage": {...} }` (observed `media_type: "image/png"`); the error envelope
  is `{ "error": { "message", "code" } }`; `openrouter/auto` returns HTTP 404 `No endpoint
  found for model "openrouter/auto"` (SPEC-PROVIDER-006 validated — naba's early exit-2 guard
  means that call is never made). The `input_references[]` shape, the moderation-403 metadata,
  and the image-model discovery endpoint remain mock-validated (not exercised by the minimal
  smoke). Default image model slug `google/gemini-3.1-flash-image-preview`.
- **SPEC-PROVIDER-005** [PINNED/RESOLVED] **Per-provider `quality` semantics.** The trait
  carries the raw `--quality` value; each provider resolves it:
  - **Gemini**: quality → **model tier**. `fast` → `FlashModel`, `high` → `ProModel`. Any
    other value → `ExitUsage` `"invalid quality %q\n\nValid values: fast, high"`. `--quality`
    is overridden by an explicit `--model`.
  - **OpenRouter**: quality → the **native `quality` request parameter** on `/api/v1/images`;
    it does **NOT** swap the model. The model slug is selected independently (`--model` /
    config `model` / the default slug). Therefore `--provider openrouter --quality high`
    means: keep the resolved OpenRouter model slug, and pass `quality: high` to the API.
    (The `fast`/`high` vocabulary is preserved as the cross-provider surface; OpenRouter
    maps it onto its native quality parameter.)
- **SPEC-PROVIDER-006** [NEW] `openrouter/auto` **cannot generate images** (text-only
  router) and must **never** be selected for an image path — not as a default, not via
  autodetect. It is reserved for a possible future text path only. A request that would
  route image generation through `auto` is rejected.
- **SPEC-PROVIDER-007** [NEW] **Provider/model resolution precedence** (the selector
  factory): **CLI flags > config (`provider`/`model`) > env-key autodetect > built-in
  fallback**. Rules:
  - Env autodetect: only `GEMINI_API_KEY` present → **gemini** (+ Gemini default model);
    only `OPENROUTER_API_KEY` present → **openrouter** (+ default slug).
  - **Multiple keys + no config default → openrouter** with
    `google/gemini-3.1-flash-image-preview` (never `auto` for images).
  - **`--model` on the CLI requires `--provider`** — a model name alone is ambiguous across
    providers; `--model` without `--provider` is a usage error (`ExitUsage`, 2).
- **SPEC-PROVIDER-008** [NEW/INTENTIONAL — Concern 6] The multi-key default is an
  **intentional precedence outcome**: a user who already has `GEMINI_API_KEY` set and then
  adds `OPENROUTER_API_KEY` (with no `provider` in config) is **rerouted to OpenRouter**.
  This is documented, not a bug. The mitigation for a user who wants to stay on Gemini is to
  pin `provider: gemini` in config (config beats autodetect). SPEC and the `naba` skill/docs
  must call this out explicitly (Issue 5.2).
- **SPEC-PROVIDER-009** [NEW] **Provider registry** (Epic 2). The set of providers is a single
  registered list (`src/provider/registry.rs`), not a fixed pair of hardcoded match arms. Each
  registration declares the provider's `name`, conventional key env var, compiled-in default
  model (SPEC-CFGSCHEMA-006), whether `--quality` selects the model (SPEC-PROVIDER-005), whether
  it rejects the `auto` router (SPEC-PROVIDER-006), and a builder. The registry is the single
  source of truth the selector, config (`Valid keys:`), doctor, and the `provider`/`models`
  commands all read; adding a provider (e.g. Bedrock) is one new registration. The provider
  **count is no longer fixed at two**. **Explicit N-provider autodetect precedence:** the
  registry's declared order (oldest→newest) is the tie-break — among providers with resolvable
  creds the one appearing **latest** in the order wins (generalizing SPEC-PROVIDER-008: adding a
  newer provider's key reroutes to it); with no resolvable creds the fallback is the **first**
  registered provider. For the two-provider case this reproduces SPEC-PROVIDER-007 exactly
  (only-gemini→gemini, only-openrouter→openrouter, both→openrouter, neither→gemini).
- **SPEC-PROVIDER-010** [NEW] **`naba provider`** lists every registered provider with: whether
  it is the effective default (config `default_provider` > autodetect), whether its credentials
  resolve (SPEC-CFGSCHEMA-003 — **and, for bedrock, also a resolvable AWS profile /
  default-credential-chain (SigV4) credential per SPEC-PROVIDER-013**, so profile-only bedrock is
  reported `credentials: present`, not missing), and its effective default model. Human output +
  the universal `--json` envelope (SPEC-JSON-006, `data = {default_provider, providers:[{name,
  default, credentials, model}]}`). Read-only — no network call.
- **SPEC-PROVIDER-011** [NEW] **`naba models [--provider <name>]`** lists a provider's models via
  `Provider::list_models`. The target provider is the global `--provider` when set (validated
  against the registry; an unknown name is a usage error, exit 2) else the resolved default
  provider. It is a live API call: a provider with **no resolvable credential** raises the
  provider-named SPEC-ERR-001 "not set" auth error (exit 3). Credential validity is the same probe
  as SPEC-PROVIDER-010 — for bedrock a resolvable AWS profile / SigV4 credential (SPEC-PROVIDER-013)
  counts, so a profile-only bedrock does not hit the empty-key gate (the empty bearer key is fine;
  the provider signs with SigV4 at invoke time). Human output + the universal `--json` envelope
  (SPEC-JSON-006, `data = {provider, models:[<id>…]}`).
- **SPEC-PROVIDER-012** [NEW] **AWS Bedrock** provider (Epic 3): a **thin `reqwest`** client over
  the Bedrock Runtime **`InvokeModel`** REST call (operator decision at the bedrock-transport
  capability gate — chosen over the ~100-crate `aws-sdk-bedrockruntime`; `aws-sigv4` is pulled in
  only for the profile signing path, see SPEC-PROVIDER-013). Endpoint host pattern
  `https://bedrock-runtime.<region>.amazonaws.com`, override via **`BEDROCK_BASE_URL`** (mirrors
  `GEMINI_BASE_URL`/`OPENROUTER_BASE_URL`, for mockable tests); URL
  `{base}/model/{modelId}/invoke`, POST, `Content-Type`/`Accept: application/json`. Region default
  **`us-east-1`** (broadest image-model coverage), from `AWS_REGION` > `AWS_DEFAULT_REGION` > the
  default. Default model `amazon.nova-canvas-v1:0`. **Two request/response families** (raw
  per-model JSON body; both return base64 images decoded to bytes): the **Amazon** schema
  (`amazon.*` — Nova Canvas, Titan Image v1/v2: `{taskType, textToImageParams, imageGenerationConfig}`,
  edit/restore via `IMAGE_VARIATION`) and the **Stability** schema (`stability.*` — Stable Image
  Core/Ultra/SD 3.5: `{prompt, aspect_ratio, output_format}`, edit/restore via `mode:
  "image-to-image"`). Response images come from `{"images":[<b64>]}` (older Stability
  `{"artifacts":[{"base64":…}]}` also tolerated); no images → exit 5. `list_models` returns the
  curated image-model set (no network / no credentials). Registered through the Epic-2 registry
  (`registry.rs`) as one `ProviderSpec` entry; `--quality` is a native request param, not a model
  tier (SPEC-PROVIDER-005), and Bedrock has no `auto` router. **Edit/restore wire shapes and the
  SigV4 path are mock/unit-validated only — not exercised against real AWS.**
- **SPEC-PROVIDER-013** [NEW] **Bedrock auth — two modes.** (a) **api-key bearer**:
  `Authorization: Bearer <token>`, the token resolved through Epic-1's uniform api-key resolution
  (`providers.bedrock.api-key` inline > `providers.bedrock.api-key-envvar` > the conventional
  `AWS_BEARER_TOKEN_BEDROCK`). (b) **AWS profile / SigV4**: sign the request with `aws-sigv4` using
  credentials from the environment (`AWS_ACCESS_KEY_ID`/`AWS_SECRET_ACCESS_KEY`/`AWS_SESSION_TOKEN`)
  or a named `~/.aws/credentials` profile (`AWS_PROFILE`); region as in SPEC-PROVIDER-012. **Mode
  selection** (unit-testable, pure): prefer the bearer path when a non-empty bearer token is
  resolvable, else fall back to the profile/SigV4 path. The shared-credentials file is
  `AWS_SHARED_CREDENTIALS_FILE` (the standard AWS override) when set, else `~/.aws/credentials`.
  Full SSO-token / IMDS credential resolution is intentionally **out of scope** (the heavy
  `aws-config` path the thin-transport decision avoids). **Credential-validity probe**: the same
  profile/SigV4 resolution is exposed network-free to the command layer so `naba provider` /
  `naba models` (SPEC-PROVIDER-010/011) report bedrock credentials as present when only a profile /
  static-env / default-credential-chain credential (no bearer token) is configured — the probe
  shares the invoke-time credential loader and never changes auth-mode selection.
