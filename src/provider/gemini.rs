//! Gemini provider — a faithful async Rust port of the Go `internal/gemini` client
//! (Issue 2.3). Implements the [`Provider`] trait from Issue 2.2.
//!
//! Wire behavior is matched byte-for-byte against the Go source
//! (`client.go` / `types.go` / `imageconfig.go` / `models.go`):
//!
//! * **Endpoint / headers (SPEC-PROVIDER-002).** POST `{base}/models/{model}:generateContent`
//!   with `Content-Type: application/json` + `x-goog-api-key: <key>`. Base URL defaults to
//!   `https://generativelanguage.googleapis.com/v1beta`, overridable via `GEMINI_BASE_URL`.
//!   120s HTTP timeout.
//! * **Model constants (SPEC-PROVIDER-003).** `DEFAULT_MODEL = FLASH_MODEL =
//!   "gemini-3.1-flash-image"`, `PRO_MODEL = "gemini-3-pro-image"`.
//! * **Request shape.** `contents` with a `user` role part carrying the text prompt; when an
//!   input image is present, an additional inline-data part (base64). `generationConfig`
//!   always carries `responseModalities = ["TEXT","IMAGE"]`; `imageConfig` (aspectRatio /
//!   imageSize, each omitempty) is emitted only when present.
//! * **Quality → model tier (SPEC-PROVIDER-005).** `fast` → Flash, `high` → Pro, other →
//!   `ExitUsage`. Exposed as [`model_for_quality`] so the 2.5 selector can call it.
//! * **Per-model image size (SPEC-IMG-007 / naba-a3a).** `512` is model-dependent; see
//!   [`model_supports_size`] and the module's `MODEL → SIZE` table below.
//! * **Error / exit mapping (SPEC-ERR-010..015).** Mirrors Go `parseAPIError` plus the
//!   prompt-blocked / no-images / read-image cases.
//!
//! ## naba-a3a per-model image-size support table (SPEC-IMG-007)
//!
//! | Model                   | 512 | 1K | 2K | 4K |
//! |-------------------------|-----|----|----|----|
//! | gemini-3.1-flash-image  | no  | ✓  | ✓  | ✓  |
//! | gemini-3-pro-image      | no  | ✓  | ✓  | ✓  |
//! | (any other / unknown)   | pass-through — no client-side rejection |
//!
//! **Source.** Bead `naba-a3a` (live smoke, plan-003 Issue 5.2): *"gemini-3.1-flash-image
//! rejects imageConfig.imageSize=512 ('Image size 512 is not supported for this model', HTTP
//! non-200 → ExitAPI 5), though 1K/2K/4K work."* `gemini-3-pro-image` is the sibling model in
//! the same current image-model generation (the prior 512-capable default,
//! `gemini-2.0-flash-exp-image-generation`, was shut down 2025-11-14 — see `client.go`), so it
//! is treated as also rejecting `512`. Unknown/future models are **not** rejected client-side:
//! we let the API remain the authority (the pre-fix behavior), so the fix never wrongly blocks
//! a model that gains 512 support.

use async_trait::async_trait;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::provider::{GenerateRequest, GeneratedImage, ImageConfig, ModelInfo, Provider};

/// Default Gemini base URL (SPEC-PROVIDER-002); overridable via `GEMINI_BASE_URL`.
const DEFAULT_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta";

/// SPEC-PROVIDER-003 model constants.
pub const DEFAULT_MODEL: &str = "gemini-3.1-flash-image";
pub const FLASH_MODEL: &str = "gemini-3.1-flash-image";
pub const PRO_MODEL: &str = "gemini-3-pro-image";

/// Sizes rejected client-side for the known current image models (naba-a3a). See module docs.
const KNOWN_MODEL_SIZES: &[&str] = &["1K", "2K", "4K"];

/// Map a `--quality` alias to a concrete Gemini model id (SPEC-PROVIDER-005): `fast` → Flash,
/// `high` → Pro. Any other value is a usage error (exit 2), verbatim per Go `ModelForQuality`.
///
/// Exposed for the 2.5 selector factory, which owns cross-provider precedence but delegates the
/// Gemini-specific tier mapping here rather than duplicating it.
pub fn model_for_quality(quality: &str) -> Result<&'static str, AppError> {
    match quality {
        "fast" => Ok(FLASH_MODEL),
        "high" => Ok(PRO_MODEL),
        other => Err(AppError::usage(format!(
            "invalid quality {other:?}\n\nValid values: fast, high"
        ))),
    }
}

/// Whether `model` supports image `size` client-side (SPEC-IMG-007 / naba-a3a). Known current
/// image models reject `512`; unknown models pass through (no client-side rejection).
pub fn model_supports_size(model: &str, size: &str) -> bool {
    match model {
        FLASH_MODEL | PRO_MODEL => KNOWN_MODEL_SIZES.contains(&size),
        _ => true,
    }
}

/// The sizes a known model supports, for error messaging. Empty slice → unknown model.
fn supported_sizes_for(model: &str) -> &'static [&'static str] {
    match model {
        FLASH_MODEL | PRO_MODEL => KNOWN_MODEL_SIZES,
        _ => &[],
    }
}

/// Detect an image MIME type from a file extension, matching Go `detectMIMEType`
/// (unknown → `image/png`).
pub fn detect_mime_type(path: &str) -> String {
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        _ => "image/png",
    }
    .to_string()
}

/// Read an image file into an [`InputImage`] for the edit/restore path, mirroring Go
/// `readImageFile`: a read failure yields exit 10 with `read image file %q: %v` (SPEC-ERR-015).
/// The command layer (2.6) uses this to populate [`GenerateRequest::input_image`].
pub fn read_image_file(path: &str) -> Result<crate::provider::InputImage, AppError> {
    let bytes = std::fs::read(path)
        .map_err(|e| AppError::file_io(format!("read image file {path:?}: {e}")))?;
    Ok(crate::provider::InputImage {
        bytes,
        mime: detect_mime_type(path),
    })
}

// ---------------------------------------------------------------------------------------------
// Wire types (serde) — JSON tags match Go `types.go` exactly.
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Serialize)]
struct WireRequest<'a> {
    contents: Vec<WireContent<'a>>,
    #[serde(rename = "generationConfig")]
    generation_config: WireGenerationConfig<'a>,
}

#[derive(Debug, Serialize)]
struct WireContent<'a> {
    role: &'a str,
    parts: Vec<WirePart<'a>>,
}

#[derive(Debug, Serialize)]
struct WirePart<'a> {
    #[serde(skip_serializing_if = "str::is_empty")]
    text: &'a str,
    #[serde(rename = "inlineData", skip_serializing_if = "Option::is_none")]
    inline_data: Option<WireInlineData>,
}

#[derive(Debug, Serialize)]
struct WireInlineData {
    #[serde(rename = "mimeType")]
    mime_type: String,
    data: String,
}

#[derive(Debug, Serialize)]
struct WireGenerationConfig<'a> {
    #[serde(rename = "responseModalities")]
    response_modalities: [&'a str; 2],
    #[serde(rename = "imageConfig", skip_serializing_if = "Option::is_none")]
    image_config: Option<WireImageConfig>,
}

#[derive(Debug, Serialize)]
struct WireImageConfig {
    #[serde(rename = "aspectRatio", skip_serializing_if = "String::is_empty")]
    aspect_ratio: String,
    #[serde(rename = "imageSize", skip_serializing_if = "String::is_empty")]
    image_size: String,
}

impl From<&ImageConfig> for WireImageConfig {
    fn from(cfg: &ImageConfig) -> Self {
        WireImageConfig {
            aspect_ratio: cfg.aspect.clone().unwrap_or_default(),
            image_size: cfg.size.clone().unwrap_or_default(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct WireResponse {
    #[serde(default)]
    candidates: Vec<WireCandidate>,
    #[serde(rename = "promptFeedback", default)]
    prompt_feedback: Option<WirePromptFeedback>,
}

#[derive(Debug, Deserialize)]
struct WireCandidate {
    #[serde(default)]
    content: Option<WireRespContent>,
}

#[derive(Debug, Deserialize)]
struct WireRespContent {
    #[serde(default)]
    parts: Vec<WireRespPart>,
}

#[derive(Debug, Deserialize)]
struct WireRespPart {
    #[serde(rename = "inlineData", default)]
    inline_data: Option<WireRespInlineData>,
}

#[derive(Debug, Deserialize)]
struct WireRespInlineData {
    #[serde(rename = "mimeType", default)]
    mime_type: String,
    #[serde(default)]
    data: String,
}

#[derive(Debug, Deserialize)]
struct WirePromptFeedback {
    #[serde(rename = "blockReason", default)]
    block_reason: String,
}

#[derive(Debug, Default, Deserialize)]
struct WireErrorResponse {
    #[serde(default)]
    error: WireErrorBody,
}

#[derive(Debug, Default, Deserialize)]
struct WireErrorBody {
    #[serde(default)]
    message: String,
}

#[derive(Debug, Deserialize)]
struct WireModelList {
    #[serde(default)]
    models: Vec<WireModel>,
}

#[derive(Debug, Deserialize)]
struct WireModel {
    #[serde(default)]
    name: String,
}

// ---------------------------------------------------------------------------------------------
// Provider
// ---------------------------------------------------------------------------------------------

/// Async Gemini provider (port of the Go `gemini.Client`).
pub struct GeminiProvider {
    api_key: String,
    model: String,
    base_url: String,
    http: reqwest::Client,
}

impl GeminiProvider {
    /// Construct a provider. An empty `model` falls back to [`DEFAULT_MODEL`]. The base URL is
    /// taken from `GEMINI_BASE_URL` when set, else the SPEC default. 120s HTTP timeout.
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        let model = model.into();
        let model = if model.is_empty() {
            DEFAULT_MODEL.to_string()
        } else {
            model
        };
        let base_url = std::env::var("GEMINI_BASE_URL")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());
        Self::with_base_url(api_key, model, base_url)
    }

    /// Construct with an explicit base URL, bypassing the `GEMINI_BASE_URL` env lookup. Used by
    /// mock-server integration tests (avoids process-global env races) and available to the 2.5
    /// selector if it wants to inject a base URL directly. An empty `model` still defaults.
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

    /// Resolve the effective model for a request: an explicit `req.model` wins; otherwise a raw
    /// `quality` maps via the Gemini tier (SPEC-PROVIDER-005); otherwise the provider's own
    /// configured model. The 2.5 selector normally sets `req.model` up front — this keeps the
    /// provider correct standalone and is the seam the selector plugs into.
    fn resolve_model(&self, req: &GenerateRequest) -> Result<String, AppError> {
        if !req.model.is_empty() {
            return Ok(req.model.clone());
        }
        if let Some(q) = req.quality.as_deref() {
            if !q.is_empty() {
                return Ok(model_for_quality(q)?.to_string());
            }
        }
        Ok(self.model.clone())
    }

    /// Apply the naba-a3a per-model image-size gate (SPEC-IMG-007). Rejects an unsupported size
    /// for the resolved model with a provider/model-specific usage error (exit 2) rather than
    /// letting the API fail it (exit 5).
    fn check_image_size(&self, model: &str, cfg: &Option<ImageConfig>) -> Result<(), AppError> {
        if let Some(cfg) = cfg {
            if let Some(size) = cfg.size.as_deref() {
                if !size.is_empty() && !model_supports_size(model, size) {
                    return Err(AppError::usage(format!(
                        "image size {size:?} is not supported by gemini model {model:?}\n\nSupported sizes for this model: {}",
                        supported_sizes_for(model).join(", ")
                    )));
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl Provider for GeminiProvider {
    fn name(&self) -> &str {
        "gemini"
    }

    async fn generate(&self, req: &GenerateRequest) -> Result<Vec<GeneratedImage>, AppError> {
        let model = self.resolve_model(req)?;
        self.check_image_size(&model, &req.image_config)?;

        // Build parts: text always; inline image part when input_image is present (edit/restore).
        let mut parts = vec![WirePart {
            text: &req.prompt,
            inline_data: None,
        }];
        if let Some(img) = req.input_image.as_ref() {
            parts.push(WirePart {
                text: "",
                inline_data: Some(WireInlineData {
                    mime_type: img.mime.clone(),
                    data: BASE64.encode(&img.bytes),
                }),
            });
        }

        let wire = WireRequest {
            contents: vec![WireContent {
                role: "user",
                parts,
            }],
            generation_config: WireGenerationConfig {
                response_modalities: ["TEXT", "IMAGE"],
                image_config: req.image_config.as_ref().map(WireImageConfig::from),
            },
        };

        let url = format!("{}/models/{}:generateContent", self.base_url, model);
        let resp = self
            .http
            .post(&url)
            .header("Content-Type", "application/json")
            .header("x-goog-api-key", &self.api_key)
            .json(&wire)
            .send()
            .await
            .map_err(|e| AppError::api(format!("api request failed: {e}")))?;

        let status = resp.status();
        let body = resp
            .bytes()
            .await
            .map_err(|e| AppError::api(format!("read response: {e}")))?;

        if !status.is_success() {
            return Err(parse_api_error(status.as_u16(), &body));
        }

        let parsed: WireResponse = serde_json::from_slice(&body)
            .map_err(|e| AppError::api(format!("parse response: {e}")))?;

        if let Some(fb) = parsed.prompt_feedback.as_ref() {
            if !fb.block_reason.is_empty() {
                return Err(AppError::api(format!(
                    "prompt blocked: {}",
                    fb.block_reason
                )));
            }
        }

        extract_images(parsed)
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>, AppError> {
        let url = format!("{}/models?pageSize=1000", self.base_url);
        let resp = self
            .http
            .get(&url)
            .header("x-goog-api-key", &self.api_key)
            .send()
            .await
            .map_err(|e| AppError::api(format!("api request failed: {e}")))?;

        let status = resp.status();
        let body = resp
            .bytes()
            .await
            .map_err(|e| AppError::api(format!("read response: {e}")))?;

        if !status.is_success() {
            return Err(parse_api_error(status.as_u16(), &body));
        }

        let parsed: WireModelList = serde_json::from_slice(&body)
            .map_err(|e| AppError::api(format!("parse response: {e}")))?;

        Ok(parsed
            .models
            .into_iter()
            .map(|m| ModelInfo {
                id: strip_models_prefix(&m.name).to_string(),
            })
            .collect())
    }
}

/// Strip the `models/` prefix Gemini uses on model resource names.
fn strip_models_prefix(name: &str) -> &str {
    name.strip_prefix("models/").unwrap_or(name)
}

/// Prefix-normalized membership check `naba doctor` uses (port of Go `ModelReachable`).
pub fn model_reachable(model_id: &str, available: &[ModelInfo]) -> bool {
    let want = strip_models_prefix(model_id);
    available.iter().any(|m| strip_models_prefix(&m.id) == want)
}

/// Extract inline-data image parts into decoded [`GeneratedImage`]s, mirroring Go
/// `extractImages`: no images → exit 5 `no images in response` (SPEC-ERR-014).
fn extract_images(resp: WireResponse) -> Result<Vec<GeneratedImage>, AppError> {
    let mut images = Vec::new();
    for candidate in resp.candidates {
        let Some(content) = candidate.content else {
            continue;
        };
        for part in content.parts {
            let Some(inline) = part.inline_data else {
                continue;
            };
            let bytes = BASE64
                .decode(inline.data.as_bytes())
                .map_err(|e| AppError::api(format!("decode image data: {e}")))?;
            images.push(GeneratedImage {
                bytes,
                mime: inline.mime_type,
            });
        }
    }
    if images.is_empty() {
        return Err(AppError::api("no images in response"));
    }
    Ok(images)
}

/// Map a non-2xx HTTP status + body to an [`AppError`] with the right exit code, mirroring Go
/// `parseAPIError` (SPEC-ERR-010..012). 401/403 → auth (3); 429 → rate-limit (4); ≥500 →
/// server (5); any other non-2xx → API (5).
fn parse_api_error(status: u16, body: &[u8]) -> AppError {
    let parsed: WireErrorResponse = serde_json::from_slice(body).unwrap_or_default();
    let mut msg = parsed.error.message;
    if msg.is_empty() {
        msg = format!("API error (HTTP {status})");
    }

    if status == 401 || status == 403 {
        AppError::auth(format!(
            "authentication failed: {msg}\n\nSet GEMINI_API_KEY or run: naba config set api_key <your-key>"
        ))
    } else if status == 429 {
        AppError::rate_limit(format!(
            "rate limit exceeded: {msg}\n\nWait a moment and try again."
        ))
    } else if status >= 500 {
        AppError::api(format!(
            "Gemini server error: {msg}\n\nThis is a temporary issue. Try again shortly."
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
    fn model_for_quality_maps_tiers() {
        assert_eq!(model_for_quality("fast").unwrap(), FLASH_MODEL);
        assert_eq!(model_for_quality("high").unwrap(), PRO_MODEL);
        let err = model_for_quality("medium").unwrap_err();
        assert_eq!(err.code, exit::USAGE);
        assert_eq!(
            err.message,
            "invalid quality \"medium\"\n\nValid values: fast, high"
        );
    }

    #[test]
    fn model_supports_size_naba_a3a() {
        // naba-a3a: flash and pro reject 512; 1K/2K/4K accepted; unknown models pass through.
        assert!(!model_supports_size(FLASH_MODEL, "512"));
        assert!(!model_supports_size(PRO_MODEL, "512"));
        assert!(model_supports_size(FLASH_MODEL, "1K"));
        assert!(model_supports_size(FLASH_MODEL, "4K"));
        assert!(model_supports_size("some-future-model", "512"));
    }

    #[test]
    fn empty_model_defaults_to_flash() {
        let p = GeminiProvider::new("k", "");
        assert_eq!(p.model, DEFAULT_MODEL);
    }

    #[test]
    fn resolve_model_precedence() {
        let p = GeminiProvider::new("k", "");
        // explicit model wins
        let r = GenerateRequest {
            model: "custom-model".to_string(),
            quality: Some("high".to_string()),
            ..Default::default()
        };
        assert_eq!(p.resolve_model(&r).unwrap(), "custom-model");
        // quality maps when no explicit model
        let r = GenerateRequest {
            quality: Some("high".to_string()),
            ..Default::default()
        };
        assert_eq!(p.resolve_model(&r).unwrap(), PRO_MODEL);
        // falls back to provider model
        let r = GenerateRequest::default();
        assert_eq!(p.resolve_model(&r).unwrap(), DEFAULT_MODEL);
    }

    #[test]
    fn check_image_size_rejects_512_for_flash() {
        let p = GeminiProvider::new("k", FLASH_MODEL);
        let cfg = Some(ImageConfig {
            aspect: None,
            size: Some("512".to_string()),
        });
        let err = p.check_image_size(FLASH_MODEL, &cfg).unwrap_err();
        assert_eq!(err.code, exit::USAGE);
        assert_eq!(
            err.message,
            "image size \"512\" is not supported by gemini model \"gemini-3.1-flash-image\"\n\nSupported sizes for this model: 1K, 2K, 4K"
        );
        // 2K is fine
        let cfg = Some(ImageConfig {
            aspect: None,
            size: Some("2K".to_string()),
        });
        assert!(p.check_image_size(FLASH_MODEL, &cfg).is_ok());
    }

    #[test]
    fn detect_mime_type_matches_go() {
        assert_eq!(detect_mime_type("a.png"), "image/png");
        assert_eq!(detect_mime_type("a.JPG"), "image/jpeg");
        assert_eq!(detect_mime_type("a.jpeg"), "image/jpeg");
        assert_eq!(detect_mime_type("a.gif"), "image/gif");
        assert_eq!(detect_mime_type("a.webp"), "image/webp");
        assert_eq!(detect_mime_type("a.bmp"), "image/bmp");
        assert_eq!(detect_mime_type("a.unknown"), "image/png");
        assert_eq!(detect_mime_type("noext"), "image/png");
    }

    #[test]
    fn read_image_file_missing_is_exit_10() {
        let err = read_image_file("/nonexistent/path/to/image.png").unwrap_err();
        assert_eq!(err.code, exit::FILE_IO);
        assert!(err
            .message
            .starts_with("read image file \"/nonexistent/path/to/image.png\": "));
    }

    #[test]
    fn model_reachable_normalizes_prefix() {
        let avail = vec![
            ModelInfo {
                id: "gemini-3.1-flash-image".to_string(),
            },
            ModelInfo {
                id: "gemini-3-pro-image".to_string(),
            },
        ];
        assert!(model_reachable("gemini-3.1-flash-image", &avail));
        assert!(model_reachable("models/gemini-3-pro-image", &avail));
        assert!(!model_reachable("gemini-nonexistent", &avail));
    }

    #[test]
    fn request_body_shape_text_with_image_config() {
        // Prove the emitted JSON matches the Go wire shape exactly.
        let img_cfg = WireImageConfig {
            aspect_ratio: "16:9".to_string(),
            image_size: "2K".to_string(),
        };
        let wire = WireRequest {
            contents: vec![WireContent {
                role: "user",
                parts: vec![WirePart {
                    text: "a cat",
                    inline_data: None,
                }],
            }],
            generation_config: WireGenerationConfig {
                response_modalities: ["TEXT", "IMAGE"],
                image_config: Some(img_cfg),
            },
        };
        let v: serde_json::Value = serde_json::to_value(&wire).unwrap();
        let expected: serde_json::Value = serde_json::json!({
            "contents": [
                {"role": "user", "parts": [{"text": "a cat"}]}
            ],
            "generationConfig": {
                "responseModalities": ["TEXT", "IMAGE"],
                "imageConfig": {"aspectRatio": "16:9", "imageSize": "2K"}
            }
        });
        assert_eq!(v, expected);
    }

    #[test]
    fn request_body_omits_image_config_when_none() {
        let wire = WireRequest {
            contents: vec![WireContent {
                role: "user",
                parts: vec![WirePart {
                    text: "hi",
                    inline_data: None,
                }],
            }],
            generation_config: WireGenerationConfig {
                response_modalities: ["TEXT", "IMAGE"],
                image_config: None,
            },
        };
        let v: serde_json::Value = serde_json::to_value(&wire).unwrap();
        assert!(v["generationConfig"].get("imageConfig").is_none());
        // text-only part must not carry an inlineData key
        assert!(v["contents"][0]["parts"][0].get("inlineData").is_none());
    }

    #[test]
    fn request_body_edit_carries_inline_data() {
        let wire = WireRequest {
            contents: vec![WireContent {
                role: "user",
                parts: vec![
                    WirePart {
                        text: "edit",
                        inline_data: None,
                    },
                    WirePart {
                        text: "",
                        inline_data: Some(WireInlineData {
                            mime_type: "image/png".to_string(),
                            data: BASE64.encode([1u8, 2, 3]),
                        }),
                    },
                ],
            }],
            generation_config: WireGenerationConfig {
                response_modalities: ["TEXT", "IMAGE"],
                image_config: None,
            },
        };
        let v: serde_json::Value = serde_json::to_value(&wire).unwrap();
        let parts = &v["contents"][0]["parts"];
        assert_eq!(parts[0]["text"], "edit");
        assert!(parts[0].get("inlineData").is_none());
        // image part: no "text" key (omitempty), inlineData present with camelCase mimeType
        assert!(parts[1].get("text").is_none());
        assert_eq!(parts[1]["inlineData"]["mimeType"], "image/png");
        assert_eq!(parts[1]["inlineData"]["data"], BASE64.encode([1u8, 2, 3]));
    }

    #[test]
    fn parse_api_error_mappings() {
        let body = br#"{"error":{"code":401,"message":"bad key","status":"UNAUTHENTICATED"}}"#;
        let e = parse_api_error(401, body);
        assert_eq!(e.code, exit::AUTH);
        assert_eq!(
            e.message,
            "authentication failed: bad key\n\nSet GEMINI_API_KEY or run: naba config set api_key <your-key>"
        );

        let e = parse_api_error(403, body);
        assert_eq!(e.code, exit::AUTH);

        let body429 = br#"{"error":{"message":"quota"}}"#;
        let e = parse_api_error(429, body429);
        assert_eq!(e.code, exit::RATE_LIMIT);
        assert_eq!(
            e.message,
            "rate limit exceeded: quota\n\nWait a moment and try again."
        );

        let body500 = br#"{"error":{"message":"boom"}}"#;
        let e = parse_api_error(500, body500);
        assert_eq!(e.code, exit::API);
        assert_eq!(
            e.message,
            "Gemini server error: boom\n\nThis is a temporary issue. Try again shortly."
        );

        // other non-2xx (e.g. 400) → API (5), raw message
        let body400 = br#"{"error":{"message":"invalid arg"}}"#;
        let e = parse_api_error(400, body400);
        assert_eq!(e.code, exit::API);
        assert_eq!(e.message, "invalid arg");

        // empty body → synthesized message
        let e = parse_api_error(500, b"");
        assert_eq!(e.code, exit::API);
        assert!(e.message.contains("API error (HTTP 500)"));
    }

    #[test]
    fn extract_images_no_images_is_exit_5() {
        let resp = WireResponse {
            candidates: vec![WireCandidate {
                content: Some(WireRespContent { parts: vec![] }),
            }],
            prompt_feedback: None,
        };
        let err = extract_images(resp).unwrap_err();
        assert_eq!(err.code, exit::API);
        assert_eq!(err.message, "no images in response");
    }

    // ----------------------------------------------------------------------------------------
    // Integration tests over a wiremock HTTP server (no real API). These assert the OUTGOING
    // request shape (URL / headers / body) and the error→exit mapping end-to-end. They live in
    // the crate's test module because `naba` is a bin crate with no lib target for `tests/`.
    // ----------------------------------------------------------------------------------------
    mod http {
        use super::*;
        use crate::provider::InputImage;
        use serde_json::json;
        use wiremock::matchers::{header, method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        fn canned_response(b64: &str) -> serde_json::Value {
            json!({
                "candidates": [{
                    "content": {
                        "role": "model",
                        "parts": [{"inlineData": {"mimeType": "image/png", "data": b64}}]
                    },
                    "finishReason": "STOP"
                }]
            })
        }

        #[tokio::test]
        async fn generate_text_request_shape() {
            let server = MockServer::start().await;
            let b64 = BASE64.encode([0xDE, 0xAD, 0xBE, 0xEF]);
            Mock::given(method("POST"))
                .and(path("/models/gemini-3.1-flash-image:generateContent"))
                .and(header("content-type", "application/json"))
                .and(header("x-goog-api-key", "secret-key"))
                .respond_with(ResponseTemplate::new(200).set_body_json(canned_response(&b64)))
                .mount(&server)
                .await;

            let provider = GeminiProvider::with_base_url("secret-key", FLASH_MODEL, server.uri());
            let req = GenerateRequest {
                prompt: "a corgi".to_string(),
                model: FLASH_MODEL.to_string(),
                image_config: ImageConfig::new("16:9", "2K").unwrap(),
                ..Default::default()
            };
            let images = provider.generate(&req).await.unwrap();
            assert_eq!(images.len(), 1);
            assert_eq!(images[0].mime, "image/png");
            assert_eq!(images[0].bytes, vec![0xDE, 0xAD, 0xBE, 0xEF]);

            // Assert the outgoing body shape.
            let reqs = server.received_requests().await.unwrap();
            assert_eq!(reqs.len(), 1);
            let body: serde_json::Value = serde_json::from_slice(&reqs[0].body).unwrap();
            assert_eq!(
                body,
                json!({
                    "contents": [{"role": "user", "parts": [{"text": "a corgi"}]}],
                    "generationConfig": {
                        "responseModalities": ["TEXT", "IMAGE"],
                        "imageConfig": {"aspectRatio": "16:9", "imageSize": "2K"}
                    }
                })
            );
        }

        #[tokio::test]
        async fn generate_edit_includes_inline_image() {
            let server = MockServer::start().await;
            let b64 = BASE64.encode([1u8, 2, 3]);
            Mock::given(method("POST"))
                .and(path("/models/gemini-3.1-flash-image:generateContent"))
                .respond_with(ResponseTemplate::new(200).set_body_json(canned_response(&b64)))
                .mount(&server)
                .await;

            let provider = GeminiProvider::with_base_url("k", FLASH_MODEL, server.uri());
            let req = GenerateRequest {
                prompt: "remove background".to_string(),
                model: FLASH_MODEL.to_string(),
                input_image: Some(InputImage {
                    bytes: vec![9, 9, 9, 9],
                    mime: "image/jpeg".to_string(),
                }),
                ..Default::default()
            };
            provider.generate(&req).await.unwrap();

            let reqs = server.received_requests().await.unwrap();
            let body: serde_json::Value = serde_json::from_slice(&reqs[0].body).unwrap();
            let parts = &body["contents"][0]["parts"];
            assert_eq!(parts[0]["text"], "remove background");
            assert_eq!(parts[1]["inlineData"]["mimeType"], "image/jpeg");
            assert_eq!(
                parts[1]["inlineData"]["data"],
                BASE64.encode([9u8, 9, 9, 9])
            );
            // No imageConfig emitted when none supplied.
            assert!(body["generationConfig"].get("imageConfig").is_none());
        }

        #[tokio::test]
        async fn generate_quality_high_maps_to_pro_model_in_url() {
            let server = MockServer::start().await;
            let b64 = BASE64.encode([7u8]);
            Mock::given(method("POST"))
                .and(path("/models/gemini-3-pro-image:generateContent"))
                .respond_with(ResponseTemplate::new(200).set_body_json(canned_response(&b64)))
                .mount(&server)
                .await;

            // Empty req.model + quality=high → Pro model tier in the request path.
            let provider = GeminiProvider::with_base_url("k", "", server.uri());
            let req = GenerateRequest {
                prompt: "x".to_string(),
                model: String::new(),
                quality: Some("high".to_string()),
                ..Default::default()
            };
            provider.generate(&req).await.unwrap();
            let reqs = server.received_requests().await.unwrap();
            assert_eq!(
                reqs[0].url.path(),
                "/models/gemini-3-pro-image:generateContent"
            );
        }

        #[tokio::test]
        async fn generate_512_rejected_before_api_call() {
            let server = MockServer::start().await;
            // No mock mounted: if the provider hit the API this would 404/panic. The naba-a3a
            // gate must reject 512 BEFORE any HTTP call.
            let provider = GeminiProvider::with_base_url("k", FLASH_MODEL, server.uri());
            let req = GenerateRequest {
                prompt: "x".to_string(),
                model: FLASH_MODEL.to_string(),
                image_config: Some(ImageConfig {
                    aspect: None,
                    size: Some("512".to_string()),
                }),
                ..Default::default()
            };
            let err = provider.generate(&req).await.unwrap_err();
            assert_eq!(err.code, exit::USAGE);
            assert!(err.message.contains("image size \"512\" is not supported"));
            // Confirm nothing was sent.
            assert!(server.received_requests().await.unwrap().is_empty());
        }

        #[tokio::test]
        async fn error_401_maps_to_exit_3() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .respond_with(ResponseTemplate::new(401).set_body_json(json!({
                    "error": {"code": 401, "message": "API key invalid", "status": "UNAUTHENTICATED"}
                })))
                .mount(&server)
                .await;
            let provider = GeminiProvider::with_base_url("k", FLASH_MODEL, server.uri());
            let req = GenerateRequest {
                prompt: "x".to_string(),
                model: FLASH_MODEL.to_string(),
                ..Default::default()
            };
            let err = provider.generate(&req).await.unwrap_err();
            assert_eq!(err.code, exit::AUTH);
            assert!(err
                .message
                .starts_with("authentication failed: API key invalid"));
        }

        #[tokio::test]
        async fn error_429_maps_to_exit_4() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .respond_with(ResponseTemplate::new(429).set_body_json(json!({
                    "error": {"message": "Resource exhausted"}
                })))
                .mount(&server)
                .await;
            let provider = GeminiProvider::with_base_url("k", FLASH_MODEL, server.uri());
            let req = GenerateRequest {
                prompt: "x".to_string(),
                model: FLASH_MODEL.to_string(),
                ..Default::default()
            };
            let err = provider.generate(&req).await.unwrap_err();
            assert_eq!(err.code, exit::RATE_LIMIT);
            assert!(err
                .message
                .starts_with("rate limit exceeded: Resource exhausted"));
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
            let provider = GeminiProvider::with_base_url("k", FLASH_MODEL, server.uri());
            let req = GenerateRequest {
                prompt: "x".to_string(),
                model: FLASH_MODEL.to_string(),
                ..Default::default()
            };
            let err = provider.generate(&req).await.unwrap_err();
            assert_eq!(err.code, exit::API);
            assert!(err
                .message
                .starts_with("Gemini server error: backend unavailable"));
        }

        #[tokio::test]
        async fn prompt_blocked_maps_to_exit_5() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "promptFeedback": {"blockReason": "SAFETY"}
                })))
                .mount(&server)
                .await;
            let provider = GeminiProvider::with_base_url("k", FLASH_MODEL, server.uri());
            let req = GenerateRequest {
                prompt: "x".to_string(),
                model: FLASH_MODEL.to_string(),
                ..Default::default()
            };
            let err = provider.generate(&req).await.unwrap_err();
            assert_eq!(err.code, exit::API);
            assert_eq!(err.message, "prompt blocked: SAFETY");
        }

        #[tokio::test]
        async fn no_images_maps_to_exit_5() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "candidates": [{"content": {"role": "model", "parts": []}}]
                })))
                .mount(&server)
                .await;
            let provider = GeminiProvider::with_base_url("k", FLASH_MODEL, server.uri());
            let req = GenerateRequest {
                prompt: "x".to_string(),
                model: FLASH_MODEL.to_string(),
                ..Default::default()
            };
            let err = provider.generate(&req).await.unwrap_err();
            assert_eq!(err.code, exit::API);
            assert_eq!(err.message, "no images in response");
        }

        #[tokio::test]
        async fn list_models_strips_prefix_and_queries_page_size() {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/models"))
                .and(query_param("pageSize", "1000"))
                .and(header("x-goog-api-key", "k"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "models": [
                        {"name": "models/gemini-3.1-flash-image"},
                        {"name": "models/gemini-3-pro-image"}
                    ]
                })))
                .mount(&server)
                .await;
            let provider = GeminiProvider::with_base_url("k", FLASH_MODEL, server.uri());
            let models = provider.list_models().await.unwrap();
            assert_eq!(
                models,
                vec![
                    ModelInfo {
                        id: "gemini-3.1-flash-image".to_string()
                    },
                    ModelInfo {
                        id: "gemini-3-pro-image".to_string()
                    },
                ]
            );
            assert!(model_reachable("models/gemini-3-pro-image", &models));
        }
    }
}
