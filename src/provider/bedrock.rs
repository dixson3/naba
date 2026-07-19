//! AWS Bedrock provider — a thin `reqwest` client over the Bedrock Runtime `InvokeModel`
//! REST call (Issue 3.2/3.3). Implements the [`Provider`] trait from Issue 2.2.
//!
//! # Why a thin reqwest client (operator decision at the capability gate)
//!
//! `InvokeModel` is a single synchronous REST call —
//! `POST https://bedrock-runtime.<region>.amazonaws.com/model/<modelId>/invoke` with a raw,
//! per-model JSON body and a raw JSON response carrying base64 images. naba's Gemini/OpenRouter
//! providers are already thin `reqwest` clients, so Bedrock matches that idiom and avoids the
//! ~100-crate `aws-sdk-bedrockruntime`. We own credential resolution, endpoint construction, and
//! error typing; we pull in only `aws-sigv4` (+ `aws-credential-types`) for the AWS-profile
//! signing path (see `findings/exp-003-bedrock.md`).
//!
//! # Endpoint / transport (SPEC-PROVIDER-012)
//!
//! * Host pattern `https://bedrock-runtime.<region>.amazonaws.com`, override via
//!   `BEDROCK_BASE_URL` (mirrors `GEMINI_BASE_URL`/`OPENROUTER_BASE_URL`, for mockable tests).
//! * URL `{base}/model/{modelId}/invoke`, POST, `Content-Type: application/json` +
//!   `Accept: application/json`.
//! * Region default `us-east-1` (broadest image-model coverage), from `AWS_REGION` >
//!   `AWS_DEFAULT_REGION` > the default. 120s HTTP timeout (parity with the other providers).
//!
//! # Two model families (SPEC-PROVIDER-012)
//!
//! Bedrock has no typed image API; each model family takes its own raw JSON body:
//!
//! * **Amazon** (`amazon.*` — Nova Canvas, Titan Image v1/v2): a shared schema
//!   `{taskType, textToImageParams:{text}, imageGenerationConfig:{numberOfImages,width,height,
//!   quality}}`; edit/restore uses `taskType: "IMAGE_VARIATION"` with `imageVariationParams`.
//! * **Stability** (`stability.*` — Stable Image Core / Ultra / SD 3.5): `{prompt, aspect_ratio,
//!   output_format}`; edit/restore adds `mode: "image-to-image"` + a base64 `image`.
//!
//! Both families return base64 images; the response is `{"images":["<b64>"]}` (older Stability
//! SDXL shape `{"artifacts":[{"base64":…}]}` is also tolerated). See [`parse_images`].
//!
//! # Two auth modes (SPEC-PROVIDER-013)
//!
//! * **api-key bearer**: `Authorization: Bearer <token>`, token from Epic-1's uniform api-key
//!   resolution (`providers.bedrock.api-key` / `api-key-envvar` / `AWS_BEARER_TOKEN_BEDROCK`).
//! * **AWS profile / SigV4**: sign the request with `aws-sigv4` using credentials from the
//!   environment (`AWS_ACCESS_KEY_ID`/`AWS_SECRET_ACCESS_KEY`/`AWS_SESSION_TOKEN`) or a named
//!   `~/.aws/credentials` profile (`AWS_PROFILE`).
//!
//! Selection (see [`select_auth_mode`]) prefers the bearer token when one is resolvable, else
//! falls back to the profile/SigV4 path. Full SSO-token / IMDS credential resolution is **not**
//! implemented (that is the heavy `aws-config` path we deliberately avoid) — env vars and static
//! profiles cover the common cases; see the module tests.

use async_trait::async_trait;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::provider::{GenerateRequest, GeneratedImage, ImageConfig, ModelInfo, Provider};

/// Default Bedrock region (SPEC-PROVIDER-012): broadest image-model coverage.
pub const DEFAULT_REGION: &str = "us-east-1";

/// The Bedrock service name for SigV4 signing.
const SIGV4_SERVICE: &str = "bedrock";

/// Default image model (SPEC-CFGSCHEMA-006): Amazon Nova Canvas, the current flagship Amazon
/// image model. Empty `model` falls back to this.
pub const DEFAULT_MODEL: &str = "amazon.nova-canvas-v1:0";

/// The curated set of image models naba advertises for Bedrock (`list_models`), from
/// `findings/exp-003-bedrock.md`. Order: Amazon family, then Stability family.
pub const CURATED_MODELS: &[&str] = &[
    "amazon.nova-canvas-v1:0",
    "amazon.titan-image-generator-v1",
    "amazon.titan-image-generator-v2:0",
    "stability.stable-image-core-v1:0",
    "stability.stable-image-ultra-v1:1",
    "stability.sd3-5-large-v1:0",
];

/// Fallback MIME for a decoded Bedrock image (all families return PNG).
const DEFAULT_IMAGE_MIME: &str = "image/png";

// ---------------------------------------------------------------------------------------------
// Model families
// ---------------------------------------------------------------------------------------------

/// The two request/response families Bedrock image models fall into (SPEC-PROVIDER-012).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    /// `amazon.*` — Nova Canvas, Titan Image v1/v2 (shared Amazon schema).
    Amazon,
    /// `stability.*` — Stable Image Core / Ultra / SD 3.5.
    Stability,
}

/// Classify a Bedrock `modelId` into its request/response [`Family`] by provider prefix. An
/// unrecognized prefix is a usage error (exit 2) — naba only knows the Amazon/Stability schemas.
pub fn family_for(model: &str) -> Result<Family, AppError> {
    if model.starts_with("amazon.") {
        Ok(Family::Amazon)
    } else if model.starts_with("stability.") {
        Ok(Family::Stability)
    } else {
        Err(AppError::usage(format!(
            "unsupported bedrock model {model:?}\n\nnaba supports the Amazon (amazon.*) and Stability (stability.*) image families"
        )))
    }
}

// ---------------------------------------------------------------------------------------------
// Auth-mode selection (SPEC-PROVIDER-013)
// ---------------------------------------------------------------------------------------------

/// Which auth path the provider will use for a request (SPEC-PROVIDER-013).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthChoice {
    /// `Authorization: Bearer <token>` — the resolved api-key path.
    Bearer,
    /// AWS SigV4 request signing — the profile / credential-chain path.
    SigV4,
}

/// Prefer the api-key bearer path when a non-empty bearer token is resolvable, else fall back to
/// the AWS profile / SigV4 path (SPEC-PROVIDER-013). Pure, so it is unit-testable without HTTP.
pub fn select_auth_mode(bearer_token: &str) -> AuthChoice {
    if bearer_token.trim().is_empty() {
        AuthChoice::SigV4
    } else {
        AuthChoice::Bearer
    }
}

// ---------------------------------------------------------------------------------------------
// Wire types (serde). Amazon vs Stability request bodies; a shared response envelope.
// ---------------------------------------------------------------------------------------------

/// Amazon-family request body (Nova Canvas / Titan): `TEXT_IMAGE` for text-to-image,
/// `IMAGE_VARIATION` for edit/restore.
#[derive(Debug, Serialize)]
struct AmazonRequest {
    #[serde(rename = "taskType")]
    task_type: &'static str,
    #[serde(rename = "textToImageParams", skip_serializing_if = "Option::is_none")]
    text_to_image_params: Option<AmazonTextParams>,
    #[serde(
        rename = "imageVariationParams",
        skip_serializing_if = "Option::is_none"
    )]
    image_variation_params: Option<AmazonVariationParams>,
    #[serde(rename = "imageGenerationConfig")]
    image_generation_config: AmazonGenConfig,
}

#[derive(Debug, Serialize)]
struct AmazonTextParams {
    text: String,
}

#[derive(Debug, Serialize)]
struct AmazonVariationParams {
    text: String,
    /// Base64-encoded source images for the variation.
    images: Vec<String>,
}

#[derive(Debug, Serialize)]
struct AmazonGenConfig {
    #[serde(rename = "numberOfImages")]
    number_of_images: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    quality: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    height: Option<u32>,
}

/// Stability-family request body (Stable Image Core / Ultra / SD 3.5).
#[derive(Debug, Serialize)]
struct StabilityRequest {
    prompt: String,
    #[serde(rename = "aspect_ratio", skip_serializing_if = "Option::is_none")]
    aspect_ratio: Option<String>,
    #[serde(rename = "output_format")]
    output_format: &'static str,
    /// `image-to-image` for edit/restore; omitted for text-to-image.
    #[serde(skip_serializing_if = "Option::is_none")]
    mode: Option<&'static str>,
    /// Base64 source image for `image-to-image`.
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    strength: Option<f32>,
}

/// Shared response envelope. Amazon and the current Stability image models return
/// `{"images":["<b64>"]}`; older Stability SDXL returns `{"artifacts":[{"base64":…}]}`.
#[derive(Debug, Default, Deserialize)]
struct BedrockResponse {
    #[serde(default)]
    images: Vec<String>,
    #[serde(default)]
    artifacts: Vec<StabilityArtifact>,
}

#[derive(Debug, Default, Deserialize)]
struct StabilityArtifact {
    #[serde(default)]
    base64: String,
}

/// Bedrock error envelope — `{"message": "..."}` (also `{"Message": "..."}` on some paths).
#[derive(Debug, Default, Deserialize)]
struct BedrockError {
    #[serde(default)]
    message: String,
    #[serde(rename = "Message", default)]
    message_alt: String,
}

// ---------------------------------------------------------------------------------------------
// Request-body construction (pure, testable)
// ---------------------------------------------------------------------------------------------

/// Map naba's `--quality` alias to an Amazon `imageGenerationConfig.quality`: `high` → `premium`,
/// `fast` → `standard`, anything else → omitted (let the model default).
fn amazon_quality(quality: Option<&str>) -> Option<&'static str> {
    match quality {
        Some("high") => Some("premium"),
        Some("fast") => Some("standard"),
        _ => None,
    }
}

/// Base pixel dimension for a naba image size (`512`/`1K`/`2K`/`4K`).
fn size_to_px(size: &str) -> Option<u32> {
    match size {
        "512" => Some(512),
        "1K" => Some(1024),
        "2K" => Some(2048),
        "4K" => Some(4096),
        _ => None,
    }
}

/// Derive Amazon `width`/`height` from naba's [`ImageConfig`] (SPEC-IMG-003). Returns `None` when
/// no imageConfig is present (let Bedrock default). The larger side is the size base (default 1024
/// when only an aspect is given); the other side is scaled to the aspect ratio and rounded down to
/// a multiple of 64 (Bedrock requires 64-aligned dimensions), floored at 320.
fn amazon_dimensions(cfg: &Option<ImageConfig>) -> Option<(u32, u32)> {
    let cfg = cfg.as_ref()?;
    let base = cfg.size.as_deref().and_then(size_to_px).unwrap_or(1024);
    let (aw, ah) = cfg
        .aspect
        .as_deref()
        .and_then(parse_aspect)
        .unwrap_or((1, 1));
    let (w, h) = if aw >= ah {
        (base, base * ah / aw)
    } else {
        (base * aw / ah, base)
    };
    Some((align64(w), align64(h)))
}

/// Parse an `W:H` aspect ratio into its integer parts.
fn parse_aspect(aspect: &str) -> Option<(u32, u32)> {
    let (w, h) = aspect.split_once(':')?;
    let w: u32 = w.trim().parse().ok()?;
    let h: u32 = h.trim().parse().ok()?;
    if w == 0 || h == 0 {
        None
    } else {
        Some((w, h))
    }
}

/// Round `px` down to a multiple of 64, floored at 320.
fn align64(px: u32) -> u32 {
    let aligned = (px / 64) * 64;
    aligned.max(320)
}

/// Serialize the Amazon-family request body for `req` (pure, testable).
fn amazon_body(req: &GenerateRequest) -> Result<Vec<u8>, AppError> {
    let dims = amazon_dimensions(&req.image_config);
    let (width, height) = match dims {
        Some((w, h)) => (Some(w), Some(h)),
        None => (None, None),
    };
    let gen = AmazonGenConfig {
        number_of_images: 1,
        quality: amazon_quality(req.quality.as_deref()),
        width,
        height,
    };
    let body = if let Some(img) = req.input_image.as_ref() {
        AmazonRequest {
            task_type: "IMAGE_VARIATION",
            text_to_image_params: None,
            image_variation_params: Some(AmazonVariationParams {
                text: req.prompt.clone(),
                images: vec![BASE64.encode(&img.bytes)],
            }),
            image_generation_config: gen,
        }
    } else {
        AmazonRequest {
            task_type: "TEXT_IMAGE",
            text_to_image_params: Some(AmazonTextParams {
                text: req.prompt.clone(),
            }),
            image_variation_params: None,
            image_generation_config: gen,
        }
    };
    serde_json::to_vec(&body).map_err(|e| AppError::api(format!("encode request: {e}")))
}

/// Serialize the Stability-family request body for `req` (pure, testable). `aspect_ratio` is
/// omitted on the `image-to-image` path (Stability rejects it together with a source image).
fn stability_body(req: &GenerateRequest) -> Result<Vec<u8>, AppError> {
    let aspect = req.image_config.as_ref().and_then(|c| c.aspect.clone());
    let body = if let Some(img) = req.input_image.as_ref() {
        StabilityRequest {
            prompt: req.prompt.clone(),
            aspect_ratio: None,
            output_format: "png",
            mode: Some("image-to-image"),
            image: Some(BASE64.encode(&img.bytes)),
            strength: Some(0.35),
        }
    } else {
        StabilityRequest {
            prompt: req.prompt.clone(),
            aspect_ratio: aspect,
            output_format: "png",
            mode: None,
            image: None,
            strength: None,
        }
    };
    serde_json::to_vec(&body).map_err(|e| AppError::api(format!("encode request: {e}")))
}

/// Build the raw `InvokeModel` request body for `model`'s family (SPEC-PROVIDER-012).
fn build_body(model: &str, req: &GenerateRequest) -> Result<Vec<u8>, AppError> {
    match family_for(model)? {
        Family::Amazon => amazon_body(req),
        Family::Stability => stability_body(req),
    }
}

// ---------------------------------------------------------------------------------------------
// Response parsing (pure, testable)
// ---------------------------------------------------------------------------------------------

/// Decode the base64 images from a Bedrock response body (SPEC-PROVIDER-012). Handles the
/// `{"images":[…]}` shape (Amazon + current Stability) and the `{"artifacts":[{"base64":…}]}`
/// shape (older Stability SDXL). No images → exit 5 `no images in response` (SPEC-ERR-014, parity
/// with the other providers).
fn parse_images(body: &[u8]) -> Result<Vec<GeneratedImage>, AppError> {
    let parsed: BedrockResponse =
        serde_json::from_slice(body).map_err(|e| AppError::api(format!("parse response: {e}")))?;
    let mut b64s: Vec<String> = parsed.images;
    b64s.extend(
        parsed
            .artifacts
            .into_iter()
            .map(|a| a.base64)
            .filter(|s| !s.is_empty()),
    );

    let mut images = Vec::new();
    for b64 in b64s {
        if b64.is_empty() {
            continue;
        }
        let bytes = BASE64
            .decode(b64.as_bytes())
            .map_err(|e| AppError::api(format!("decode image data: {e}")))?;
        images.push(GeneratedImage {
            bytes,
            mime: DEFAULT_IMAGE_MIME.to_string(),
        });
    }
    if images.is_empty() {
        return Err(AppError::api("no images in response"));
    }
    Ok(images)
}

/// Map a non-2xx status + body to an [`AppError`] with the right exit code (parity with the
/// Gemini/OpenRouter mapping): 401/403 → auth (3); 429 → rate-limit (4); ≥500 → server (5);
/// other non-2xx → API (5).
fn parse_api_error(status: u16, body: &[u8]) -> AppError {
    let parsed: BedrockError = serde_json::from_slice(body).unwrap_or_default();
    let mut msg = if !parsed.message.is_empty() {
        parsed.message
    } else {
        parsed.message_alt
    };
    if msg.is_empty() {
        msg = format!("API error (HTTP {status})");
    }

    if status == 401 || status == 403 {
        AppError::auth(format!(
            "authentication failed: {msg}\n\nSet AWS_BEARER_TOKEN_BEDROCK or configure AWS credentials (AWS_PROFILE / ~/.aws/credentials)."
        ))
    } else if status == 429 {
        AppError::rate_limit(format!(
            "rate limit exceeded: {msg}\n\nWait a moment and try again."
        ))
    } else if status >= 500 {
        AppError::api(format!(
            "Bedrock server error: {msg}\n\nThis is a temporary issue. Try again shortly."
        ))
    } else {
        AppError::api(msg)
    }
}

// ---------------------------------------------------------------------------------------------
// AWS credentials (minimal resolver for the SigV4 path)
// ---------------------------------------------------------------------------------------------

/// Static AWS credentials used to sign a SigV4 request.
#[derive(Debug, Clone, PartialEq, Eq)]
struct AwsCreds {
    access_key_id: String,
    secret_access_key: String,
    session_token: Option<String>,
}

/// Resolve static AWS credentials for the SigV4 path (SPEC-PROVIDER-013). Precedence: the
/// standard `AWS_ACCESS_KEY_ID`/`AWS_SECRET_ACCESS_KEY`(/`AWS_SESSION_TOKEN`) environment
/// variables, then the named profile in `~/.aws/credentials` (`profile` else `default`). SSO-token
/// and IMDS resolution are intentionally out of scope (the heavy `aws-config` path). An
/// unresolvable credential is an auth error (exit 3).
fn load_aws_credentials(profile: Option<&str>) -> Result<AwsCreds, AppError> {
    resolve_aws_creds_from_env(profile).ok_or_else(|| {
        AppError::auth(
            "AWS credentials not found.\n\nSet AWS_BEARER_TOKEN_BEDROCK for the api-key path, or provide AWS credentials (AWS_ACCESS_KEY_ID/AWS_SECRET_ACCESS_KEY, or a profile in ~/.aws/credentials).",
        )
    })
}

/// Network-free **validity probe** (SPEC-PROVIDER-013): whether a resolvable AWS profile /
/// default-credential-chain credential exists for the SigV4 path — the SAME resolution
/// [`load_aws_credentials`] performs at invoke time (both call [`resolve_aws_creds_from_env`], so
/// the probe never diverges from the actual credential loader). Exposed so the command layer can
/// report bedrock credentials as *present* when only an AWS profile / static env credential (and
/// no api-key bearer token) is configured. This does NOT change auth-mode selection —
/// [`select_auth_mode`] still prefers the bearer path at invoke time.
pub fn aws_credentials_resolvable(profile: Option<&str>) -> bool {
    resolve_aws_creds_from_env(profile).is_some()
}

/// Gather the process-environment credential sources (static env vars, the resolved profile name,
/// and the shared-credentials INI) and resolve them through the pure [`resolve_aws_creds`] core.
/// The single reader shared by [`load_aws_credentials`] and [`aws_credentials_resolvable`].
fn resolve_aws_creds_from_env(profile: Option<&str>) -> Option<AwsCreds> {
    let profile = profile
        .map(str::to_string)
        .or_else(|| std::env::var("AWS_PROFILE").ok())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "default".to_string());
    let ini = read_credentials_ini();
    resolve_aws_creds(
        env_nonempty("AWS_ACCESS_KEY_ID").as_deref(),
        env_nonempty("AWS_SECRET_ACCESS_KEY").as_deref(),
        env_nonempty("AWS_SESSION_TOKEN").as_deref(),
        &profile,
        ini.as_deref(),
    )
}

/// Pure credential resolution over already-read inputs (SPEC-PROVIDER-013): the static env
/// `AWS_ACCESS_KEY_ID`/`AWS_SECRET_ACCESS_KEY`(/`AWS_SESSION_TOKEN`) win; else the named `profile`
/// in the shared-credentials INI (`ini`, when present). Pure — no env, no filesystem — so both
/// invoke-time resolution and the validity probe share ONE code path and it is unit-testable.
fn resolve_aws_creds(
    env_id: Option<&str>,
    env_secret: Option<&str>,
    env_token: Option<&str>,
    profile: &str,
    ini: Option<&str>,
) -> Option<AwsCreds> {
    if let (Some(id), Some(secret)) = (env_id, env_secret) {
        if !id.is_empty() && !secret.is_empty() {
            return Some(AwsCreds {
                access_key_id: id.to_string(),
                secret_access_key: secret.to_string(),
                session_token: env_token.filter(|s| !s.is_empty()).map(str::to_string),
            });
        }
    }
    ini.and_then(|data| parse_credentials_ini(data, profile))
}

/// The process-env value of `name`, or `None` when unset or empty.
fn env_nonempty(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|s| !s.is_empty())
}

/// Read the shared-credentials INI text: `AWS_SHARED_CREDENTIALS_FILE` (the standard AWS override)
/// when set, else `$HOME/.aws/credentials`. `None` when no file is configured or it is unreadable.
fn read_credentials_ini() -> Option<String> {
    if let Some(path) = env_nonempty("AWS_SHARED_CREDENTIALS_FILE") {
        return std::fs::read_to_string(path).ok();
    }
    let home = std::env::var_os("HOME")?;
    let path = std::path::Path::new(&home).join(".aws").join("credentials");
    std::fs::read_to_string(path).ok()
}

/// Extract `profile`'s `aws_access_key_id`/`aws_secret_access_key`(/`aws_session_token`) from an
/// AWS-style INI document. Returns `None` when the section or a required key is missing.
fn parse_credentials_ini(data: &str, profile: &str) -> Option<AwsCreds> {
    let mut in_section = false;
    let mut id = None;
    let mut secret = None;
    let mut token = None;
    for raw in data.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if let Some(name) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            // AWS config uses `[profile foo]`; credentials uses `[foo]`. Accept both.
            let name = name.trim().strip_prefix("profile ").unwrap_or(name.trim());
            in_section = name == profile;
            continue;
        }
        if !in_section {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            let (k, v) = (k.trim(), v.trim().to_string());
            match k {
                "aws_access_key_id" => id = Some(v),
                "aws_secret_access_key" => secret = Some(v),
                "aws_session_token" => token = Some(v),
                _ => {}
            }
        }
    }
    match (id, secret) {
        (Some(access_key_id), Some(secret_access_key))
            if !access_key_id.is_empty() && !secret_access_key.is_empty() =>
        {
            Some(AwsCreds {
                access_key_id,
                secret_access_key,
                session_token: token.filter(|s| !s.is_empty()),
            })
        }
        _ => None,
    }
}

/// Compute the SigV4 signing headers to add to the request (host is signed; reqwest sends the
/// matching `Host`). Returns `(name, value)` pairs, including `Authorization`, `x-amz-date`, and
/// `x-amz-security-token` when a session token is present.
fn sigv4_headers(
    url: &str,
    host: &str,
    body: &[u8],
    creds: &AwsCreds,
    region: &str,
) -> Result<Vec<(String, String)>, AppError> {
    use aws_credential_types::Credentials;
    use aws_sigv4::http_request::{
        sign, SignableBody, SignableRequest, SigningParams, SigningSettings,
    };
    use aws_sigv4::sign::v4;
    use std::time::SystemTime;

    let identity = Credentials::new(
        creds.access_key_id.clone(),
        creds.secret_access_key.clone(),
        creds.session_token.clone(),
        None,
        "naba-bedrock",
    )
    .into();
    let settings = SigningSettings::default();
    let params: SigningParams = v4::SigningParams::builder()
        .identity(&identity)
        .region(region)
        .name(SIGV4_SERVICE)
        .time(SystemTime::now())
        .settings(settings)
        .build()
        .map_err(|e| AppError::api(format!("sigv4 params: {e}")))?
        .into();

    let signable = SignableRequest::new(
        "POST",
        url,
        std::iter::once(("host", host)),
        SignableBody::Bytes(body),
    )
    .map_err(|e| AppError::api(format!("sigv4 signable: {e}")))?;

    let (instructions, _sig) = sign(signable, &params)
        .map_err(|e| AppError::api(format!("sigv4 sign: {e}")))?
        .into_parts();
    Ok(instructions
        .headers()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect())
}

// ---------------------------------------------------------------------------------------------
// Provider
// ---------------------------------------------------------------------------------------------

/// Async AWS Bedrock provider (thin `reqwest` `InvokeModel` client).
pub struct BedrockProvider {
    /// The resolved api-key bearer token (empty → use the SigV4/profile path).
    api_key: String,
    model: String,
    region: String,
    base_url: String,
    /// Optional explicit AWS profile for the SigV4 path.
    profile: Option<String>,
    http: reqwest::Client,
}

impl BedrockProvider {
    /// Construct a provider. Empty `model` → [`DEFAULT_MODEL`]. Region from `AWS_REGION` >
    /// `AWS_DEFAULT_REGION` > [`DEFAULT_REGION`]; base URL from `BEDROCK_BASE_URL` else the
    /// regional host; profile from `AWS_PROFILE`. 120s HTTP timeout.
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        let region = std::env::var("AWS_REGION")
            .ok()
            .filter(|s| !s.is_empty())
            .or_else(|| {
                std::env::var("AWS_DEFAULT_REGION")
                    .ok()
                    .filter(|s| !s.is_empty())
            })
            .unwrap_or_else(|| DEFAULT_REGION.to_string());
        let base_url = std::env::var("BEDROCK_BASE_URL")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| format!("https://bedrock-runtime.{region}.amazonaws.com"));
        let profile = std::env::var("AWS_PROFILE").ok().filter(|s| !s.is_empty());
        Self::build(api_key, model, region, base_url, profile)
    }

    /// Construct with an explicit base URL + region, bypassing env lookups. Used by mock-server
    /// integration tests (avoids process-global env races).
    pub fn with_base_url(
        api_key: impl Into<String>,
        model: impl Into<String>,
        base_url: impl Into<String>,
        region: impl Into<String>,
    ) -> Self {
        Self::build(api_key, model, region.into(), base_url.into(), None)
    }

    fn build(
        api_key: impl Into<String>,
        model: impl Into<String>,
        region: String,
        base_url: String,
        profile: Option<String>,
    ) -> Self {
        let model = model.into();
        let model = if model.is_empty() {
            DEFAULT_MODEL.to_string()
        } else {
            model
        };
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .expect("build reqwest client");
        Self {
            api_key: api_key.into(),
            model,
            region,
            base_url,
            profile,
            http,
        }
    }

    /// Resolve the effective model: an explicit `req.model` wins, else the provider's configured
    /// model. `quality` does NOT swap the model for Bedrock (SPEC-PROVIDER-005) — it maps to a
    /// native param inside the family body builders.
    fn resolve_model(&self, req: &GenerateRequest) -> String {
        if req.model.is_empty() {
            self.model.clone()
        } else {
            req.model.clone()
        }
    }

    /// The `{base}/model/{modelId}/invoke` URL for `model`.
    fn invoke_url(&self, model: &str) -> String {
        format!("{}/model/{}/invoke", self.base_url, model)
    }

    /// The host component of the base URL (for SigV4 host signing).
    fn host(&self) -> String {
        self.base_url
            .split("://")
            .nth(1)
            .unwrap_or(&self.base_url)
            .split('/')
            .next()
            .unwrap_or("")
            .to_string()
    }

    /// Compute the auth headers to attach to an `InvokeModel` request, per the selected mode.
    fn auth_headers(&self, url: &str, body: &[u8]) -> Result<Vec<(String, String)>, AppError> {
        match select_auth_mode(&self.api_key) {
            AuthChoice::Bearer => Ok(vec![(
                "authorization".to_string(),
                format!("Bearer {}", self.api_key),
            )]),
            AuthChoice::SigV4 => {
                let creds = load_aws_credentials(self.profile.as_deref())?;
                sigv4_headers(url, &self.host(), body, &creds, &self.region)
            }
        }
    }
}

#[async_trait]
impl Provider for BedrockProvider {
    fn name(&self) -> &str {
        "bedrock"
    }

    async fn generate(&self, req: &GenerateRequest) -> Result<Vec<GeneratedImage>, AppError> {
        let model = self.resolve_model(req);
        let body = build_body(&model, req)?;
        let url = self.invoke_url(&model);

        let mut builder = self
            .http
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json");
        for (name, value) in self.auth_headers(&url, &body)? {
            builder = builder.header(name, value);
        }

        let resp = builder
            .body(body)
            .send()
            .await
            .map_err(|e| AppError::api(format!("api request failed: {e}")))?;

        let status = resp.status();
        let bytes = resp
            .bytes()
            .await
            .map_err(|e| AppError::api(format!("read response: {e}")))?;

        if !status.is_success() {
            return Err(parse_api_error(status.as_u16(), &bytes));
        }
        parse_images(&bytes)
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>, AppError> {
        // The curated image-model set (SPEC-PROVIDER-012). Bedrock's ListFoundationModels returns
        // every text/embedding model too, so naba advertises the vetted image models rather than a
        // live, unfiltered list — and this stays a zero-network, credential-free call.
        Ok(CURATED_MODELS
            .iter()
            .map(|id| ModelInfo {
                id: (*id).to_string(),
            })
            .collect())
    }
}

/// Exact-match membership check `naba doctor` uses (Bedrock analogue of the other providers'
/// `model_reachable`). Bedrock modelIds are fully qualified, so this is an exact match.
pub fn model_reachable(model_id: &str, available: &[ModelInfo]) -> bool {
    available.iter().any(|m| m.id == model_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::exit;
    use crate::provider::{GenerateRequest, ImageConfig, InputImage};

    // ---- family classification ----

    #[test]
    fn family_for_classifies_by_prefix() {
        assert_eq!(
            family_for("amazon.nova-canvas-v1:0").unwrap(),
            Family::Amazon
        );
        assert_eq!(
            family_for("amazon.titan-image-generator-v2:0").unwrap(),
            Family::Amazon
        );
        assert_eq!(
            family_for("stability.stable-image-core-v1:0").unwrap(),
            Family::Stability
        );
        assert_eq!(
            family_for("stability.sd3-5-large-v1:0").unwrap(),
            Family::Stability
        );
        let err = family_for("meta.llama3").unwrap_err();
        assert_eq!(err.code, exit::USAGE);
        assert!(err.message.contains("unsupported bedrock model"));
    }

    // ---- auth-mode selection (SPEC-PROVIDER-013) ----

    #[test]
    fn select_auth_mode_prefers_bearer_when_token_present() {
        assert_eq!(select_auth_mode("tok"), AuthChoice::Bearer);
        assert_eq!(select_auth_mode(""), AuthChoice::SigV4);
        assert_eq!(select_auth_mode("   "), AuthChoice::SigV4);
    }

    #[test]
    fn provider_auth_headers_bearer_when_key_present() {
        let p =
            BedrockProvider::with_base_url("secret-token", DEFAULT_MODEL, "http://x", "us-east-1");
        let headers = p.auth_headers("http://x/model/m/invoke", b"{}").unwrap();
        assert_eq!(
            headers,
            vec![(
                "authorization".to_string(),
                "Bearer secret-token".to_string()
            )]
        );
    }

    // ---- Amazon request body ----

    #[test]
    fn amazon_body_text_to_image_shape() {
        let req = GenerateRequest {
            prompt: "a corgi".to_string(),
            model: "amazon.nova-canvas-v1:0".to_string(),
            image_config: ImageConfig::new("16:9", "2K").unwrap(),
            quality: Some("high".to_string()),
            ..Default::default()
        };
        let body = amazon_body(&req).unwrap();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["taskType"], "TEXT_IMAGE");
        assert_eq!(v["textToImageParams"]["text"], "a corgi");
        assert_eq!(v["imageGenerationConfig"]["numberOfImages"], 1);
        assert_eq!(v["imageGenerationConfig"]["quality"], "premium");
        // 2K base, 16:9 → 2048 x 1152 (both 64-aligned).
        assert_eq!(v["imageGenerationConfig"]["width"], 2048);
        assert_eq!(v["imageGenerationConfig"]["height"], 1152);
        // No variation params on the text path.
        assert!(v.get("imageVariationParams").is_none());
    }

    #[test]
    fn amazon_body_omits_dimensions_and_quality_when_absent() {
        let req = GenerateRequest {
            prompt: "hi".to_string(),
            model: "amazon.titan-image-generator-v1".to_string(),
            ..Default::default()
        };
        let body = amazon_body(&req).unwrap();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["taskType"], "TEXT_IMAGE");
        assert_eq!(v["imageGenerationConfig"]["numberOfImages"], 1);
        assert!(v["imageGenerationConfig"].get("width").is_none());
        assert!(v["imageGenerationConfig"].get("height").is_none());
        assert!(v["imageGenerationConfig"].get("quality").is_none());
    }

    #[test]
    fn amazon_body_edit_uses_image_variation() {
        let req = GenerateRequest {
            prompt: "make it blue".to_string(),
            model: "amazon.nova-canvas-v1:0".to_string(),
            input_image: Some(InputImage {
                bytes: vec![1, 2, 3],
                mime: "image/png".to_string(),
            }),
            ..Default::default()
        };
        let body = amazon_body(&req).unwrap();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["taskType"], "IMAGE_VARIATION");
        assert_eq!(v["imageVariationParams"]["text"], "make it blue");
        assert_eq!(
            v["imageVariationParams"]["images"][0],
            BASE64.encode([1u8, 2, 3])
        );
        assert!(v.get("textToImageParams").is_none());
    }

    // ---- Stability request body ----

    #[test]
    fn stability_body_text_to_image_shape() {
        let req = GenerateRequest {
            prompt: "a mountain".to_string(),
            model: "stability.stable-image-core-v1:0".to_string(),
            image_config: ImageConfig::new("16:9", "").unwrap(),
            ..Default::default()
        };
        let body = stability_body(&req).unwrap();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["prompt"], "a mountain");
        assert_eq!(v["aspect_ratio"], "16:9");
        assert_eq!(v["output_format"], "png");
        assert!(v.get("mode").is_none());
        assert!(v.get("image").is_none());
    }

    #[test]
    fn stability_body_edit_uses_image_to_image_without_aspect() {
        let req = GenerateRequest {
            prompt: "restyle".to_string(),
            model: "stability.sd3-5-large-v1:0".to_string(),
            image_config: ImageConfig::new("16:9", "").unwrap(),
            input_image: Some(InputImage {
                bytes: vec![9, 9, 9, 9],
                mime: "image/jpeg".to_string(),
            }),
            ..Default::default()
        };
        let body = stability_body(&req).unwrap();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["mode"], "image-to-image");
        assert_eq!(v["image"], BASE64.encode([9u8, 9, 9, 9]));
        // aspect_ratio must be omitted alongside a source image.
        assert!(v.get("aspect_ratio").is_none());
    }

    // ---- dimension mapping ----

    #[test]
    fn amazon_dimensions_mapping() {
        assert_eq!(amazon_dimensions(&None), None);
        // size only, square default aspect.
        let c = ImageConfig::new("", "1K").unwrap();
        assert_eq!(amazon_dimensions(&c), Some((1024, 1024)));
        // aspect only, default 1024 base.
        let c = ImageConfig::new("2:3", "").unwrap();
        assert_eq!(amazon_dimensions(&c), Some((640, 1024)));
        // portrait 9:16 at 2K.
        let c = ImageConfig::new("9:16", "2K").unwrap();
        assert_eq!(amazon_dimensions(&c), Some((1152, 2048)));
    }

    // ---- response parsing ----

    #[test]
    fn parse_images_reads_images_array() {
        let b64 = BASE64.encode([0xDE, 0xAD, 0xBE, 0xEF]);
        let body = format!(r#"{{"images":["{b64}"]}}"#);
        let imgs = parse_images(body.as_bytes()).unwrap();
        assert_eq!(imgs.len(), 1);
        assert_eq!(imgs[0].bytes, vec![0xDE, 0xAD, 0xBE, 0xEF]);
        assert_eq!(imgs[0].mime, "image/png");
    }

    #[test]
    fn parse_images_reads_stability_artifacts() {
        let b64 = BASE64.encode([1u8, 2]);
        let body = format!(r#"{{"artifacts":[{{"base64":"{b64}"}}]}}"#);
        let imgs = parse_images(body.as_bytes()).unwrap();
        assert_eq!(imgs.len(), 1);
        assert_eq!(imgs[0].bytes, vec![1, 2]);
    }

    #[test]
    fn parse_images_no_images_is_exit_5() {
        let err = parse_images(br#"{"images":[]}"#).unwrap_err();
        assert_eq!(err.code, exit::API);
        assert_eq!(err.message, "no images in response");
    }

    // ---- error mapping ----

    #[test]
    fn parse_api_error_mappings() {
        let e = parse_api_error(403, br#"{"message":"not authorized"}"#);
        assert_eq!(e.code, exit::AUTH);
        assert!(e
            .message
            .starts_with("authentication failed: not authorized"));

        let e = parse_api_error(429, br#"{"message":"throttled"}"#);
        assert_eq!(e.code, exit::RATE_LIMIT);
        assert!(e.message.starts_with("rate limit exceeded: throttled"));

        let e = parse_api_error(500, br#"{"message":"boom"}"#);
        assert_eq!(e.code, exit::API);
        assert!(e.message.starts_with("Bedrock server error: boom"));

        // Alt-cased Message key.
        let e = parse_api_error(400, br#"{"Message":"bad input"}"#);
        assert_eq!(e.code, exit::API);
        assert_eq!(e.message, "bad input");

        // Empty body → synthesized.
        let e = parse_api_error(500, b"");
        assert_eq!(e.code, exit::API);
        assert!(e.message.contains("API error (HTTP 500)"));
    }

    // ---- credentials INI parsing ----

    #[test]
    fn parse_credentials_ini_named_and_default() {
        let ini = "[default]\naws_access_key_id = AKIA_DEFAULT\naws_secret_access_key = secret0\n\n[prod]\naws_access_key_id = AKIA_PROD\naws_secret_access_key = secret1\naws_session_token = tok1\n";
        let d = parse_credentials_ini(ini, "default").unwrap();
        assert_eq!(d.access_key_id, "AKIA_DEFAULT");
        assert_eq!(d.secret_access_key, "secret0");
        assert_eq!(d.session_token, None);

        let p = parse_credentials_ini(ini, "prod").unwrap();
        assert_eq!(p.access_key_id, "AKIA_PROD");
        assert_eq!(p.session_token.as_deref(), Some("tok1"));

        // `[profile name]` (config-style) header is also accepted.
        let cfg = "[profile staging]\naws_access_key_id = AKIA_STG\naws_secret_access_key = s\n";
        assert_eq!(
            parse_credentials_ini(cfg, "staging").unwrap().access_key_id,
            "AKIA_STG"
        );

        // Missing section / missing key → None.
        assert!(parse_credentials_ini(ini, "nope").is_none());
        assert!(parse_credentials_ini("[x]\naws_access_key_id = only-id\n", "x").is_none());
    }

    // ---- pure AWS credential resolution (SPEC-PROVIDER-013, validity-probe core) ----

    #[test]
    fn resolve_aws_creds_prefers_static_env_over_profile() {
        // Static env keys win even when a profile INI is also present.
        let ini = "[default]\naws_access_key_id = FROM_INI\naws_secret_access_key = ini_secret\n";
        let c = resolve_aws_creds(
            Some("AKIA_ENV"),
            Some("env_secret"),
            Some("env_tok"),
            "default",
            Some(ini),
        )
        .unwrap();
        assert_eq!(c.access_key_id, "AKIA_ENV");
        assert_eq!(c.secret_access_key, "env_secret");
        assert_eq!(c.session_token.as_deref(), Some("env_tok"));
    }

    #[test]
    fn resolve_aws_creds_falls_back_to_named_profile() {
        // No static env keys → the named profile in the INI (respecting AWS_PROFILE / default).
        let ini = "[default]\naws_access_key_id = AKIA_DEF\naws_secret_access_key = def_secret\n[prod]\naws_access_key_id = AKIA_PROD\naws_secret_access_key = prod_secret\n";
        let d = resolve_aws_creds(None, None, None, "default", Some(ini)).unwrap();
        assert_eq!(d.access_key_id, "AKIA_DEF");
        let p = resolve_aws_creds(None, None, None, "prod", Some(ini)).unwrap();
        assert_eq!(p.access_key_id, "AKIA_PROD");
    }

    #[test]
    fn resolve_aws_creds_none_when_nothing_resolves() {
        // No env keys, no INI → unresolvable.
        assert!(resolve_aws_creds(None, None, None, "default", None).is_none());
        // A partial env credential (id but no secret) does not resolve, and the profile is absent.
        assert!(resolve_aws_creds(Some("AKIA"), None, None, "default", None).is_none());
        // INI present but the requested profile is missing.
        let ini = "[other]\naws_access_key_id = x\naws_secret_access_key = y\n";
        assert!(resolve_aws_creds(None, None, None, "default", Some(ini)).is_none());
    }

    // ---- list_models curated set ----

    #[tokio::test]
    async fn list_models_returns_curated_set() {
        let p = BedrockProvider::with_base_url("tok", DEFAULT_MODEL, "http://x", "us-east-1");
        let models = p.list_models().await.unwrap();
        let ids: Vec<String> = models.iter().map(|m| m.id.clone()).collect();
        assert_eq!(
            ids,
            vec![
                "amazon.nova-canvas-v1:0",
                "amazon.titan-image-generator-v1",
                "amazon.titan-image-generator-v2:0",
                "stability.stable-image-core-v1:0",
                "stability.stable-image-ultra-v1:1",
                "stability.sd3-5-large-v1:0",
            ]
        );
        assert!(model_reachable("stability.sd3-5-large-v1:0", &models));
        assert!(!model_reachable("openai/gpt-image-1", &models));
    }

    #[test]
    fn empty_model_defaults_to_nova_canvas() {
        let p = BedrockProvider::with_base_url("k", "", "http://x", "us-east-1");
        assert_eq!(p.model, DEFAULT_MODEL);
    }

    #[test]
    fn invoke_url_and_host() {
        let p = BedrockProvider::with_base_url(
            "k",
            DEFAULT_MODEL,
            "https://bedrock-runtime.us-west-2.amazonaws.com",
            "us-west-2",
        );
        assert_eq!(
            p.invoke_url("amazon.nova-canvas-v1:0"),
            "https://bedrock-runtime.us-west-2.amazonaws.com/model/amazon.nova-canvas-v1:0/invoke"
        );
        assert_eq!(p.host(), "bedrock-runtime.us-west-2.amazonaws.com");
    }

    // ----------------------------------------------------------------------------------------
    // Integration tests over a wiremock HTTP server (no real AWS). Bearer mode (no SigV4) so the
    // request is deterministic; assert the OUTGOING body + the decode/error mapping end-to-end.
    // ----------------------------------------------------------------------------------------
    mod http {
        use super::*;
        use serde_json::json;
        use wiremock::matchers::{body_json, header, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        #[tokio::test]
        async fn generate_amazon_bearer_request_and_decode() {
            let server = MockServer::start().await;
            let b64 = BASE64.encode([0xDE, 0xAD, 0xBE, 0xEF]);
            Mock::given(method("POST"))
                .and(path("/model/amazon.nova-canvas-v1:0/invoke"))
                .and(header("authorization", "Bearer secret-token"))
                .and(header("content-type", "application/json"))
                .and(body_json(json!({
                    "taskType": "TEXT_IMAGE",
                    "textToImageParams": {"text": "a corgi"},
                    "imageGenerationConfig": {"numberOfImages": 1}
                })))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({"images": [b64]})))
                .mount(&server)
                .await;

            let provider = BedrockProvider::with_base_url(
                "secret-token",
                "amazon.nova-canvas-v1:0",
                server.uri(),
                "us-east-1",
            );
            let req = GenerateRequest {
                prompt: "a corgi".to_string(),
                model: "amazon.nova-canvas-v1:0".to_string(),
                ..Default::default()
            };
            let images = provider.generate(&req).await.unwrap();
            assert_eq!(images.len(), 1);
            assert_eq!(images[0].bytes, vec![0xDE, 0xAD, 0xBE, 0xEF]);
            assert_eq!(images[0].mime, "image/png");
        }

        #[tokio::test]
        async fn generate_stability_request_and_decode() {
            let server = MockServer::start().await;
            let b64 = BASE64.encode([1u8, 2, 3]);
            Mock::given(method("POST"))
                .and(path("/model/stability.stable-image-core-v1:0/invoke"))
                .and(body_json(json!({
                    "prompt": "a mountain",
                    "aspect_ratio": "16:9",
                    "output_format": "png"
                })))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({"images": [b64]})))
                .mount(&server)
                .await;

            let provider = BedrockProvider::with_base_url(
                "tok",
                "stability.stable-image-core-v1:0",
                server.uri(),
                "us-east-1",
            );
            let req = GenerateRequest {
                prompt: "a mountain".to_string(),
                model: "stability.stable-image-core-v1:0".to_string(),
                image_config: ImageConfig::new("16:9", "").unwrap(),
                ..Default::default()
            };
            let images = provider.generate(&req).await.unwrap();
            assert_eq!(images.len(), 1);
            assert_eq!(images[0].bytes, vec![1, 2, 3]);
        }

        #[tokio::test]
        async fn error_403_maps_to_exit_3() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .respond_with(ResponseTemplate::new(403).set_body_json(json!({
                    "message": "not authorized to invoke"
                })))
                .mount(&server)
                .await;
            let provider = BedrockProvider::with_base_url(
                "tok",
                "amazon.nova-canvas-v1:0",
                server.uri(),
                "us-east-1",
            );
            let req = GenerateRequest {
                prompt: "x".to_string(),
                model: "amazon.nova-canvas-v1:0".to_string(),
                ..Default::default()
            };
            let err = provider.generate(&req).await.unwrap_err();
            assert_eq!(err.code, exit::AUTH);
            assert!(err
                .message
                .starts_with("authentication failed: not authorized to invoke"));
        }

        #[tokio::test]
        async fn error_429_maps_to_exit_4() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .respond_with(ResponseTemplate::new(429).set_body_json(json!({
                    "message": "Too many requests"
                })))
                .mount(&server)
                .await;
            let provider = BedrockProvider::with_base_url(
                "tok",
                "amazon.nova-canvas-v1:0",
                server.uri(),
                "us-east-1",
            );
            let req = GenerateRequest {
                prompt: "x".to_string(),
                model: "amazon.nova-canvas-v1:0".to_string(),
                ..Default::default()
            };
            let err = provider.generate(&req).await.unwrap_err();
            assert_eq!(err.code, exit::RATE_LIMIT);
        }

        #[tokio::test]
        async fn no_images_maps_to_exit_5() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({"images": []})))
                .mount(&server)
                .await;
            let provider = BedrockProvider::with_base_url(
                "tok",
                "amazon.nova-canvas-v1:0",
                server.uri(),
                "us-east-1",
            );
            let req = GenerateRequest {
                prompt: "x".to_string(),
                model: "amazon.nova-canvas-v1:0".to_string(),
                ..Default::default()
            };
            let err = provider.generate(&req).await.unwrap_err();
            assert_eq!(err.code, exit::API);
            assert_eq!(err.message, "no images in response");
        }
    }
}
