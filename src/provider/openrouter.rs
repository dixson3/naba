//! OpenRouter provider — a bespoke async Rust client against OpenRouter's dedicated
//! **Unified Image API** (`POST /api/v1/images`, shipped 2026-06-23). Implements the
//! [`Provider`] trait from Issue 2.2. Companion to the Gemini provider (Issue 2.3).
//!
//! Wire behavior is designed against OpenRouter's *documented* `/api/v1/images` surface
//! (INV-2, `findings/exp-002-openrouter-image-api.md`) and **confirmed by the Issue 2.6
//! live-key smoke (2026-07-12)**. The smoke ran `POST /api/v1/images` against a real key and
//! observed the exact success envelope
//! `{ "created", "data": [{ "b64_json", "media_type" }], "usage": {...} }` — matching the
//! design below — and confirmed `openrouter/auto` returns HTTP 404
//! (`No endpoint found for model "openrouter/auto"`), i.e. `auto` cannot back an image path
//! (SPEC-PROVIDER-006). The response-envelope `// CONFIRM 2.6` markers are now RESOLVED. Three
//! secondary markers were NOT exercised by the minimal smoke and remain mock-validated
//! (accepted risk, flagged inline): the `input_references[]` element shape (edit/restore),
//! the moderation-403 metadata keys, and the image-model *discovery* endpoint.
//!
//! * **Endpoint / headers (SPEC-PROVIDER-004).** POST `{base}/images` with
//!   `Content-Type: application/json` + `Authorization: Bearer <key>`. Base URL defaults to
//!   `https://openrouter.ai/api/v1`, overridable via `OPENROUTER_BASE_URL` (mirrors
//!   `GEMINI_BASE_URL`). OpenRouter also commonly wants the optional app-identification
//!   headers `HTTP-Referer` / `X-Title`; we send sensible naba defaults (see
//!   [`APP_REFERER`] / [`APP_TITLE`]). 120s HTTP timeout (parity with Gemini).
//! * **Default model (SPEC-PROVIDER-004).** Empty model → [`DEFAULT_MODEL`] =
//!   `google/gemini-3.1-flash-image-preview` ("Nano Banana 2"). There is **no** image `auto`
//!   router — see the `auto` guard below.
//! * **Request shape.** `model` (slug) + `prompt`; `aspect_ratio` ← `image_config.aspect`,
//!   `resolution` ← `image_config.size` (`512`/`1K`/`2K`/`4K`, same vocabulary), and the
//!   native `quality` ← `req.quality` (the RAW value — SPEC-PROVIDER-005: OpenRouter treats
//!   quality as a first-class request param that does NOT swap the model). For edit/restore,
//!   `input_references[]` carries the input image as a base64 data URL.
//! * **`auto` guard (SPEC-PROVIDER-006).** A resolved model of `openrouter/auto` (or bare
//!   `auto`) is rejected with a usage error (exit 2): `auto` is a text-only chat router and
//!   must never back an image path — not as a default, not via autodetect.
//! * **Error / exit mapping (SPEC-ERR-010..017).** 401/403 auth → exit 3; a moderation /
//!   content-policy 403 → exit 5 (documented decision below); 429 → exit 4 honoring
//!   `Retry-After`; ≥500 → exit 5 with a provider-named string (SPEC-ERR-012 [DIVERGENCE]);
//!   other non-2xx → exit 5; no images → exit 5.
//!
//! ## Content-policy (moderation) exit decision — SPEC-ERR-017
//!
//! SPEC-ERR-017 leaves the moderation/content-policy 403 mapping as **exit 3 (auth-class) or
//! exit 5**, to be pinned by the live smoke. **naba maps a moderation 403 to exit 5** (API /
//! generation-refusal class), not exit 3. Rationale: a moderation block is a refusal to
//! produce *this content* — semantically the sibling of Gemini's `prompt blocked` (SPEC-ERR-013,
//! exit 5) — not a credential problem (exit 3 is reserved for "your key is bad / set your
//! key"). Presenting a content refusal as an auth failure would mislead the user into
//! re-checking their key. This is a deliberate pick; the `// CONFIRM 2.6` marker on
//! [`is_moderation_error`] flags the detection heuristic for live confirmation.

use async_trait::async_trait;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::provider::{GenerateRequest, GeneratedImage, ImageConfig, ModelInfo, Provider};

/// Default OpenRouter base URL (SPEC-PROVIDER-004); overridable via `OPENROUTER_BASE_URL`.
const DEFAULT_BASE_URL: &str = "https://openrouter.ai/api/v1";

/// Default image model slug (SPEC-PROVIDER-004). "Nano Banana 2". NOT `auto` (SPEC-PROVIDER-006).
pub const DEFAULT_MODEL: &str = "google/gemini-3.1-flash-image-preview";

/// Optional OpenRouter app-identification headers. Not required for auth, but OpenRouter
/// surfaces them in its dashboard and some rankings; we send stable naba defaults.
const APP_REFERER: &str = "https://github.com/dixson3/naba";
const APP_TITLE: &str = "naba";

/// Fallback MIME when the response omits a per-image media type.
const DEFAULT_IMAGE_MIME: &str = "image/png";

/// The `auto` router slugs that must never back an image path (SPEC-PROVIDER-006).
/// Public so the 2.5 selector factory can apply the same early guard (single source of truth).
pub fn is_auto_router(model: &str) -> bool {
    model == "openrouter/auto" || model == "auto"
}

// ---------------------------------------------------------------------------------------------
// Wire types (serde). Request field names are from the documented `/api/v1/images` surface;
// RESPONSE field names carry `// CONFIRM 2.6` markers where the docs could not be pinned exactly.
// ---------------------------------------------------------------------------------------------

/// Outgoing request body for `POST /api/v1/images`.
#[derive(Debug, Serialize)]
struct WireRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    #[serde(rename = "aspect_ratio", skip_serializing_if = "Option::is_none")]
    aspect_ratio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resolution: Option<String>,
    /// Native quality param (SPEC-PROVIDER-005): raw `--quality`, does NOT swap the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    quality: Option<String>,
    /// Edit/restore input images. Omitted entirely (not `[]`) when there is no input image.
    #[serde(rename = "input_references", skip_serializing_if = "Vec::is_empty")]
    input_references: Vec<WireInputReference>,
}

/// One entry in `input_references[]`. Documented shape: `input_references[].image_url.url`,
/// where `url` is an HTTP(S) URL or a base64 data URL. naba always sends a base64 data URL
/// (`data:<mime>;base64,<...>`) built from the local [`InputImage`]. The `type: "image_url"`
/// discriminator mirrors OpenAI-style content blocks.
// CONFIRM 2.6: exact `input_references[]` element shape — whether `type` is required and
// whether the key is `image_url` (vs `image`) — needs a live key. Kept in one struct so a
// change is local.
#[derive(Debug, Serialize)]
struct WireInputReference {
    #[serde(rename = "type")]
    kind: &'static str,
    image_url: WireImageUrl,
}

#[derive(Debug, Serialize)]
struct WireImageUrl {
    /// `data:<mime>;base64,<b64>` (or an HTTP(S) URL). naba emits a data URL.
    url: String,
}

/// Response envelope. OpenAI-Images-style: `data[]` of base64 images.
// CONFIRMED 2.6 (live smoke 2026-07-12): the envelope key IS `data` (top level also carries
// `created` and `usage`, which naba ignores).
#[derive(Debug, Default, Deserialize)]
struct WireResponse {
    #[serde(default)]
    data: Vec<WireImageData>,
}

/// One image in the response.
#[derive(Debug, Default, Deserialize)]
struct WireImageData {
    // CONFIRMED 2.6 (live smoke 2026-07-12): the base64 payload field IS `b64_json`.
    #[serde(default)]
    b64_json: String,
    // CONFIRMED 2.6 (live smoke 2026-07-12): the per-image MIME field IS `media_type`
    // (observed value `image/png`). Fallback to image/png retained if ever absent.
    #[serde(default)]
    media_type: String,
}

/// Models-listing envelope for `GET /api/v1/models`.
// CONFIRM 2.6: image-model *discovery* has richer endpoints
// (`GET /api/v1/models?output_modalities=image`, `GET /api/v1/images/models`, per-model
// `/endpoints`). This general `/models` list is the doctor-liveness seam; a follow-up (2.6)
// may switch to the image-filtered endpoint and read `output_modalities`.
#[derive(Debug, Default, Deserialize)]
struct WireModelList {
    #[serde(default)]
    data: Vec<WireModel>,
}

#[derive(Debug, Default, Deserialize)]
struct WireModel {
    #[serde(default)]
    id: String,
}

/// Error envelope. OpenRouter wraps errors as `{ "error": { "message", "code", "metadata" } }`.
#[derive(Debug, Default, Deserialize)]
struct WireErrorResponse {
    #[serde(default)]
    error: WireErrorBody,
}

#[derive(Debug, Default, Deserialize)]
struct WireErrorBody {
    #[serde(default)]
    message: String,
    #[serde(default)]
    metadata: WireErrorMetadata,
}

/// Moderation-error metadata (403 content-policy). Presence of `reasons`/`flagged_input`
/// distinguishes a moderation block from a plain permissions 403.
// CONFIRM 2.6: moderation metadata keys (`reasons`, `flagged_input`) per docs; confirm live.
#[derive(Debug, Default, Deserialize)]
struct WireErrorMetadata {
    #[serde(default)]
    reasons: Vec<String>,
    #[serde(default)]
    flagged_input: String,
}

// ---------------------------------------------------------------------------------------------
// Provider
// ---------------------------------------------------------------------------------------------

/// Async OpenRouter provider (bespoke `/api/v1/images` client).
pub struct OpenRouterProvider {
    api_key: String,
    model: String,
    base_url: String,
    http: reqwest::Client,
}

impl OpenRouterProvider {
    /// Construct a provider. An empty `model` falls back to [`DEFAULT_MODEL`]. The base URL is
    /// taken from `OPENROUTER_BASE_URL` when set, else the SPEC default. 120s HTTP timeout.
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        let base_url = std::env::var("OPENROUTER_BASE_URL")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());
        Self::with_base_url(api_key, model, base_url)
    }

    /// Construct with an explicit base URL, bypassing the `OPENROUTER_BASE_URL` env lookup.
    /// Used by mock-server integration tests (avoids process-global env races) and available
    /// to the 2.5 selector if it wants to inject a base URL directly. Empty `model` still
    /// defaults to [`DEFAULT_MODEL`].
    pub fn with_base_url(
        api_key: impl Into<String>,
        model: impl Into<String>,
        base_url: impl Into<String>,
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
            base_url: base_url.into(),
            http,
        }
    }

    /// Resolve the effective model for a request. Unlike Gemini, `quality` does NOT map to a
    /// model (SPEC-PROVIDER-005): an explicit `req.model` wins, otherwise the provider's
    /// configured model. The 2.5 selector normally sets `req.model` up front — this keeps the
    /// provider correct standalone.
    fn resolve_model(&self, req: &GenerateRequest) -> String {
        if !req.model.is_empty() {
            req.model.clone()
        } else {
            self.model.clone()
        }
    }
}

#[async_trait]
impl Provider for OpenRouterProvider {
    fn name(&self) -> &str {
        "openrouter"
    }

    async fn generate(&self, req: &GenerateRequest) -> Result<Vec<GeneratedImage>, AppError> {
        let model = self.resolve_model(req);

        // SPEC-PROVIDER-006: `auto` is a text-only router; reject BEFORE any HTTP call (exit 2).
        if is_auto_router(&model) {
            return Err(AppError::usage(format!(
                "model {model:?} cannot generate images: openrouter/auto is a text-only router\n\nSet an image model, e.g. --model {DEFAULT_MODEL}"
            )));
        }

        // Build input_references[] for the edit/restore path (base64 data URL).
        let mut input_references = Vec::new();
        if let Some(img) = req.input_image.as_ref() {
            let data_url = format!("data:{};base64,{}", img.mime, BASE64.encode(&img.bytes));
            input_references.push(WireInputReference {
                kind: "image_url",
                image_url: WireImageUrl { url: data_url },
            });
        }

        let (aspect_ratio, resolution) = match req.image_config.as_ref() {
            Some(ImageConfig { aspect, size }) => (aspect.clone(), size.clone()),
            None => (None, None),
        };

        let wire = WireRequest {
            model: &model,
            prompt: &req.prompt,
            aspect_ratio,
            resolution,
            quality: req.quality.clone().filter(|q| !q.is_empty()),
            input_references,
        };

        let url = format!("{}/images", self.base_url);
        let resp = self
            .http
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("HTTP-Referer", APP_REFERER)
            .header("X-Title", APP_TITLE)
            .json(&wire)
            .send()
            .await
            .map_err(|e| AppError::api(format!("api request failed: {e}")))?;

        let status = resp.status();
        let retry_after = resp
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        let body = resp
            .bytes()
            .await
            .map_err(|e| AppError::api(format!("read response: {e}")))?;

        if !status.is_success() {
            return Err(parse_api_error(
                status.as_u16(),
                &body,
                retry_after.as_deref(),
            ));
        }

        let parsed: WireResponse = serde_json::from_slice(&body)
            .map_err(|e| AppError::api(format!("parse response: {e}")))?;

        extract_images(parsed)
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>, AppError> {
        let url = format!("{}/models", self.base_url);
        let resp = self
            .http
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("HTTP-Referer", APP_REFERER)
            .header("X-Title", APP_TITLE)
            .send()
            .await
            .map_err(|e| AppError::api(format!("api request failed: {e}")))?;

        let status = resp.status();
        let retry_after = resp
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        let body = resp
            .bytes()
            .await
            .map_err(|e| AppError::api(format!("read response: {e}")))?;

        if !status.is_success() {
            return Err(parse_api_error(
                status.as_u16(),
                &body,
                retry_after.as_deref(),
            ));
        }

        let parsed: WireModelList = serde_json::from_slice(&body)
            .map_err(|e| AppError::api(format!("parse response: {e}")))?;

        Ok(parsed
            .data
            .into_iter()
            .map(|m| ModelInfo { id: m.id })
            .collect())
    }
}

/// Prefix-tolerant membership check `naba doctor` uses (OpenRouter analogue of Gemini's
/// `model_reachable`). OpenRouter slugs are already fully qualified (`author/slug`), so this is
/// an exact match; kept as a named seam so the doctor call site is provider-symmetric.
pub fn model_reachable(model_id: &str, available: &[ModelInfo]) -> bool {
    available.iter().any(|m| m.id == model_id)
}

/// Extract base64 images from the response envelope into decoded [`GeneratedImage`]s. No images
/// → exit 5 `no images in response` (SPEC-ERR-014, parity with Gemini).
fn extract_images(resp: WireResponse) -> Result<Vec<GeneratedImage>, AppError> {
    let mut images = Vec::new();
    for img in resp.data {
        if img.b64_json.is_empty() {
            continue;
        }
        let bytes = BASE64
            .decode(img.b64_json.as_bytes())
            .map_err(|e| AppError::api(format!("decode image data: {e}")))?;
        let mime = if img.media_type.is_empty() {
            DEFAULT_IMAGE_MIME.to_string()
        } else {
            img.media_type
        };
        images.push(GeneratedImage { bytes, mime });
    }
    if images.is_empty() {
        return Err(AppError::api("no images in response"));
    }
    Ok(images)
}

/// Whether a 403 body is a moderation / content-policy block (vs a plain permissions 403).
/// Heuristic: moderation metadata (`reasons` / `flagged_input`) present, or the message reads
/// like a content-policy refusal.
// CONFIRM 2.6: moderation detection heuristic — confirm the live 403 moderation body carries
// `error.metadata.reasons` / `flagged_input` (per docs) and/or a "moderation"/"content policy"
// message. If the live shape differs, this is the one place to adjust.
fn is_moderation_error(err: &WireErrorBody) -> bool {
    if !err.metadata.reasons.is_empty() || !err.metadata.flagged_input.is_empty() {
        return true;
    }
    let m = err.message.to_ascii_lowercase();
    m.contains("moderation") || m.contains("content policy") || m.contains("flagged")
}

/// Map a non-2xx HTTP status + body to an [`AppError`] with the right exit code
/// (SPEC-ERR-010..017). 401 and non-moderation 403 → auth (3); moderation 403 → API (5, see
/// module docs); 429 → rate-limit (4) honoring `Retry-After`; ≥500 → server (5, provider-named
/// per SPEC-ERR-012 [DIVERGENCE]); any other non-2xx → API (5).
fn parse_api_error(status: u16, body: &[u8], retry_after: Option<&str>) -> AppError {
    let parsed: WireErrorResponse = serde_json::from_slice(body).unwrap_or_default();
    let err = parsed.error;
    let mut msg = err.message.clone();
    if msg.is_empty() {
        msg = format!("API error (HTTP {status})");
    }

    // Moderation / content-policy (403) → exit 5 (SPEC-ERR-017; see module-docs decision).
    if status == 403 && is_moderation_error(&err) {
        return AppError::api(format!("content policy violation: {msg}"));
    }

    if status == 401 || status == 403 {
        AppError::auth(format!(
            "authentication failed: {msg}\n\nSet OPENROUTER_API_KEY or run: naba config set api_key <your-key>"
        ))
    } else if status == 429 {
        let wait = match retry_after {
            Some(secs) if !secs.is_empty() => {
                format!("Retry after {secs}s and try again.")
            }
            _ => "Wait a moment and try again.".to_string(),
        };
        AppError::rate_limit(format!("rate limit exceeded: {msg}\n\n{wait}"))
    } else if status >= 500 {
        AppError::api(format!(
            "OpenRouter server error: {msg}\n\nThis is a temporary issue. Try again shortly."
        ))
    } else {
        AppError::api(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::exit;
    use crate::provider::GenerateRequest;

    #[test]
    fn empty_model_defaults_to_slug() {
        let p = OpenRouterProvider::new("k", "");
        assert_eq!(p.model, DEFAULT_MODEL);
    }

    #[test]
    fn resolve_model_precedence_ignores_quality() {
        // Unlike Gemini, quality does NOT change the model (SPEC-PROVIDER-005).
        let p = OpenRouterProvider::new("k", "");
        let r = GenerateRequest {
            model: "openai/gpt-image-1".to_string(),
            quality: Some("high".to_string()),
            ..Default::default()
        };
        assert_eq!(p.resolve_model(&r), "openai/gpt-image-1");
        let r = GenerateRequest {
            quality: Some("high".to_string()),
            ..Default::default()
        };
        // quality set, model empty → provider default (NOT a tier swap).
        assert_eq!(p.resolve_model(&r), DEFAULT_MODEL);
    }

    #[test]
    fn is_auto_router_matches() {
        assert!(is_auto_router("openrouter/auto"));
        assert!(is_auto_router("auto"));
        assert!(!is_auto_router(DEFAULT_MODEL));
    }

    #[test]
    fn request_body_shape_text() {
        let wire = WireRequest {
            model: DEFAULT_MODEL,
            prompt: "a cat",
            aspect_ratio: Some("16:9".to_string()),
            resolution: Some("2K".to_string()),
            quality: Some("high".to_string()),
            input_references: Vec::new(),
        };
        let v: serde_json::Value = serde_json::to_value(&wire).unwrap();
        let expected: serde_json::Value = serde_json::json!({
            "model": DEFAULT_MODEL,
            "prompt": "a cat",
            "aspect_ratio": "16:9",
            "resolution": "2K",
            "quality": "high"
        });
        assert_eq!(v, expected);
        // input_references omitted entirely when empty (not `[]`).
        assert!(v.get("input_references").is_none());
    }

    #[test]
    fn request_body_omits_optional_knobs_when_none() {
        let wire = WireRequest {
            model: DEFAULT_MODEL,
            prompt: "hi",
            aspect_ratio: None,
            resolution: None,
            quality: None,
            input_references: Vec::new(),
        };
        let v: serde_json::Value = serde_json::to_value(&wire).unwrap();
        assert_eq!(
            v,
            serde_json::json!({"model": DEFAULT_MODEL, "prompt": "hi"})
        );
    }

    #[test]
    fn request_body_edit_carries_input_reference_data_url() {
        let wire = WireRequest {
            model: DEFAULT_MODEL,
            prompt: "remove bg",
            aspect_ratio: None,
            resolution: None,
            quality: None,
            input_references: vec![WireInputReference {
                kind: "image_url",
                image_url: WireImageUrl {
                    url: format!("data:image/png;base64,{}", BASE64.encode([1u8, 2, 3])),
                },
            }],
        };
        let v: serde_json::Value = serde_json::to_value(&wire).unwrap();
        let refs = &v["input_references"];
        assert_eq!(refs[0]["type"], "image_url");
        assert_eq!(
            refs[0]["image_url"]["url"],
            format!("data:image/png;base64,{}", BASE64.encode([1u8, 2, 3]))
        );
    }

    #[test]
    fn extract_images_decodes_b64_json() {
        let resp = WireResponse {
            data: vec![WireImageData {
                b64_json: BASE64.encode([0xDE, 0xAD, 0xBE, 0xEF]),
                media_type: "image/png".to_string(),
            }],
        };
        let imgs = extract_images(resp).unwrap();
        assert_eq!(imgs.len(), 1);
        assert_eq!(imgs[0].bytes, vec![0xDE, 0xAD, 0xBE, 0xEF]);
        assert_eq!(imgs[0].mime, "image/png");
    }

    #[test]
    fn extract_images_defaults_mime_when_absent() {
        let resp = WireResponse {
            data: vec![WireImageData {
                b64_json: BASE64.encode([1u8]),
                media_type: String::new(),
            }],
        };
        let imgs = extract_images(resp).unwrap();
        assert_eq!(imgs[0].mime, DEFAULT_IMAGE_MIME);
    }

    #[test]
    fn extract_images_no_images_is_exit_5() {
        let resp = WireResponse { data: vec![] };
        let err = extract_images(resp).unwrap_err();
        assert_eq!(err.code, exit::API);
        assert_eq!(err.message, "no images in response");
    }

    #[test]
    fn parse_api_error_mappings() {
        let body401 = br#"{"error":{"message":"No auth credentials found","code":401}}"#;
        let e = parse_api_error(401, body401, None);
        assert_eq!(e.code, exit::AUTH);
        assert!(e
            .message
            .starts_with("authentication failed: No auth credentials found"));

        // Plain permissions 403 (no moderation metadata) → auth (3).
        let body403 = br#"{"error":{"message":"insufficient permissions","code":403}}"#;
        let e = parse_api_error(403, body403, None);
        assert_eq!(e.code, exit::AUTH);

        // Moderation 403 → API (5), content-policy message (SPEC-ERR-017 decision).
        let bodymod = br#"{"error":{"message":"flagged","code":403,"metadata":{"reasons":["violence"],"flagged_input":"..."}}}"#;
        let e = parse_api_error(403, bodymod, None);
        assert_eq!(e.code, exit::API);
        assert!(e.message.starts_with("content policy violation:"));

        // 429 without Retry-After.
        let body429 = br#"{"error":{"message":"rate limited"}}"#;
        let e = parse_api_error(429, body429, None);
        assert_eq!(e.code, exit::RATE_LIMIT);
        assert!(e.message.contains("Wait a moment and try again."));

        // 429 WITH Retry-After honored.
        let e = parse_api_error(429, body429, Some("30"));
        assert_eq!(e.code, exit::RATE_LIMIT);
        assert!(e.message.contains("Retry after 30s"));

        // ≥500 → provider-named server error (5).
        let body500 = br#"{"error":{"message":"boom"}}"#;
        let e = parse_api_error(503, body500, None);
        assert_eq!(e.code, exit::API);
        assert!(e.message.starts_with("OpenRouter server error: boom"));

        // Other non-2xx (400) → API (5), raw message.
        let body400 = br#"{"error":{"message":"invalid model"}}"#;
        let e = parse_api_error(400, body400, None);
        assert_eq!(e.code, exit::API);
        assert_eq!(e.message, "invalid model");

        // Empty body → synthesized message.
        let e = parse_api_error(500, b"", None);
        assert_eq!(e.code, exit::API);
        assert!(e.message.contains("API error (HTTP 500)"));
    }

    #[test]
    fn model_reachable_exact_match() {
        let avail = vec![
            ModelInfo {
                id: "google/gemini-3.1-flash-image-preview".to_string(),
            },
            ModelInfo {
                id: "openai/gpt-image-1".to_string(),
            },
        ];
        assert!(model_reachable("openai/gpt-image-1", &avail));
        assert!(!model_reachable("bytedance-seed/seedream-4.5", &avail));
    }

    // ----------------------------------------------------------------------------------------
    // Integration tests over a wiremock HTTP server (no real API). Assert the OUTGOING request
    // (URL / headers / body), the response decode, the auto guard, and error→exit mapping
    // end-to-end. In-crate because `naba` is a bin crate with no lib target for `tests/`.
    // ----------------------------------------------------------------------------------------
    mod http {
        use super::*;
        use crate::provider::InputImage;
        use serde_json::json;
        use wiremock::matchers::{header, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        fn canned_response(b64: &str) -> serde_json::Value {
            json!({ "data": [{ "b64_json": b64, "media_type": "image/png" }] })
        }

        #[tokio::test]
        async fn generate_text_request_shape() {
            let server = MockServer::start().await;
            let b64 = BASE64.encode([0xDE, 0xAD, 0xBE, 0xEF]);
            Mock::given(method("POST"))
                .and(path("/images"))
                .and(header("content-type", "application/json"))
                .and(header("authorization", "Bearer secret-key"))
                .respond_with(ResponseTemplate::new(200).set_body_json(canned_response(&b64)))
                .mount(&server)
                .await;

            let provider =
                OpenRouterProvider::with_base_url("secret-key", DEFAULT_MODEL, server.uri());
            let req = GenerateRequest {
                prompt: "a corgi".to_string(),
                model: DEFAULT_MODEL.to_string(),
                image_config: ImageConfig::new("16:9", "2K").unwrap(),
                quality: Some("high".to_string()),
                ..Default::default()
            };
            let images = provider.generate(&req).await.unwrap();
            assert_eq!(images.len(), 1);
            assert_eq!(images[0].mime, "image/png");
            assert_eq!(images[0].bytes, vec![0xDE, 0xAD, 0xBE, 0xEF]);

            let reqs = server.received_requests().await.unwrap();
            assert_eq!(reqs.len(), 1);
            let body: serde_json::Value = serde_json::from_slice(&reqs[0].body).unwrap();
            assert_eq!(
                body,
                json!({
                    "model": DEFAULT_MODEL,
                    "prompt": "a corgi",
                    "aspect_ratio": "16:9",
                    "resolution": "2K",
                    "quality": "high"
                })
            );
        }

        #[tokio::test]
        async fn generate_edit_includes_input_reference() {
            let server = MockServer::start().await;
            let b64 = BASE64.encode([1u8, 2, 3]);
            Mock::given(method("POST"))
                .and(path("/images"))
                .respond_with(ResponseTemplate::new(200).set_body_json(canned_response(&b64)))
                .mount(&server)
                .await;

            let provider = OpenRouterProvider::with_base_url("k", DEFAULT_MODEL, server.uri());
            let req = GenerateRequest {
                prompt: "remove background".to_string(),
                model: DEFAULT_MODEL.to_string(),
                input_image: Some(InputImage {
                    bytes: vec![9, 9, 9, 9],
                    mime: "image/jpeg".to_string(),
                }),
                ..Default::default()
            };
            provider.generate(&req).await.unwrap();

            let reqs = server.received_requests().await.unwrap();
            let body: serde_json::Value = serde_json::from_slice(&reqs[0].body).unwrap();
            let refs = &body["input_references"];
            assert_eq!(refs[0]["type"], "image_url");
            assert_eq!(
                refs[0]["image_url"]["url"],
                format!("data:image/jpeg;base64,{}", BASE64.encode([9u8, 9, 9, 9]))
            );
        }

        #[tokio::test]
        async fn generate_auto_rejected_before_api_call() {
            let server = MockServer::start().await;
            // No mock mounted: if the provider hit the API this would 404. The SPEC-PROVIDER-006
            // guard must reject `auto` BEFORE any HTTP call (exit 2).
            let provider = OpenRouterProvider::with_base_url("k", "openrouter/auto", server.uri());
            let req = GenerateRequest {
                prompt: "x".to_string(),
                model: "openrouter/auto".to_string(),
                ..Default::default()
            };
            let err = provider.generate(&req).await.unwrap_err();
            assert_eq!(err.code, exit::USAGE);
            assert!(err.message.contains("cannot generate images"));
            assert!(server.received_requests().await.unwrap().is_empty());
        }

        #[tokio::test]
        async fn error_401_maps_to_exit_3() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .respond_with(ResponseTemplate::new(401).set_body_json(json!({
                    "error": {"message": "No auth credentials found", "code": 401}
                })))
                .mount(&server)
                .await;
            let provider = OpenRouterProvider::with_base_url("k", DEFAULT_MODEL, server.uri());
            let req = GenerateRequest {
                prompt: "x".to_string(),
                model: DEFAULT_MODEL.to_string(),
                ..Default::default()
            };
            let err = provider.generate(&req).await.unwrap_err();
            assert_eq!(err.code, exit::AUTH);
            assert!(err
                .message
                .starts_with("authentication failed: No auth credentials found"));
        }

        #[tokio::test]
        async fn error_429_honors_retry_after_and_maps_to_exit_4() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .respond_with(
                    ResponseTemplate::new(429)
                        .insert_header("Retry-After", "42")
                        .set_body_json(json!({"error": {"message": "slow down"}})),
                )
                .mount(&server)
                .await;
            let provider = OpenRouterProvider::with_base_url("k", DEFAULT_MODEL, server.uri());
            let req = GenerateRequest {
                prompt: "x".to_string(),
                model: DEFAULT_MODEL.to_string(),
                ..Default::default()
            };
            let err = provider.generate(&req).await.unwrap_err();
            assert_eq!(err.code, exit::RATE_LIMIT);
            assert!(err.message.starts_with("rate limit exceeded: slow down"));
            assert!(err.message.contains("Retry after 42s"));
        }

        #[tokio::test]
        async fn error_500_maps_to_exit_5() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .respond_with(ResponseTemplate::new(503).set_body_json(json!({
                    "error": {"message": "backend unavailable"}
                })))
                .mount(&server)
                .await;
            let provider = OpenRouterProvider::with_base_url("k", DEFAULT_MODEL, server.uri());
            let req = GenerateRequest {
                prompt: "x".to_string(),
                model: DEFAULT_MODEL.to_string(),
                ..Default::default()
            };
            let err = provider.generate(&req).await.unwrap_err();
            assert_eq!(err.code, exit::API);
            assert!(err
                .message
                .starts_with("OpenRouter server error: backend unavailable"));
        }

        #[tokio::test]
        async fn moderation_403_maps_to_exit_5() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .respond_with(ResponseTemplate::new(403).set_body_json(json!({
                    "error": {
                        "message": "input flagged",
                        "code": 403,
                        "metadata": {"reasons": ["violence"], "flagged_input": "…"}
                    }
                })))
                .mount(&server)
                .await;
            let provider = OpenRouterProvider::with_base_url("k", DEFAULT_MODEL, server.uri());
            let req = GenerateRequest {
                prompt: "x".to_string(),
                model: DEFAULT_MODEL.to_string(),
                ..Default::default()
            };
            let err = provider.generate(&req).await.unwrap_err();
            assert_eq!(err.code, exit::API);
            assert!(err.message.starts_with("content policy violation:"));
        }

        #[tokio::test]
        async fn no_images_maps_to_exit_5() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": []})))
                .mount(&server)
                .await;
            let provider = OpenRouterProvider::with_base_url("k", DEFAULT_MODEL, server.uri());
            let req = GenerateRequest {
                prompt: "x".to_string(),
                model: DEFAULT_MODEL.to_string(),
                ..Default::default()
            };
            let err = provider.generate(&req).await.unwrap_err();
            assert_eq!(err.code, exit::API);
            assert_eq!(err.message, "no images in response");
        }

        #[tokio::test]
        async fn list_models_reads_data_ids() {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/models"))
                .and(header("authorization", "Bearer k"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "data": [
                        {"id": "google/gemini-3.1-flash-image-preview"},
                        {"id": "openai/gpt-image-1"}
                    ]
                })))
                .mount(&server)
                .await;
            let provider = OpenRouterProvider::with_base_url("k", DEFAULT_MODEL, server.uri());
            let models = provider.list_models().await.unwrap();
            assert_eq!(
                models,
                vec![
                    ModelInfo {
                        id: "google/gemini-3.1-flash-image-preview".to_string()
                    },
                    ModelInfo {
                        id: "openai/gpt-image-1".to_string()
                    },
                ]
            );
            assert!(model_reachable("openai/gpt-image-1", &models));
        }
    }
}
