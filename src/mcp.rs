//! MCP (Model Context Protocol) server (Issue 4.4, SPEC §11 SPEC-MCP-001..013).
//!
//! Exposes naba's image-generation pipeline as an MCP stdio server via the `rmcp` crate
//! (v2.2, features `server` + `transport-io`). The server registers exactly **8 tools**
//! (`generate_image`, `edit_image`, `restore_image`, `generate_icon`, `generate_pattern`,
//! `generate_story`, `generate_diagram`, `list_images`) plus the `file:///{path}` resource
//! template and the `skill://naba/<rel>` skill-resource surface (see below), and drives the
//! SAME provider/selector/output pipeline the CLI uses ([`crate::provider`] +
//! [`crate::prompt`] + [`crate::output`]) — no generation logic is reimplemented here.
//!
//! # Skills as MCP resources — lazy loading (SPEC-MCP-014/015)
//!
//! `resources/list` ([`list_resources`](NabaMcpServer::list_resources)) enumerates the
//! embedded skill tree (`skills/<name>/…`, via [`crate::embed`]) as concrete MCP resources —
//! one `skill://<name>/<rel>` URI per file (`SKILL.md`, `commands/*.md`, `README.md`) plus a
//! compact `skill://<name>` index resource. Listing carries **URIs/paths only** (no file
//! bodies), so a client discovers skills cheaply and fetches full instruction content ON
//! DEMAND via `resources/read` of a `skill://<name>/<rel>` URI — the lazy-loading pattern.
//! Reads are served from the same [`embed::read_skill_file`] / [`embed::skill_files`]
//! accessors the CLI skill commands use; no content is duplicated.
//!
//! # Reserved / slash-matching resource read (SPEC-MCP-012)
//!
//! Go's mcp-go registered `file:///{path}` with RFC 6570 *simple* expansion, whose regex
//! `[^/]` never matches an absolute path (slashes), so `resources/read` returned
//! `resource not found`. This port handles [`read_resource`](NabaMcpServer::read_resource)
//! DIRECTLY — it strips the `file://` prefix and reads whatever path is given, so absolute
//! slash paths read correctly. This is the reserved-expansion fix; the 3 previously-xfail
//! `resources/read` parity cases become live and pass.
//!
//! # Output-dir asymmetry (SPEC-CFGSCHEMA-005 / SPEC-MCP-013)
//!
//! MCP tools write via the **MCP** output-dir resolution (`NABA_OUTPUT_DIR` env > config
//! `default_output_dir` > XDG default `~/.local/share/naba/images`), NOT the CLI `-o`/CWD
//! path. All errors surface as tool-level error results (`isError: true`), never a process
//! exit.

use std::path::Path;
use std::sync::Arc;

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use serde_json::{json, Map, Value};

use rmcp::model::{
    CallToolRequestParams, CallToolResult, ContentBlock, Implementation, InitializeResult,
    JsonObject, ListResourceTemplatesResult, ListResourcesResult, ListToolsResult,
    PaginatedRequestParams, ReadResourceRequestParams, ReadResourceResult, Resource,
    ResourceContents, ResourceTemplate, ServerCapabilities, ServerInfo, Tool,
};
use rmcp::service::RequestContext;
use rmcp::{serve_server, ErrorData as McpError, RoleServer, ServerHandler};

use crate::config::Config;
use crate::embed;
use crate::error::{AppError, AppResult};
use crate::output;
use crate::prompt;
use crate::provider::{
    self, build_provider, gemini, resolve_selection, EnvKeys, GenerateRequest, ImageConfig,
    InputImage, Selection, SelectionInputs, VALID_ASPECT_RATIOS, VALID_IMAGE_SIZES,
};
use crate::version;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Launch the MCP server on stdio and run until the client disconnects (SPEC-MCP-001).
/// Wired from the `naba mcp` command.
pub async fn serve() -> AppResult<()> {
    let running = serve_server(NabaMcpServer, rmcp::transport::stdio())
        .await
        .map_err(|e| AppError::general(format!("mcp: failed to start server: {e}")))?;
    running
        .waiting()
        .await
        .map_err(|e| AppError::general(format!("mcp: server error: {e}")))?;
    Ok(())
}

/// The MCP server handler. Stateless — every tool call re-resolves config/provider (matching
/// Go, where each handler calls `resolveClient`), so runtime env/config changes are honored.
#[derive(Clone)]
pub struct NabaMcpServer;

impl ServerHandler for NabaMcpServer {
    fn get_info(&self) -> ServerInfo {
        let capabilities = ServerCapabilities::builder()
            .enable_tools()
            .enable_resources()
            .build();
        // SPEC-MCP-001: identity `naba` + build version; tool + resource capabilities.
        InitializeResult::new(capabilities)
            .with_server_info(Implementation::new("naba", version::VERSION))
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        Ok(ListToolsResult::with_all_items(tools()))
    }

    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, McpError> {
        // SPEC-MCP-012: the single `file:///{path}` template with pinned metadata.
        let tmpl = ResourceTemplate::new("file:///{path}", "Generated image file")
            .with_description("Access a generated image by its file path")
            .with_mime_type("image/*");
        Ok(ListResourceTemplatesResult::with_all_items(vec![tmpl]))
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        // SPEC-MCP-014: enumerate the embedded skill tree as cheap (URI-only) resources.
        Ok(ListResourcesResult::with_all_items(skill_resources()))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        let uri = request.uri;
        // SPEC-MCP-015: `skill://<name>/<rel>` serves embedded skill content on demand;
        // `skill://<name>` serves a compact markdown index of that skill's files.
        if let Some(rest) = uri.strip_prefix("skill://") {
            return read_skill_resource(&uri, rest);
        }
        // SPEC-MCP-012 (reserved/slash-matching fix): strip `file://` and read the raw path.
        let path = uri.strip_prefix("file://").unwrap_or(&uri);
        let data = std::fs::read(path)
            .map_err(|e| McpError::internal_error(format!("read image: {e}"), None))?;
        let mime = mime_from_ext(path);
        let encoded = BASE64.encode(&data);
        Ok(ReadResourceResult::new(vec![
            ResourceContents::BlobResourceContents {
                uri,
                mime_type: Some(mime.to_string()),
                blob: encoded,
                meta: None,
            },
        ]))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let name = request.name.as_ref();
        let args = request.arguments.unwrap_or_default();
        let result = match name {
            "generate_image" => handle_generate_image(&args).await,
            "edit_image" => handle_edit_image(&args).await,
            "restore_image" => handle_restore_image(&args).await,
            "generate_icon" => handle_generate_icon(&args).await,
            "generate_pattern" => handle_generate_pattern(&args).await,
            "generate_story" => handle_generate_story(&args).await,
            "generate_diagram" => handle_generate_diagram(&args).await,
            "list_images" => handle_list_images(&args),
            other => {
                return Err(McpError::invalid_params(
                    format!("unknown tool: {other}"),
                    None,
                ))
            }
        };
        // Every tool maps its Result into a CallToolResult: Ok(content) -> success, Err(msg) ->
        // tool-level error result (isError:true), NOT a process exit (SPEC-MCP-013).
        Ok(match result {
            Ok(content) => CallToolResult::success(content),
            Err(msg) => CallToolResult::error(vec![ContentBlock::text(msg)]),
        })
    }
}

// ---------------------------------------------------------------------------
// Tool handlers — each returns Ok(content-blocks) or Err(tool-error message).
// ---------------------------------------------------------------------------

/// SPEC-MCP-004: text-to-image, optional style/variations, `count` 1..8.
async fn handle_generate_image(args: &Map<String, Value>) -> Result<Vec<ContentBlock>, String> {
    let prompt = require_str(args, "prompt").ok_or("missing required parameter: prompt")?;
    let style = get_str(args, "style", "");
    let variations = get_str_slice(args, "variations");
    let count = get_i64(args, "count", 1);
    if !(1..=8).contains(&count) {
        return Err("count must be between 1 and 8".to_string());
    }

    let image_config = image_config_from(args)?;
    let (selection, cfg) = resolve_selection_mcp(&get_str(args, "quality", ""))?;
    let provider = build_provider(&selection);
    let out_dir = cfg.resolve_output_dir();
    let enriched = prompt::enrich_generate_prompt(&prompt, &style, &variations);

    let mut images: Vec<(String, String)> = Vec::new();
    for _ in 0..count {
        let req = gen_request(&enriched, &selection, image_config.clone(), None);
        let generated = provider.generate(&req).await.map_err(|e| e.message)?;
        for img in &generated {
            let idx = images.len();
            let path = write_image(&out_dir, "generate", img, idx)?;
            images.push((path, img.mime.clone()));
        }
    }
    Ok(image_content(&images))
}

/// SPEC-MCP-005: edit an existing image (prompt + file both required).
async fn handle_edit_image(args: &Map<String, Value>) -> Result<Vec<ContentBlock>, String> {
    let prompt = require_str(args, "prompt").ok_or("missing required parameter: prompt")?;
    let file = require_str(args, "file").ok_or("missing required parameter: file")?;
    let image_config = image_config_from(args)?;
    let (selection, cfg) = resolve_selection_mcp(&get_str(args, "quality", ""))?;
    let enriched = prompt::enrich_edit_prompt(&prompt);
    generate_with_image(&enriched, &file, "edit", &selection, image_config, &cfg).await
}

/// SPEC-MCP-006: restore/enhance an existing image (file required, prompt optional).
async fn handle_restore_image(args: &Map<String, Value>) -> Result<Vec<ContentBlock>, String> {
    let file = require_str(args, "file").ok_or("missing required parameter: file")?;
    let image_config = image_config_from(args)?;
    let (selection, cfg) = resolve_selection_mcp(&get_str(args, "quality", ""))?;
    let raw_prompt = get_str(args, "prompt", "");
    let enriched = prompt::enrich_restore_prompt(&raw_prompt);
    generate_with_image(&enriched, &file, "restore", &selection, image_config, &cfg).await
}

/// SPEC-MCP-007: app icons in one or more sizes (quality only, no imageConfig).
async fn handle_generate_icon(args: &Map<String, Value>) -> Result<Vec<ContentBlock>, String> {
    let prompt = require_str(args, "prompt").ok_or("missing required parameter: prompt")?;
    let style = get_str(args, "style", "modern");
    let background = get_str(args, "background", "transparent");
    let corners = get_str(args, "corners", "rounded");
    let sizes = get_i64_slice(args, "sizes", &[256]);

    // icon takes the model-selecting `quality` param but NO imageConfig (--size is canvas px).
    let (selection, cfg) = resolve_selection_mcp(&get_str(args, "quality", ""))?;
    let provider = build_provider(&selection);
    let out_dir = cfg.resolve_output_dir();

    let mut images: Vec<(String, String)> = Vec::new();
    for size in sizes {
        let enriched = prompt::enrich_icon_prompt(&prompt, &style, size, &background, &corners);
        let req = gen_request(&enriched, &selection, None, None);
        let generated = provider.generate(&req).await.map_err(|e| e.message)?;
        for img in &generated {
            let idx = images.len();
            let path = write_image(&out_dir, "icon", img, idx)?;
            images.push((path, img.mime.clone()));
        }
    }
    Ok(image_content(&images))
}

/// SPEC-MCP-008: seamless pattern (single generate call).
async fn handle_generate_pattern(args: &Map<String, Value>) -> Result<Vec<ContentBlock>, String> {
    let prompt = require_str(args, "prompt").ok_or("missing required parameter: prompt")?;
    let style = get_str(args, "style", "abstract");
    let colors = get_str(args, "colors", "colorful");
    let density = get_str(args, "density", "medium");
    let size = get_str(args, "size", "256x256");
    let repeat = get_str(args, "repeat", "tile");
    let image_config = image_config_from(args)?;
    let (selection, cfg) = resolve_selection_mcp(&get_str(args, "quality", ""))?;
    let enriched =
        prompt::enrich_pattern_prompt(&prompt, &style, &colors, &density, &size, &repeat);
    generate_single(&enriched, "pattern", &selection, image_config, &cfg).await
}

/// SPEC-MCP-009: visual story, `steps` 2..8 frames.
async fn handle_generate_story(args: &Map<String, Value>) -> Result<Vec<ContentBlock>, String> {
    let prompt = require_str(args, "prompt").ok_or("missing required parameter: prompt")?;
    let steps = get_i64(args, "steps", 4);
    let style = get_str(args, "style", "consistent");
    let transition = get_str(args, "transition", "smooth");
    if !(2..=8).contains(&steps) {
        return Err("steps must be between 2 and 8".to_string());
    }
    let image_config = image_config_from(args)?;
    let (selection, cfg) = resolve_selection_mcp(&get_str(args, "quality", ""))?;
    let provider = build_provider(&selection);
    let out_dir = cfg.resolve_output_dir();

    let mut images: Vec<(String, String)> = Vec::new();
    for step in 1..=steps {
        let enriched = prompt::enrich_story_prompt(&prompt, step, steps, &style, &transition);
        let req = gen_request(&enriched, &selection, image_config.clone(), None);
        let generated = provider.generate(&req).await.map_err(|e| e.message)?;
        for img in &generated {
            let idx = images.len();
            let path = write_image(&out_dir, "story", img, idx)?;
            images.push((path, img.mime.clone()));
        }
    }
    Ok(image_content(&images))
}

/// SPEC-MCP-010: technical diagram (single generate call).
async fn handle_generate_diagram(args: &Map<String, Value>) -> Result<Vec<ContentBlock>, String> {
    let prompt = require_str(args, "prompt").ok_or("missing required parameter: prompt")?;
    let diagram_type = get_str(args, "type", "flowchart");
    let style = get_str(args, "style", "professional");
    let layout = get_str(args, "layout", "hierarchical");
    let complexity = get_str(args, "complexity", "detailed");
    let colors = get_str(args, "colors", "accent");
    let image_config = image_config_from(args)?;
    let (selection, cfg) = resolve_selection_mcp(&get_str(args, "quality", ""))?;
    let enriched = prompt::enrich_diagram_prompt(
        &prompt,
        &diagram_type,
        &style,
        &layout,
        &complexity,
        &colors,
    );
    generate_single(&enriched, "diagram", &selection, image_config, &cfg).await
}

/// SPEC-MCP-011: list recently generated `naba-*` images (MCP-only, no CLI counterpart).
fn handle_list_images(args: &Map<String, Value>) -> Result<Vec<ContentBlock>, String> {
    let cfg = Config::load().unwrap_or_default();
    let out_dir = cfg.resolve_output_dir();
    if out_dir.is_empty() {
        return Err("no output directory configured".to_string());
    }
    let mut limit = get_i64(args, "limit", 20);
    if limit < 1 {
        limit = 20;
    }

    // Missing directory is NOT an error result — it is a text note (matches Go).
    if !Path::new(&out_dir).exists() {
        return Ok(vec![ContentBlock::text(
            "No images found (directory does not exist)",
        )]);
    }

    let files = list_image_files(&out_dir, limit as usize);
    if files.is_empty() {
        return Ok(vec![ContentBlock::text("No images found")]);
    }
    Ok(files.into_iter().map(ContentBlock::text).collect())
}

// ---------------------------------------------------------------------------
// Shared generation helpers (reuse the CLI pipeline — no generation logic here)
// ---------------------------------------------------------------------------

/// Build a [`GenerateRequest`] from the resolved selection (model already resolved by the
/// selector; `quality` carried raw for OpenRouter's native param — SPEC-PROVIDER-005).
fn gen_request(
    prompt: &str,
    selection: &Selection,
    image_config: Option<ImageConfig>,
    input_image: Option<InputImage>,
) -> GenerateRequest {
    GenerateRequest {
        prompt: prompt.to_string(),
        model: selection.model.clone(),
        image_config,
        input_image,
        quality: selection.quality.clone(),
    }
}

/// The single-image text-to-image flow (pattern/diagram): one provider call, one written image.
async fn generate_single(
    enriched: &str,
    command: &str,
    selection: &Selection,
    image_config: Option<ImageConfig>,
    cfg: &Config,
) -> Result<Vec<ContentBlock>, String> {
    let provider = build_provider(selection);
    let out_dir = cfg.resolve_output_dir();
    let req = gen_request(enriched, selection, image_config, None);
    let generated = provider.generate(&req).await.map_err(|e| e.message)?;
    if generated.is_empty() {
        return Err("no images in response".to_string());
    }
    let mut images: Vec<(String, String)> = Vec::new();
    for (idx, img) in generated.iter().enumerate() {
        let path = write_image(&out_dir, command, img, idx)?;
        images.push((path, img.mime.clone()));
    }
    Ok(image_content(&images))
}

/// The image-input flow (edit/restore): validate the file, read it, single provider call.
async fn generate_with_image(
    enriched: &str,
    file: &str,
    command: &str,
    selection: &Selection,
    image_config: Option<ImageConfig>,
    cfg: &Config,
) -> Result<Vec<ContentBlock>, String> {
    // SPEC-MCP-005: a nonexistent input file -> `file not found: <path>`.
    if !Path::new(file).exists() {
        return Err(format!("file not found: {file}"));
    }
    let input_image = gemini::read_image_file(file).map_err(|e| e.message)?;
    let provider = build_provider(selection);
    let out_dir = cfg.resolve_output_dir();
    let req = gen_request(enriched, selection, image_config, Some(input_image));
    let generated = provider.generate(&req).await.map_err(|e| e.message)?;
    if generated.is_empty() {
        return Err("no images in response".to_string());
    }
    let mut images: Vec<(String, String)> = Vec::new();
    for (idx, img) in generated.iter().enumerate() {
        let path = write_image(&out_dir, command, img, idx)?;
        images.push((path, img.mime.clone()));
    }
    Ok(image_content(&images))
}

/// Write one image to the MCP output dir (SPEC-CFGSCHEMA-005) and return the absolute path.
fn write_image(
    out_dir: &str,
    command: &str,
    img: &provider::GeneratedImage,
    index: usize,
) -> Result<String, String> {
    let out_path = output::output_path(out_dir, command, &img.mime);
    let w = output::write_image_result(&img.bytes, &img.mime, &out_path, command, index)
        .map_err(|e| format!("write image: {e}"))?;
    Ok(w.path)
}

/// Build the `imageResult` content (SPEC-MCP-013): per image a text path, a `Format: <mime>`
/// note, and a `file://` resource link. One entry per image; the link label is
/// `Generated image` for a lone image, `Generated image N` when there are several.
fn image_content(images: &[(String, String)]) -> Vec<ContentBlock> {
    let mut content = Vec::with_capacity(images.len() * 3);
    let multi = images.len() > 1;
    for (i, (path, mime)) in images.iter().enumerate() {
        let base = Path::new(path)
            .file_name()
            .map(|f| f.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.clone());
        let label = if multi {
            format!("Generated image {}", i + 1)
        } else {
            "Generated image".to_string()
        };
        content.push(ContentBlock::text(path.clone()));
        content.push(ContentBlock::text(format!("Format: {mime}")));
        content.push(ContentBlock::resource_link(
            Resource::new(format!("file://{path}"), base)
                .with_description(label)
                .with_mime_type(mime.clone()),
        ));
    }
    content
}

/// Resolve provider/model/key for an MCP tool via the shared selector (SPEC-PROVIDER-007),
/// feeding the tool's `quality` param as the model tier. On an empty resolved key, raise the
/// SPEC-MCP-013 missing-key message (single-line MCP form). Returns the loaded [`Config`] too
/// (for output-dir resolution).
fn resolve_selection_mcp(quality: &str) -> Result<(Selection, Config), String> {
    let cfg = Config::load().unwrap_or_default();
    let cfg_defaults = cfg.to_config_defaults().map_err(|e| e.message)?;
    let env_keys = EnvKeys::from_resolved(
        provider::registry::names()
            .into_iter()
            .map(|name| (name.to_string(), cfg.resolve_api_key_for(name))),
    );
    let inputs = SelectionInputs {
        provider: None,
        model: None,
        quality: Some(quality.to_string()),
    };
    let selection = resolve_selection(&inputs, &cfg_defaults, &env_keys).map_err(|e| e.message)?;
    if selection.api_key.is_empty() {
        return Err(mcp_missing_key_message(&selection.provider));
    }
    Ok((selection, cfg))
}

/// The SPEC-MCP-013 missing-key message (single-line MCP form). [DIVERGENCE] names the
/// selected provider's key for openrouter.
fn mcp_missing_key_message(provider: &str) -> String {
    let key = if provider == "openrouter" {
        "OPENROUTER_API_KEY"
    } else {
        "GEMINI_API_KEY"
    };
    format!("{key} not set. Set it with: export {key}=<your-key> or run: naba config set api_key <your-key>")
}

/// Build the validated [`ImageConfig`] from a tool's `aspect`/`resolution` args (MCP takes no
/// config default — matches Go's `NewImageConfig(req aspect, req resolution)`). Invalid enum
/// values surface as the tool-error message.
fn image_config_from(args: &Map<String, Value>) -> Result<Option<ImageConfig>, String> {
    let aspect = get_str(args, "aspect", "");
    let resolution = get_str(args, "resolution", "");
    ImageConfig::new(&aspect, &resolution).map_err(|e| e.message)
}

/// The `naba-*` image-file filter + newest-first sort + cap (SPEC-MCP-011). Extracted for unit
/// testing; the handler wraps it with the empty / missing-dir / no-output-dir messaging.
fn list_image_files(out_dir: &str, limit: usize) -> Vec<String> {
    let entries = match std::fs::read_dir(out_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    let mut files: Vec<(String, std::time::SystemTime)> = Vec::new();
    for entry in entries.flatten() {
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        if meta.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with("naba-") {
            continue;
        }
        let ext = Path::new(&name)
            .extension()
            .map(|e| e.to_string_lossy().to_ascii_lowercase())
            .unwrap_or_default();
        if !matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "gif" | "webp") {
            continue;
        }
        let mtime = meta.modified().unwrap_or(std::time::UNIX_EPOCH);
        let path = Path::new(out_dir)
            .join(&name)
            .to_string_lossy()
            .into_owned();
        files.push((path, mtime));
    }
    // Newest first by modtime.
    files.sort_by_key(|f| std::cmp::Reverse(f.1));
    files.truncate(limit);
    files.into_iter().map(|(p, _)| p).collect()
}

/// MIME type by file extension for `resources/read` (SPEC-MCP-012); unknown ->
/// `application/octet-stream`.
fn mime_from_ext(path: &str) -> &'static str {
    let ext = Path::new(path)
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        _ => "application/octet-stream",
    }
}

// ---------------------------------------------------------------------------
// Skills-as-resources: cheap listing + on-demand read (SPEC-MCP-014/015).
// ---------------------------------------------------------------------------

/// Enumerate the embedded skill tree as MCP resources (SPEC-MCP-014). Per embedded skill
/// `<name>`, emit a compact `skill://<name>` index resource followed by one
/// `skill://<name>/<rel>` resource per file (from [`embed::skill_files`], sorted). Only
/// URIs/metadata — never file bodies — so `resources/list` stays cheap (lazy loading).
fn skill_resources() -> Vec<Resource> {
    let mut out = Vec::new();
    for name in embed::skill_names() {
        out.push(
            Resource::new(format!("skill://{name}"), format!("{name} skills index"))
                .with_description(format!(
                    "Index of embedded `{name}` skill files (read on demand)"
                ))
                .with_mime_type("text/markdown"),
        );
        for rel in embed::skill_files(&name) {
            out.push(
                Resource::new(format!("skill://{name}/{rel}"), format!("{name}/{rel}"))
                    .with_description(format!("Embedded naba skill file `{rel}`"))
                    .with_mime_type(skill_mime(&rel)),
            );
        }
    }
    out
}

/// Serve a `skill://…` read (SPEC-MCP-015). `skill://<name>/<rel>` returns the embedded file
/// as `TextResourceContents` (MIME by extension); `skill://<name>` returns a generated
/// markdown index of that skill's files. Unknown skill/file → `resource not found: <uri>`.
fn read_skill_resource(uri: &str, rest: &str) -> Result<ReadResourceResult, McpError> {
    let not_found = || McpError::invalid_params(format!("resource not found: {uri}"), None);
    // Tolerate a trailing slash (some clients normalize `skill://naba` to `skill://naba/`):
    // an empty `<rel>` collapses to the skill index rather than a missing-file error.
    let rest = rest.strip_suffix('/').unwrap_or(rest);
    let contents = match rest.split_once('/') {
        Some((name, rel)) => {
            let bytes = embed::read_skill_file(name, rel).ok_or_else(not_found)?;
            ResourceContents::TextResourceContents {
                uri: uri.to_string(),
                mime_type: Some(skill_mime(rel).to_string()),
                text: String::from_utf8_lossy(bytes).into_owned(),
                meta: None,
            }
        }
        None => {
            let files = embed::skill_files(rest);
            if files.is_empty() {
                return Err(not_found());
            }
            ResourceContents::TextResourceContents {
                uri: uri.to_string(),
                mime_type: Some("text/markdown".to_string()),
                text: skill_index_markdown(rest, &files),
                meta: None,
            }
        }
    };
    Ok(ReadResourceResult::new(vec![contents]))
}

/// MIME for an embedded skill file by extension: `.md` → `text/markdown`, else `text/plain`.
fn skill_mime(rel: &str) -> &'static str {
    match Path::new(rel).extension().and_then(|e| e.to_str()) {
        Some(e) if e.eq_ignore_ascii_case("md") => "text/markdown",
        _ => "text/plain",
    }
}

/// The compact `skill://<name>` index body: a markdown bullet list of each file's read URI.
fn skill_index_markdown(name: &str, files: &[String]) -> String {
    let mut s = format!("# naba skill: {name}\n\nAvailable resources (read on demand):\n\n");
    for rel in files {
        s.push_str(&format!("- `skill://{name}/{rel}`\n"));
    }
    s
}

// ---------------------------------------------------------------------------
// Argument accessors (mirror mcp-go's req.GetString / GetInt / GetStringSlice).
// ---------------------------------------------------------------------------

/// A present string param (any string value, including empty), else `None` — mirrors mcp-go's
/// `RequireString` (missing/non-string -> error; the handler maps `None` to the SPEC message).
fn require_str(args: &Map<String, Value>, key: &str) -> Option<String> {
    args.get(key).and_then(Value::as_str).map(str::to_string)
}

/// A string param with a default (mcp-go `GetString`).
fn get_str(args: &Map<String, Value>, key: &str, default: &str) -> String {
    args.get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| default.to_string())
}

/// An integer param with a default (mcp-go `GetInt`). Accepts JSON ints and floats.
fn get_i64(args: &Map<String, Value>, key: &str, default: i64) -> i64 {
    match args.get(key) {
        Some(Value::Number(n)) => n
            .as_i64()
            .or_else(|| n.as_f64().map(|f| f as i64))
            .unwrap_or(default),
        _ => default,
    }
}

/// A string-array param (mcp-go `GetStringSlice`); non-string items are skipped.
fn get_str_slice(args: &Map<String, Value>, key: &str) -> Vec<String> {
    match args.get(key).and_then(Value::as_array) {
        Some(arr) => arr
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect(),
        None => Vec::new(),
    }
}

/// An integer-array param with a default (mcp-go `GetIntSlice`). Accepts JSON ints/floats.
fn get_i64_slice(args: &Map<String, Value>, key: &str, default: &[i64]) -> Vec<i64> {
    match args.get(key).and_then(Value::as_array) {
        Some(arr) if !arr.is_empty() => arr
            .iter()
            .filter_map(|v| v.as_i64().or_else(|| v.as_f64().map(|f| f as i64)))
            .collect(),
        _ => default.to_vec(),
    }
}

// ---------------------------------------------------------------------------
// Tool schema assembly (SPEC-MCP-002..011). Schemas are hand-built serde_json to match the
// Go-captured golden verbatim (types, enums, defaults, required order, descriptions).
// ---------------------------------------------------------------------------

/// The 8 pinned MCP tools with their exact input schemas.
fn tools() -> Vec<Tool> {
    vec![
        generate_image_tool(),
        edit_image_tool(),
        restore_image_tool(),
        generate_icon_tool(),
        generate_pattern_tool(),
        generate_story_tool(),
        generate_diagram_tool(),
        list_images_tool(),
    ]
}

/// Wrap a properties object + required list into a JSON-Schema object (`type: object`).
fn schema(properties: Value, required: &[&str]) -> Arc<JsonObject> {
    let mut m = Map::new();
    m.insert("type".into(), json!("object"));
    m.insert("properties".into(), properties);
    m.insert("required".into(), json!(required));
    Arc::new(m)
}

/// Insert the shared imageConfig params (`aspect`/`resolution`/`quality`) into a properties map
/// (SPEC-MCP-003).
fn insert_image_config(props: &mut Map<String, Value>) {
    props.insert(
        "aspect".into(),
        json!({
            "type": "string",
            "description": "Aspect ratio (generationConfig.imageConfig.aspectRatio)",
            "enum": VALID_ASPECT_RATIOS,
        }),
    );
    props.insert(
        "resolution".into(),
        json!({
            "type": "string",
            "description": "Image resolution (generationConfig.imageConfig.imageSize)",
            "enum": VALID_IMAGE_SIZES,
        }),
    );
    props.insert("quality".into(), quality_prop());
}

/// The `quality` (model alias) param (SPEC-MCP-003). Description is [DIVERGENCE] under
/// multi-provider; the parity golden normalizes it to `<QUALITY_DESC>`.
fn quality_prop() -> Value {
    json!({
        "type": "string",
        "description": "Quality tier: fast (gemini-3.1-flash-image) or high (gemini-3-pro-image)",
        "enum": ["fast", "high"],
    })
}

fn generate_image_tool() -> Tool {
    let mut props = Map::new();
    props.insert(
        "prompt".into(),
        json!({"type": "string", "description": "The text prompt describing the image to generate"}),
    );
    props.insert(
        "style".into(),
        json!({
            "type": "string",
            "description": "Art style",
            "enum": ["photorealistic", "watercolor", "oil-painting", "sketch", "pixel-art",
                     "anime", "vintage", "modern", "abstract", "minimalist"],
        }),
    );
    props.insert(
        "variations".into(),
        json!({
            "type": "array",
            "description": "Variation types to apply",
            "items": {
                "type": "string",
                "enum": ["lighting", "angle", "color-palette", "composition", "mood",
                         "season", "time-of-day"],
            },
        }),
    );
    props.insert(
        "count".into(),
        json!({
            "type": "number",
            "description": "Number of variations to generate (1-8)",
            "default": 1,
            "minimum": 1,
            "maximum": 8,
        }),
    );
    props.insert(
        "seed".into(),
        json!({"type": "number", "description": "Seed for reproducible output"}),
    );
    insert_image_config(&mut props);
    Tool::new(
        "generate_image",
        "Generate an image from a text prompt",
        schema(Value::Object(props), &["prompt"]),
    )
}

fn edit_image_tool() -> Tool {
    let mut props = Map::new();
    props.insert(
        "prompt".into(),
        json!({"type": "string", "description": "The text prompt describing the edits to make"}),
    );
    props.insert(
        "file".into(),
        json!({"type": "string", "description": "The file path of the input image to edit"}),
    );
    insert_image_config(&mut props);
    Tool::new(
        "edit_image",
        "Edit an existing image based on a text prompt",
        schema(Value::Object(props), &["prompt", "file"]),
    )
}

fn restore_image_tool() -> Tool {
    let mut props = Map::new();
    props.insert(
        "file".into(),
        json!({"type": "string", "description": "The file path of the input image to restore"}),
    );
    props.insert(
        "prompt".into(),
        json!({"type": "string", "description": "The text prompt describing the restoration to perform"}),
    );
    insert_image_config(&mut props);
    Tool::new(
        "restore_image",
        "Restore or enhance an existing image",
        schema(Value::Object(props), &["file"]),
    )
}

fn generate_icon_tool() -> Tool {
    let mut props = Map::new();
    props.insert(
        "prompt".into(),
        json!({"type": "string", "description": "Description of the icon to generate"}),
    );
    props.insert(
        "sizes".into(),
        json!({
            "type": "array",
            "description": "Icon sizes in pixels (e.g. 64, 128, 256, 512)",
            "items": {"type": "number", "minimum": 16, "maximum": 1024},
        }),
    );
    props.insert(
        "style".into(),
        json!({
            "type": "string",
            "description": "Visual style of the icon",
            "default": "modern",
            "enum": ["flat", "skeuomorphic", "minimal", "modern"],
        }),
    );
    props.insert(
        "background".into(),
        json!({"type": "string", "description": "Background type", "default": "transparent"}),
    );
    props.insert(
        "corners".into(),
        json!({
            "type": "string",
            "description": "Corner style",
            "default": "rounded",
            "enum": ["rounded", "sharp"],
        }),
    );
    props.insert(
        "format".into(),
        json!({
            "type": "string",
            "description": "Output format",
            "default": "png",
            "enum": ["png", "jpeg"],
        }),
    );
    // icon: quality only (no aspect/resolution).
    props.insert("quality".into(), quality_prop());
    Tool::new(
        "generate_icon",
        "Generate app icons in multiple sizes",
        schema(Value::Object(props), &["prompt"]),
    )
}

fn generate_pattern_tool() -> Tool {
    let mut props = Map::new();
    props.insert(
        "prompt".into(),
        json!({"type": "string", "description": "Description of the pattern to generate"}),
    );
    props.insert(
        "style".into(),
        json!({
            "type": "string",
            "description": "Pattern style",
            "default": "abstract",
            "enum": ["geometric", "organic", "abstract", "floral", "tech"],
        }),
    );
    props.insert(
        "colors".into(),
        json!({
            "type": "string",
            "description": "Color scheme",
            "default": "colorful",
            "enum": ["mono", "duotone", "colorful"],
        }),
    );
    props.insert(
        "density".into(),
        json!({
            "type": "string",
            "description": "Element density",
            "default": "medium",
            "enum": ["sparse", "medium", "dense"],
        }),
    );
    props.insert(
        "size".into(),
        json!({
            "type": "string",
            "description": "Pattern tile size (e.g. 256x256, 512x512)",
            "default": "256x256",
        }),
    );
    props.insert(
        "repeat".into(),
        json!({
            "type": "string",
            "description": "Tiling method",
            "default": "tile",
            "enum": ["tile", "mirror"],
        }),
    );
    insert_image_config(&mut props);
    Tool::new(
        "generate_pattern",
        "Generate seamless patterns and textures",
        schema(Value::Object(props), &["prompt"]),
    )
}

fn generate_story_tool() -> Tool {
    let mut props = Map::new();
    props.insert(
        "prompt".into(),
        json!({"type": "string", "description": "Description of the story to visualize"}),
    );
    props.insert(
        "steps".into(),
        json!({
            "type": "number",
            "description": "Number of sequential images (2-8)",
            "default": 4,
            "minimum": 2,
            "maximum": 8,
        }),
    );
    props.insert(
        "style".into(),
        json!({
            "type": "string",
            "description": "Visual consistency across frames",
            "default": "consistent",
            "enum": ["consistent", "evolving"],
        }),
    );
    props.insert(
        "transition".into(),
        json!({
            "type": "string",
            "description": "Transition style between frames",
            "default": "smooth",
            "enum": ["smooth", "dramatic", "fade"],
        }),
    );
    props.insert(
        "layout".into(),
        json!({
            "type": "string",
            "description": "Output layout format",
            "default": "separate",
            "enum": ["separate", "grid", "comic"],
        }),
    );
    insert_image_config(&mut props);
    Tool::new(
        "generate_story",
        "Generate a sequence of images that tell a visual story",
        schema(Value::Object(props), &["prompt"]),
    )
}

fn generate_diagram_tool() -> Tool {
    let mut props = Map::new();
    props.insert(
        "prompt".into(),
        json!({"type": "string", "description": "Description of the diagram to generate"}),
    );
    props.insert(
        "type".into(),
        json!({
            "type": "string",
            "description": "Type of diagram",
            "default": "flowchart",
            "enum": ["flowchart", "architecture", "network", "database", "wireframe",
                     "mindmap", "sequence"],
        }),
    );
    props.insert(
        "style".into(),
        json!({
            "type": "string",
            "description": "Visual style",
            "default": "professional",
            "enum": ["professional", "clean", "hand-drawn", "technical"],
        }),
    );
    props.insert(
        "layout".into(),
        json!({
            "type": "string",
            "description": "Layout orientation",
            "default": "hierarchical",
            "enum": ["horizontal", "vertical", "hierarchical", "circular"],
        }),
    );
    props.insert(
        "complexity".into(),
        json!({
            "type": "string",
            "description": "Level of detail",
            "default": "detailed",
            "enum": ["simple", "detailed", "comprehensive"],
        }),
    );
    props.insert(
        "colors".into(),
        json!({
            "type": "string",
            "description": "Color scheme",
            "default": "accent",
            "enum": ["mono", "accent", "categorical"],
        }),
    );
    insert_image_config(&mut props);
    Tool::new(
        "generate_diagram",
        "Generate technical diagrams and flowcharts",
        schema(Value::Object(props), &["prompt"]),
    )
}

fn list_images_tool() -> Tool {
    let mut props = Map::new();
    props.insert(
        "limit".into(),
        json!({
            "type": "number",
            "description": "Maximum number of images to return",
            "default": 20,
        }),
    );
    Tool::new(
        "list_images",
        "List recently generated images in the output directory",
        schema(Value::Object(props), &[]),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exactly_eight_tools_with_pinned_names() {
        let names: Vec<String> = tools().iter().map(|t| t.name.to_string()).collect();
        assert_eq!(names.len(), 8);
        for expected in [
            "generate_image",
            "edit_image",
            "restore_image",
            "generate_icon",
            "generate_pattern",
            "generate_story",
            "generate_diagram",
            "list_images",
        ] {
            assert!(names.contains(&expected.to_string()), "missing {expected}");
        }
    }

    #[test]
    fn generate_image_schema_shape() {
        let t = generate_image_tool();
        let schema = t.input_schema.as_ref();
        let props = schema["properties"].as_object().unwrap();
        // count: number, default 1, bounds 1..8.
        let count = &props["count"];
        assert_eq!(count["type"], "number");
        assert_eq!(count["default"], 1);
        assert_eq!(count["minimum"], 1);
        assert_eq!(count["maximum"], 8);
        // required == ["prompt"].
        assert_eq!(schema["required"], json!(["prompt"]));
        // shared imageConfig present.
        assert_eq!(
            props["aspect"]["description"],
            "Aspect ratio (generationConfig.imageConfig.aspectRatio)"
        );
        assert_eq!(props["quality"]["enum"], json!(["fast", "high"]));
    }

    #[test]
    fn edit_required_order_is_prompt_then_file() {
        // Golden byte-compares the required array, so order is load-bearing.
        let t = edit_image_tool();
        assert_eq!(t.input_schema["required"], json!(["prompt", "file"]));
    }

    #[test]
    fn icon_has_quality_but_no_image_config() {
        let t = generate_icon_tool();
        let props = t.input_schema["properties"].as_object().unwrap();
        assert!(props.contains_key("quality"));
        assert!(!props.contains_key("aspect"));
        assert!(!props.contains_key("resolution"));
    }

    #[test]
    fn list_images_required_is_empty() {
        let t = list_images_tool();
        assert_eq!(t.input_schema["required"], json!([]));
    }

    #[test]
    fn list_image_files_filters_and_sorts_newest_first() {
        let dir = std::env::temp_dir().join(format!("naba-mcp-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mk = |name: &str, mtime: u64| {
            let p = dir.join(name);
            let f = std::fs::File::create(&p).unwrap();
            use std::io::Write;
            (&f).write_all(b"x").unwrap();
            // Deterministic mtime via std (no external dep) so newest-first ordering is stable.
            f.set_modified(std::time::UNIX_EPOCH + std::time::Duration::from_secs(mtime))
                .unwrap();
        };
        mk("naba-generate-1.png", 1_000_000);
        mk("naba-edit-2.jpg", 1_000_010);
        mk("naba-story-3.webp", 1_000_020);
        std::fs::write(dir.join("other.png"), b"x").unwrap(); // non-naba: ignored
        std::fs::write(dir.join("naba-note.txt"), b"x").unwrap(); // non-image: ignored

        let files = list_image_files(dir.to_str().unwrap(), 20);
        let names: Vec<String> = files
            .iter()
            .map(|p| {
                Path::new(p)
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        assert_eq!(
            names,
            vec![
                "naba-story-3.webp",
                "naba-edit-2.jpg",
                "naba-generate-1.png"
            ]
        );

        // limit caps the result.
        let capped = list_image_files(dir.to_str().unwrap(), 2);
        assert_eq!(capped.len(), 2);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn mime_from_ext_maps_known_and_unknown_ext() {
        assert_eq!(mime_from_ext("/a/b.png"), "image/png");
        assert_eq!(mime_from_ext("/a/b.JPG"), "image/jpeg");
        assert_eq!(mime_from_ext("/a/b.jpeg"), "image/jpeg");
        assert_eq!(mime_from_ext("/a/b.gif"), "image/gif");
        assert_eq!(mime_from_ext("/a/b.webp"), "image/webp");
        assert_eq!(mime_from_ext("/a/b.bin"), "application/octet-stream");
    }

    #[test]
    fn skill_resources_enumerate_files_and_index() {
        // SPEC-MCP-014: listing carries a per-skill index + one URI per embedded file.
        let resources = skill_resources();
        let uris: Vec<&str> = resources.iter().map(|r| r.uri.as_str()).collect();
        // Compact index resource for the naba skill.
        assert!(
            uris.contains(&"skill://naba"),
            "missing skill index: {uris:?}"
        );
        // One resource per embedded file, addressed by skill://naba/<rel>.
        for rel in embed::skill_files("naba") {
            let uri = format!("skill://naba/{rel}");
            assert!(uris.contains(&uri.as_str()), "missing {uri}");
        }
        // Markdown files advertise the text/markdown MIME.
        let skill_md = resources
            .iter()
            .find(|r| r.uri == "skill://naba/SKILL.md")
            .expect("SKILL.md resource");
        assert_eq!(skill_md.mime_type.as_deref(), Some("text/markdown"));
        // Listing is cheap: no file bodies are attached to the Resource entries.
    }

    #[test]
    fn read_skill_resource_returns_file_and_index() {
        // SPEC-MCP-015: a skill://naba/<rel> read returns the embedded content as text.
        let uri = "skill://naba/SKILL.md";
        let result = read_skill_resource(uri, "naba/SKILL.md").unwrap();
        match &result.contents[0] {
            ResourceContents::TextResourceContents {
                uri: u,
                mime_type,
                text,
                ..
            } => {
                assert_eq!(u, uri);
                assert_eq!(mime_type.as_deref(), Some("text/markdown"));
                let expected = embed::read_skill_file("naba", "SKILL.md").unwrap();
                assert_eq!(text.as_bytes(), expected);
            }
            other => panic!("expected text contents, got {other:?}"),
        }

        // The compact index (skill://naba) lists every file's read URI.
        let idx = read_skill_resource("skill://naba", "naba").unwrap();
        match &idx.contents[0] {
            ResourceContents::TextResourceContents { text, .. } => {
                assert!(text.contains("skill://naba/SKILL.md"), "index: {text}");
            }
            other => panic!("expected text index, got {other:?}"),
        }

        // A trailing slash (client URL normalization) still resolves to the index.
        let idx_slash = read_skill_resource("skill://naba/", "naba/").unwrap();
        match &idx_slash.contents[0] {
            ResourceContents::TextResourceContents { text, .. } => {
                assert!(text.contains("skill://naba/SKILL.md"), "index: {text}");
            }
            other => panic!("expected text index, got {other:?}"),
        }

        // Unknown file / skill -> resource-not-found error, not a panic.
        assert!(read_skill_resource("skill://naba/nope.md", "naba/nope.md").is_err());
        assert!(read_skill_resource("skill://ghost", "ghost").is_err());
    }

    #[test]
    fn skill_mime_maps_md_and_other() {
        assert_eq!(skill_mime("SKILL.md"), "text/markdown");
        assert_eq!(skill_mime("commands/edit.md"), "text/markdown");
        assert_eq!(skill_mime("data.txt"), "text/plain");
        assert_eq!(skill_mime("noext"), "text/plain");
    }

    #[test]
    fn missing_key_message_is_single_line_gemini_form() {
        let msg = mcp_missing_key_message("gemini");
        assert!(msg.starts_with("GEMINI_API_KEY not set. Set it with:"));
        assert!(msg.contains("naba config set api_key"));
        assert_eq!(
            mcp_missing_key_message("openrouter").split(' ').next(),
            Some("OPENROUTER_API_KEY")
        );
    }
}
