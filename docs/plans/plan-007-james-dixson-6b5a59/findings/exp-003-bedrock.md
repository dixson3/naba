# Exp 003 — AWS Bedrock image provider (SDK, models, auth, deps)

## API

- `aws-sdk-bedrockruntime` 1.130.x + `aws-config` 1.x. Image gen = **synchronous `InvokeModel`**
  (Converse is text-only). No typed image API: send a raw model-specific JSON `body: Blob`
  (content-type/accept `application/json`, `model_id`), get raw JSON back with base64 images.

## Models (all return base64 images in the JSON body)

| Model | id | Request schema | Notes |
|:--|:--|:--|:--|
| Amazon Nova Canvas | `amazon.nova-canvas-v1:0` | `{taskType:"TEXT_IMAGE", textToImageParams:{text}, imageGenerationConfig:{width,height,quality,cfgScale,numberOfImages,seed}}` | shared Amazon schema; `numberOfImages` 1-5 |
| Amazon Titan Image | `amazon.titan-image-generator-v1` / `-v2:0` | same Amazon schema | sizes via width/height |
| Stability Stable Image Core | `stability.stable-image-core-v1:0` | `{prompt, aspect_ratio, seed, output_format, negative_prompt}` | sizes by aspect_ratio; 1 image/call |
| Stability Stable Image Ultra | `stability.stable-image-ultra-v1:1` | same Stability schema | |
| Stability SD 3.5 Large | `stability.sd3-5-large-v1:0` | same Stability schema | |

→ Two request/response families (Amazon vs Stability) → per-family serde. Response: `{"images":["<b64>"]}`.

## Auth

- **(a) Profile / SigV4:** `aws_config::defaults(BehaviorVersion::latest()).profile_name(p).region(r).load()`
  → `Client::new`. Honors `AWS_PROFILE` / `~/.aws/*` / SSO / IMDS automatically.
- **(b) API key (bearer, `AWS_BEARER_TOKEN_BEDROCK`):** plain `Authorization: Bearer <token>` header on the
  bedrock-runtime endpoint — NOT SigV4. **The Rust SDK does NOT auto-read `AWS_BEARER_TOKEN_BEDROCK`**
  (that's a boto3/JS feature); wiring it needs a custom interceptor OR a hand-rolled HTTPS POST.

## Dependency weight — the design tension

- Full SDK (`aws-sdk-bedrockruntime` + `aws-config`) ≈ **70-110 transitive crates**, multi-minute cold
  compile (Smithy/Tokio/Hyper/Rustls/aws-lc/aws-sigv4/SSO/IMDS).
- **Lighter path:** `InvokeModel` is one REST call —
  `POST https://bedrock-runtime.<region>.amazonaws.com/model/<model-id>/invoke`. Use `reqwest` +
  `aws-sigv4` (signing only) for the profile path, or `reqwest` + bearer header (NO signing crate) for the
  api-key path. **naba's gemini/openrouter providers are already thin `reqwest` HTTP clients**, so this
  matches the codebase idiom and avoids ~100 crates. Trade-off: you own credential-chain resolution
  (env/profile/SSO refresh), retry/backoff, endpoint construction, error typing.

## Region

- Regional models; default **`us-east-1`** (broadest image-model coverage), configurable per-invocation
  (a wrong-region model id 4xxs).

## DECISION for the operator (surface at review)

Operator initially chose **aws-sdk-rust, both modes**. But: (1) the SDK is heavy (~100 crates) and (2) the
**api-key bearer path needs hand-rolling regardless** (SDK won't auto-read the token). Since naba's other
providers are thin reqwest clients, **Recommendation: thin `reqwest` Bedrock client** — bearer for api-key
(no signing), `aws-sigv4` only for the profile path — OR the full SDK if first-class SigV4/SSO/role support
outweighs the footprint. This materially changes the dependency footprint → confirm at review.
