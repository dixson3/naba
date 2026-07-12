//! Provider abstraction: the async `Provider` trait plus the shared request/response
//! model that both concrete providers (Gemini — Issue 2.3, OpenRouter — Issue 2.4) and
//! the command layer build on. This module is the ABSTRACTION only: no HTTP lives here.
//!
//! Design notes tied to SPEC §5:
//!
//! * **Per-provider `quality` semantics (SPEC-PROVIDER-005).** [`GenerateRequest`] carries
//!   the RAW `--quality` string; the trait never interprets it. Each impl resolves it:
//!   Gemini maps `fast`/`high` → model tier (Flash/Pro); OpenRouter passes it as the native
//!   `quality` request parameter without swapping the model. Keeping the raw value on the
//!   request is what lets one shared model serve both semantics.
//!
//! * **Model-aware image-size validation (SPEC-IMG-007 / naba-a3a).** [`ImageConfig::new`]
//!   is only the GLOBAL first gate ([`VALID_IMAGE_SIZES`]). It deliberately does not know
//!   which sizes a given model supports — e.g. `512` is model-dependent. The raw `size`
//!   value is preserved on the struct so a provider impl in 2.3/2.4 can further reject a
//!   size the *model* doesn't support, with a provider/model-specific message, rather than
//!   this global gate hard-failing `512` for everyone.
//!
//! * **One method for image input (folded), not two.** The Go client exposes both
//!   `GenerateWithConfig` and `GenerateWithImageConfig`. In Rust we fold the input image
//!   into a single [`Provider::generate`] via the optional [`GenerateRequest::input_image`]:
//!   `None` → text-to-image; `Some` → edit/restore. This is the cleaner Rust shape (one
//!   code path, no duplicated config plumbing) and matches how the command layer already
//!   branches on whether an input image was supplied.

use async_trait::async_trait;

use crate::error::AppError;

pub mod gemini;
pub mod openrouter;
pub mod select;

// Re-exported for the 2.5 selector factory / 2.6 command layer (not yet wired).
#[allow(unused_imports)]
pub use gemini::{model_for_quality, GeminiProvider};
#[allow(unused_imports)]
pub use openrouter::OpenRouterProvider;
// The 2.5 selector-factory surface, for the command layer (Issue 4.1) to wire in.
#[allow(unused_imports)]
pub use select::{
    build_provider, missing_key_error, resolve_selection, select_provider, ConfigDefaults, EnvKeys,
    Selection, SelectionInputs,
};

/// Valid aspect ratios for `imageConfig.aspectRatio` (SPEC-IMG-001, verbatim + order-preserving).
pub const VALID_ASPECT_RATIOS: &[&str] = &[
    "1:1", "1:4", "1:8", "2:3", "3:2", "3:4", "4:1", "4:3", "4:5", "5:4", "8:1", "9:16", "16:9",
    "21:9",
];

/// Valid image sizes for `imageConfig.imageSize` (SPEC-IMG-002). Uppercase `K`; lowercase is
/// rejected. This is only the GLOBAL first gate — per-model validity is a provider concern
/// (SPEC-IMG-007).
pub const VALID_IMAGE_SIZES: &[&str] = &["512", "1K", "2K", "4K"];

/// naba's `imageConfig` knobs (SPEC-IMG-003): aspect ratio and image size (a.k.a. resolution).
///
/// Both fields are optional. When both are empty the command layer sends NO `imageConfig`
/// at all, keeping the request byte-identical to a bare call (SPEC-IMG-005) — [`ImageConfig::new`]
/// returns `Ok(None)` in that case.
///
/// The `size` value is retained raw so a provider impl can apply model-aware rejection on top
/// of the global gate (SPEC-IMG-007).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImageConfig {
    /// `generationConfig.imageConfig.aspectRatio` (OpenRouter: `aspect_ratio`).
    pub aspect: Option<String>,
    /// `generationConfig.imageConfig.imageSize` (OpenRouter: `resolution`).
    pub size: Option<String>,
}

impl ImageConfig {
    /// Build a validated `ImageConfig` from raw `aspect`/`resolution` strings.
    ///
    /// * Both empty → `Ok(None)`: no `imageConfig` is sent (SPEC-IMG-005, byte-identical).
    /// * Invalid aspect → `Err` `ExitUsage` (exit 2) with the verbatim SPEC-IMG-005 message.
    /// * Invalid size  → `Err` `ExitUsage` (exit 2) with the verbatim SPEC-IMG-005 message.
    ///   This is the GLOBAL gate only; per-model rejection (e.g. a model that can't do `512`)
    ///   is layered on by the provider impl (SPEC-IMG-007).
    pub fn new(aspect: &str, resolution: &str) -> Result<Option<ImageConfig>, AppError> {
        if aspect.is_empty() && resolution.is_empty() {
            return Ok(None);
        }
        if !aspect.is_empty() && !VALID_ASPECT_RATIOS.contains(&aspect) {
            return Err(AppError::usage(format!(
                "invalid aspect ratio \"{}\"\n\nValid values: {}",
                aspect,
                VALID_ASPECT_RATIOS.join(", ")
            )));
        }
        if !resolution.is_empty() && !VALID_IMAGE_SIZES.contains(&resolution) {
            return Err(AppError::usage(format!(
                "invalid resolution \"{}\"\n\nValid values: {}",
                resolution,
                VALID_IMAGE_SIZES.join(", ")
            )));
        }
        Ok(Some(ImageConfig {
            aspect: (!aspect.is_empty()).then(|| aspect.to_string()),
            size: (!resolution.is_empty()).then(|| resolution.to_string()),
        }))
    }
}

/// An input image for the edit/restore path: raw bytes plus their MIME type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputImage {
    pub bytes: Vec<u8>,
    pub mime: String,
}

/// The shared request model every provider consumes.
///
/// `quality` is the RAW `--quality` value (SPEC-PROVIDER-005): the trait never interprets it;
/// each provider resolves it (Gemini → model tier, OpenRouter → native `quality` param). When
/// `input_image` is `Some`, this is an edit/restore request; when `None`, text-to-image.
#[derive(Debug, Clone, Default)]
pub struct GenerateRequest {
    /// The text prompt.
    pub prompt: String,
    /// The resolved model id/slug for this request.
    pub model: String,
    /// Optional aspect/size knobs. `None` → send no `imageConfig` (SPEC-IMG-005).
    pub image_config: Option<ImageConfig>,
    /// Optional input image for edit/restore. `None` → text-to-image.
    pub input_image: Option<InputImage>,
    /// RAW `--quality` value (SPEC-PROVIDER-005), interpreted per-provider. `None` → unset.
    pub quality: Option<String>,
}

/// One decoded image from a provider response. A response may carry several.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedImage {
    /// Decoded image bytes (already base64-decoded from the wire).
    pub bytes: Vec<u8>,
    /// MIME type of the image (e.g. `image/png`).
    pub mime: String,
}

/// A model advertised by a provider (for `naba doctor`'s `list_models` check).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelInfo {
    /// Model id/slug (provider-normalized, e.g. Gemini strips the `models/` prefix).
    pub id: String,
}

/// The async provider abstraction. Both Gemini (2.3) and OpenRouter (2.4) implement this.
///
/// Image input is folded into [`Provider::generate`] via [`GenerateRequest::input_image`]
/// rather than exposing a separate `generate_with_image` — one code path for both text-to-image
/// and edit/restore (see module docs).
#[async_trait]
pub trait Provider: Send + Sync {
    /// Stable provider identifier, e.g. `"gemini"` or `"openrouter"`.
    fn name(&self) -> &str;

    /// Generate one or more images. Text-to-image when `req.input_image` is `None`;
    /// edit/restore when it is `Some`.
    async fn generate(&self, req: &GenerateRequest) -> Result<Vec<GeneratedImage>, AppError>;

    /// List models the provider advertises (cheap liveness/key check for `naba doctor`).
    async fn list_models(&self) -> Result<Vec<ModelInfo>, AppError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::exit;

    /// An in-memory test double so the trait and model can be exercised without HTTP.
    struct MockProvider {
        name: String,
        images: Vec<GeneratedImage>,
        models: Vec<ModelInfo>,
        /// Captures the last request's raw quality, to prove it round-trips untouched.
        seen_quality: std::sync::Mutex<Option<String>>,
    }

    impl MockProvider {
        fn new() -> Self {
            Self {
                name: "mock".to_string(),
                images: vec![GeneratedImage {
                    bytes: vec![1, 2, 3],
                    mime: "image/png".to_string(),
                }],
                models: vec![ModelInfo {
                    id: "mock-model".to_string(),
                }],
                seen_quality: std::sync::Mutex::new(None),
            }
        }
    }

    #[async_trait]
    impl Provider for MockProvider {
        fn name(&self) -> &str {
            &self.name
        }

        async fn generate(&self, req: &GenerateRequest) -> Result<Vec<GeneratedImage>, AppError> {
            // Record the raw quality exactly as received — the trait does NOT interpret it.
            *self.seen_quality.lock().unwrap() = req.quality.clone();
            Ok(self.images.clone())
        }

        async fn list_models(&self) -> Result<Vec<ModelInfo>, AppError> {
            Ok(self.models.clone())
        }
    }

    #[test]
    fn image_config_both_empty_is_none() {
        // SPEC-IMG-005: both empty → no imageConfig.
        assert_eq!(ImageConfig::new("", "").unwrap(), None);
    }

    #[test]
    fn image_config_valid_values() {
        let cfg = ImageConfig::new("16:9", "2K").unwrap().unwrap();
        assert_eq!(cfg.aspect.as_deref(), Some("16:9"));
        assert_eq!(cfg.size.as_deref(), Some("2K"));
    }

    #[test]
    fn image_config_partial_valid() {
        let cfg = ImageConfig::new("1:1", "").unwrap().unwrap();
        assert_eq!(cfg.aspect.as_deref(), Some("1:1"));
        assert_eq!(cfg.size, None);
    }

    #[test]
    fn image_config_invalid_aspect_is_usage_error() {
        // SPEC-IMG-005 verbatim error string + exit 2.
        let err = ImageConfig::new("2:1", "").unwrap_err();
        assert_eq!(err.code, exit::USAGE);
        assert_eq!(
            err.message,
            "invalid aspect ratio \"2:1\"\n\nValid values: 1:1, 1:4, 1:8, 2:3, 3:2, 3:4, 4:1, 4:3, 4:5, 5:4, 8:1, 9:16, 16:9, 21:9"
        );
    }

    #[test]
    fn image_config_invalid_resolution_is_usage_error() {
        // SPEC-IMG-005 verbatim error string + exit 2. Lowercase 1k is rejected (SPEC-IMG-002).
        let err = ImageConfig::new("", "1k").unwrap_err();
        assert_eq!(err.code, exit::USAGE);
        assert_eq!(
            err.message,
            "invalid resolution \"1k\"\n\nValid values: 512, 1K, 2K, 4K"
        );
    }

    #[test]
    fn image_config_512_passes_global_gate() {
        // SPEC-IMG-007: 512 passes the GLOBAL gate here; per-model rejection is a provider concern.
        let cfg = ImageConfig::new("", "512").unwrap().unwrap();
        assert_eq!(cfg.size.as_deref(), Some("512"));
    }

    #[tokio::test]
    async fn provider_round_trips_raw_quality() {
        // SPEC-PROVIDER-005: the trait carries the RAW quality value untouched.
        let mock = MockProvider::new();
        let req = GenerateRequest {
            prompt: "a cat".to_string(),
            model: "mock-model".to_string(),
            quality: Some("high".to_string()),
            ..Default::default()
        };
        let images = mock.generate(&req).await.unwrap();
        assert_eq!(images.len(), 1);
        assert_eq!(images[0].mime, "image/png");
        assert_eq!(*mock.seen_quality.lock().unwrap(), Some("high".to_string()));
    }

    #[tokio::test]
    async fn provider_name_and_list_models() {
        let mock = MockProvider::new();
        assert_eq!(mock.name(), "mock");
        let models = mock.list_models().await.unwrap();
        assert_eq!(
            models,
            vec![ModelInfo {
                id: "mock-model".to_string()
            }]
        );
    }

    #[tokio::test]
    async fn provider_edit_path_carries_input_image() {
        // input_image = Some folds the edit/restore path into generate().
        let mock = MockProvider::new();
        let req = GenerateRequest {
            prompt: "remove background".to_string(),
            model: "mock-model".to_string(),
            input_image: Some(InputImage {
                bytes: vec![9, 9, 9],
                mime: "image/jpeg".to_string(),
            }),
            ..Default::default()
        };
        let images = mock.generate(&req).await.unwrap();
        assert_eq!(images.len(), 1);
    }
}
