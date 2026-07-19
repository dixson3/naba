//! Output layer (Issue 3.3): file writing with extension reconciliation, the `Result`
//! JSON shapes (SPEC-JSON-001..005), the `doctor` envelope (SPEC-JSON-004), the OS-viewer
//! preview, and the CLI-vs-MCP output-dir asymmetry (SPEC-CFGSCHEMA-004/005).
//!
//! This module is a self-contained set of PRIMITIVES the command layer (Issue 4.x) and the
//! MCP server (4.4) call. It ports Go's `internal/output` (`writer.go`, `json.go`,
//! `preview.go`) field-for-field and behavior-for-behavior. It contains no command behavior,
//! no config-YAML parsing (3.1), and no doctor logic (4.3) — only the seams those issues wire.
//!
//! # CLI-vs-MCP output-dir asymmetry (SPEC-CFGSCHEMA-005)
//!
//! The two write paths are DELIBERATELY distinct and must stay so:
//!
//! * **CLI** image commands call [`write_image_result`] with an explicit `-o` target (a file
//!   path) or `""` to auto-name in the CURRENT WORKING DIRECTORY. The CLI path NEVER consults
//!   `NABA_OUTPUT_DIR` / config `default_output_dir` / the XDG default.
//! * **MCP** tools resolve a directory with [`mcp_output_dir`] (`NABA_OUTPUT_DIR` env > config
//!   `default_output_dir` > XDG default `~/.local/share/naba/images`), then build a filename
//!   with [`output_path`] and write with [`write_image_result`] passing that full path.
//!
//! Keeping [`resolve_output_dir`] / [`mcp_output_dir`] out of the CLI call graph is what
//! preserves the asymmetry: 4.x wires the CLI to `-o`/CWD and the MCP server to `mcp_output_dir`.

use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::Serialize;
use serde_json::{Map, Value};

/// `NABA_OUTPUT_DIR` — consumed ONLY by the MCP output-dir resolution (SPEC-CFGSCHEMA-005).
pub const ENV_OUTPUT_DIR: &str = "NABA_OUTPUT_DIR";

// ---------------------------------------------------------------------------
// Result JSON shape (SPEC-JSON-001) + printers (SPEC-JSON-002/003)
// ---------------------------------------------------------------------------

/// Metadata about a generated image, serialized as the CLI's JSON output (SPEC-JSON-001).
///
/// Field order and JSON names are pinned: `path`, `command`, `prompt`, `elapsed_ms`, `params`,
/// `requested_format`, `actual_format`. `params`, `requested_format`, and `actual_format` are
/// omitempty (an empty map / empty string is omitted), matching Go's `omitempty`.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Result {
    pub path: String,
    pub command: String,
    pub prompt: String,
    pub elapsed_ms: i64,
    /// Per-command parameter set. Omitted when empty (matches Go `omitempty` on a map).
    #[serde(skip_serializing_if = "Map::is_empty")]
    pub params: Map<String, Value>,
    /// The format the caller's `-o` extension implied (e.g. `"png"`), or `""` when auto-named.
    /// Differs from `actual_format` only when the on-disk extension was corrected.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub requested_format: String,
    /// The format actually written, derived from the response mimeType (e.g. `"jpeg"`).
    #[serde(skip_serializing_if = "String::is_empty")]
    pub actual_format: String,
}

impl Result {
    /// Populate the common fields; `elapsed_ms` is measured from `start` (Go's `NewResult`,
    /// which takes a start `time.Time`). `params`/format fields are filled by the caller.
    pub fn new(
        path: impl Into<String>,
        command: impl Into<String>,
        prompt: impl Into<String>,
        start: Instant,
    ) -> Self {
        Self {
            path: path.into(),
            command: command.into(),
            prompt: prompt.into(),
            elapsed_ms: start.elapsed().as_millis() as i64,
            ..Default::default()
        }
    }

    /// Copy the requested/actual format fields from a [`WriteResult`] onto this `Result` so the
    /// JSON reports requested-vs-actual format (Go's `applyFormat`).
    pub fn apply_format(&mut self, w: &WriteResult) {
        self.requested_format = w.requested_format.clone();
        self.actual_format = w.actual_format.clone();
    }
}

/// Serialize a single `Result` as 2-space-indented JSON (Go's `PrintJSON`).
pub fn to_json(r: &Result) -> String {
    serde_json::to_string_pretty(r).expect("Result serializes")
}

/// Serialize a slice of `Result`s as a 2-space-indented JSON array (Go's `PrintJSONMulti`).
pub fn to_json_multi(results: &[Result]) -> String {
    serde_json::to_string_pretty(results).expect("Results serialize")
}

/// Print a single `Result` object to stdout (SPEC-JSON-001).
pub fn print_json(r: &Result) {
    println!("{}", to_json(r));
}

/// Print a `Result` array to stdout (SPEC-JSON-002 multi / SPEC-JSON-003 story-always-array).
pub fn print_json_multi(results: &[Result]) {
    println!("{}", to_json_multi(results));
}

/// SPEC-JSON-002: single-image commands emit a single OBJECT for one result, an ARRAY for
/// more than one. `story` does NOT use this — it always calls [`print_json_multi`]
/// (SPEC-JSON-003). The command layer decides which helper to call; this is the convenience
/// for the single-vs-array commands.
pub fn print_json_auto(results: &[Result]) {
    if results.len() == 1 {
        print_json(&results[0]);
    } else {
        print_json_multi(results);
    }
}

// ---------------------------------------------------------------------------
// doctor envelope (SPEC-JSON-004)
// ---------------------------------------------------------------------------

/// Check status strings for the doctor envelope (Go's `statusPass`/`statusWarn`/`statusFail`).
pub mod status {
    pub const PASS: &str = "pass";
    pub const WARN: &str = "warn";
    pub const FAIL: &str = "fail";
}

/// One health check in the doctor envelope (SPEC-JSON-004). JSON: `name`, `status`, `detail`.
#[derive(Debug, Clone, Serialize)]
pub struct DoctorCheck {
    pub name: String,
    pub status: String,
    pub detail: String,
}

impl DoctorCheck {
    pub fn new(
        name: impl Into<String>,
        status: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            status: status.into(),
            detail: detail.into(),
        }
    }
}

/// The doctor JSON envelope: `{"ok": bool, "failed": int, "checks": [...]}` (SPEC-JSON-004).
/// `ok`/`failed` are DERIVED from the checks (a check with status `fail` counts as failed).
#[derive(Debug, Clone, Serialize)]
pub struct DoctorEnvelope {
    pub ok: bool,
    pub failed: i64,
    pub checks: Vec<DoctorCheck>,
}

impl DoctorEnvelope {
    /// Build the envelope from checks, computing `failed` = count of `fail`-status checks and
    /// `ok` = (failed == 0). Matches Go's `reportDoctor`.
    pub fn from_checks(checks: Vec<DoctorCheck>) -> Self {
        let failed = checks.iter().filter(|c| c.status == status::FAIL).count() as i64;
        Self {
            ok: failed == 0,
            failed,
            checks,
        }
    }

    /// Serialize as 2-space-indented JSON.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("DoctorEnvelope serializes")
    }
}

// ---------------------------------------------------------------------------
// config get/set --json envelope (Issue 1.4)
// ---------------------------------------------------------------------------

/// The `config get`/`config set` JSON envelope (Issue 1.4). PROVISIONAL: the universal envelope
/// contract is finalized by Issue 2.4; this uses a `status`/key/value shape consistent with the
/// other envelopes so 2.4 can normalize it without rework. Serialized 2-space-indented.
#[derive(Debug, Clone, Serialize)]
pub struct ConfigEnvelope {
    /// `"ok"` on success (the only status this success-path envelope emits; errors surface as
    /// exit codes + stderr, per the pre-2.4 convention).
    pub status: String,
    pub key: String,
    pub value: String,
}

impl ConfigEnvelope {
    pub fn ok(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            status: "ok".to_string(),
            key: key.into(),
            value: value.into(),
        }
    }

    /// Serialize as 2-space-indented JSON.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("ConfigEnvelope serializes")
    }
}

/// Print the `config get`/`config set` JSON envelope to stdout.
pub fn print_config_json(key: &str, value: &str) {
    println!("{}", ConfigEnvelope::ok(key, value).to_json());
}

// ---------------------------------------------------------------------------
// File writing + extension reconciliation (SPEC §3 file writing)
// ---------------------------------------------------------------------------

/// The outcome of writing an image, including any extension correction applied so the on-disk
/// file matches the response mimeType (Go's `WriteResult`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WriteResult {
    /// The absolute path actually written (extension already reconciled).
    pub path: String,
    /// The normalized format the caller's `-o` extension implied (e.g. `"png"`), or `""` when
    /// no output path was given (auto-named).
    pub requested_format: String,
    /// The normalized format actually written, derived from the response mimeType.
    pub actual_format: String,
    /// `true` when the on-disk extension was changed to match the response mimeType.
    pub corrected: bool,
}

/// Write image `data` to disk, reconciling the on-disk extension to the response `mime` and
/// reporting any correction (Go's `WriteImageResult`).
///
/// * `output_path` empty → auto-name `naba-<command>-<YYYYMMDD-HHMMSS>[-N].<ext>` in CWD.
/// * `output_path` set → the requested extension is reconciled to `mime` BEFORE any index
///   suffix. The Gemini image API returns JPEG, so a `-o foo.png` is corrected to `foo.jpg`
///   and `corrected` is set so the caller can warn and surface requested-vs-actual format.
/// * `index > 0` → multi-output: append `-N` (N = `index + 1`) to the filename.
///
/// Creates the parent directory `0o755`, dedups with a `-1..-999` suffix on collision, and
/// writes the file `0o644`. Returns the absolute path plus the reconciliation details.
///
/// This is the CLI write path (SPEC-CFGSCHEMA-005): it takes an explicit target and NEVER
/// consults `NABA_OUTPUT_DIR`.
pub fn write_image_result(
    data: &[u8],
    mime: &str,
    output_path: &str,
    command: &str,
    index: usize,
) -> std::io::Result<WriteResult> {
    let actual_format = format_from_mime(mime);
    let mut res = WriteResult {
        actual_format: actual_format.clone(),
        ..Default::default()
    };

    let mut path = if output_path.is_empty() {
        generate_filename(command, mime, index)
    } else {
        let mut p = output_path.to_string();
        res.requested_format = format_from_ext(&ext_with_dot(&p));
        // Reconcile the extension to the response mimeType before any index suffix.
        if !res.requested_format.is_empty() && res.requested_format != actual_format {
            let ext = ext_with_dot(&p);
            p = trim_suffix(&p, &ext) + &mime_type_to_ext(mime);
            res.corrected = true;
        }
        if index > 0 {
            // Multiple outputs: append index to the filename.
            let ext = ext_with_dot(&p);
            let base = trim_suffix(&p, &ext);
            p = format!("{base}-{}{ext}", index + 1);
        }
        p
    };

    // Ensure the directory exists (0o755).
    let dir = Path::new(&path)
        .parent()
        .map(|d| d.to_string_lossy().into_owned())
        .unwrap_or_default();
    if !dir.is_empty() && dir != "." {
        mkdir_all_0755(&dir)?;
    }

    // Dedup: on collision, append a -1..-999 suffix.
    path = dedup(&path);

    write_file_0644(&path, data)?;

    res.path = abs_path(&path);
    Ok(res)
}

/// Auto-generate a filename: `naba-<command>-<YYYYMMDD-HHMMSS>[-N].<ext>` (Go's `generateFilename`).
/// `N` (= `index + 1`) is appended only when `index > 0`.
pub fn generate_filename(command: &str, mime: &str, index: usize) -> String {
    let ext = mime_type_to_ext(mime);
    let ts = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    if index > 0 {
        format!("naba-{command}-{ts}-{}{ext}", index + 1)
    } else {
        format!("naba-{command}-{ts}{ext}")
    }
}

/// Join a directory with an auto-generated filename (Go's `OutputPath`). Empty `dir` → empty
/// string (the caller then auto-names in CWD via [`write_image_result`]). The MCP server uses
/// this after resolving the directory with [`mcp_output_dir`].
pub fn output_path(dir: &str, command: &str, mime: &str) -> String {
    if dir.is_empty() {
        return String::new();
    }
    Path::new(dir)
        .join(generate_filename(command, mime, 0))
        .to_string_lossy()
        .into_owned()
}

// ---------------------------------------------------------------------------
// mimeType / format helpers (kept in lockstep so reconciliation never disagrees)
// ---------------------------------------------------------------------------

/// Response mimeType → file extension (Go's `mimeTypeToExt`). Unknown → `.png`.
pub fn mime_type_to_ext(mime: &str) -> String {
    match mime {
        "image/png" => ".png",
        "image/jpeg" => ".jpg",
        "image/gif" => ".gif",
        "image/webp" => ".webp",
        _ => ".png",
    }
    .to_string()
}

/// Format string → file extension (Go's `ExtForFormat`). Case-insensitive; unknown → `.png`.
pub fn ext_for_format(format: &str) -> String {
    match format.to_lowercase().as_str() {
        "jpeg" | "jpg" => ".jpg",
        "png" => ".png",
        _ => ".png",
    }
    .to_string()
}

/// Normalized format for a file extension (Go's `formatFromExt`). `.jpg`/`.jpeg` → `jpeg`;
/// `.png` → `png`; `.gif`/`.webp` likewise; empty/unrecognized → `""`.
pub fn format_from_ext(ext: &str) -> String {
    match ext.to_lowercase().as_str() {
        ".jpg" | ".jpeg" => "jpeg",
        ".png" => "png",
        ".gif" => "gif",
        ".webp" => "webp",
        _ => "",
    }
    .to_string()
}

/// Normalized format for a response mimeType (Go's `formatFromMIME`), kept in lockstep with
/// [`mime_type_to_ext`]. Unknown → `png`.
pub fn format_from_mime(mime: &str) -> String {
    match mime {
        "image/png" => "png",
        "image/jpeg" => "jpeg",
        "image/gif" => "gif",
        "image/webp" => "webp",
        _ => "png",
    }
    .to_string()
}

/// The stderr note the command layer emits (unless `--quiet`) when the on-disk extension was
/// corrected, matching Go's `generate.go`:
/// `Note: requested .%s output but API returned %s; saved as %s`.
pub fn extension_correction_note(
    requested_format: &str,
    actual_format: &str,
    filename: &str,
) -> String {
    format!("Note: requested .{requested_format} output but API returned {actual_format}; saved as {filename}")
}

// ---------------------------------------------------------------------------
// MCP output-dir resolution (SPEC-CFGSCHEMA-004/005) — NOT used by the CLI path
// ---------------------------------------------------------------------------

/// Output-dir precedence for the MCP server (SPEC-CFGSCHEMA-004): `NABA_OUTPUT_DIR` env >
/// config `default_output_dir` (passed in — config parsing is Issue 3.1) > `None`.
/// Go's `config.ResolveOutputDir`.
pub fn resolve_output_dir(config_default: Option<&str>) -> Option<String> {
    if let Ok(dir) = std::env::var(ENV_OUTPUT_DIR) {
        if !dir.is_empty() {
            return Some(dir);
        }
    }
    config_default
        .filter(|d| !d.is_empty())
        .map(|d| d.to_string())
}

/// The XDG-conventional default output directory `~/.local/share/naba/images`
/// (Go's `config.DefaultOutputDir`). `None` when the home dir can't be determined.
pub fn default_output_dir() -> Option<String> {
    home_dir().map(|h| {
        h.join(".local")
            .join("share")
            .join("naba")
            .join("images")
            .to_string_lossy()
            .into_owned()
    })
}

/// The MCP output directory: [`resolve_output_dir`] with a fallback to [`default_output_dir`]
/// (Go's `resolveOutputDirWithDefault`). Consumed ONLY by the MCP server (SPEC-CFGSCHEMA-005);
/// the CLI never calls this.
pub fn mcp_output_dir(config_default: Option<&str>) -> String {
    resolve_output_dir(config_default)
        .or_else(default_output_dir)
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Preview (SPEC §preview) — launch the OS viewer, non-blocking, errors ignored
// ---------------------------------------------------------------------------

/// Open `path` in the system's default viewer (Go's `preview.go`): `open` (macOS), `start`
/// (Windows), `xdg-open` (other). Non-blocking (the child is spawned, not waited on) and
/// ERRORS ARE IGNORED — the command layer calls this only when `--preview` is set.
pub fn preview(path: &str) {
    let cmd = if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(target_os = "windows") {
        "start"
    } else {
        "xdg-open"
    };
    // Spawn (do not wait); swallow any error per SPEC §preview.
    let _ = std::process::Command::new(cmd).arg(path).spawn();
}

// ---------------------------------------------------------------------------
// internal helpers
// ---------------------------------------------------------------------------

/// The extension INCLUDING the leading dot (e.g. `".png"`), or `""` when there is none —
/// matching Go's `filepath.Ext`.
fn ext_with_dot(path: &str) -> String {
    match Path::new(path).extension() {
        Some(e) => format!(".{}", e.to_string_lossy()),
        None => String::new(),
    }
}

/// Remove `suffix` from the end of `s` if present (Go's `strings.TrimSuffix`).
fn trim_suffix(s: &str, suffix: &str) -> String {
    if !suffix.is_empty() {
        if let Some(stripped) = s.strip_suffix(suffix) {
            return stripped.to_string();
        }
    }
    s.to_string()
}

/// On collision, append a `-1..-999` suffix before the extension (Go's `dedup`). Returns the
/// original path when it does not exist, or after 999 attempts.
fn dedup(path: &str) -> String {
    if !Path::new(path).exists() {
        return path.to_string();
    }
    let ext = ext_with_dot(path);
    let base = trim_suffix(path, &ext);
    for i in 1..1000 {
        let candidate = format!("{base}-{i}{ext}");
        if !Path::new(&candidate).exists() {
            return candidate;
        }
    }
    path.to_string()
}

/// Lexical absolute path WITHOUT symlink resolution, matching Go's `filepath.Abs` (join with
/// CWD + clean). Falls back to the input on error.
fn abs_path(path: &str) -> String {
    std::path::absolute(path)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| path.to_string())
}

fn home_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        std::env::var_os("USERPROFILE").map(PathBuf::from)
    }
    #[cfg(not(windows))]
    {
        std::env::var_os("HOME").map(PathBuf::from)
    }
}

/// `mkdir -p` with directory mode `0o755` (SPEC file-writing perms).
fn mkdir_all_0755(dir: &str) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o755)
            .create(dir)
    }
    #[cfg(not(unix))]
    {
        std::fs::create_dir_all(dir)
    }
}

/// Write `data` to `path` with file mode `0o644` (SPEC file-writing perms).
fn write_file_0644(path: &str, data: &[u8]) -> std::io::Result<()> {
    std::fs::write(path, data)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o644))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    // ---- auto-naming ----

    #[test]
    fn auto_name_format() {
        // naba-<command>-YYYYMMDD-HHMMSS.<ext>: 4 dash groups + .png, matches Go's zero-index test.
        let name = generate_filename("generate", "image/png", 0);
        assert!(name.starts_with("naba-generate-"), "got {name}");
        assert!(name.ends_with(".png"), "got {name}");
        let stem = name.strip_suffix(".png").unwrap();
        assert_eq!(stem.split('-').count(), 4, "expected 4 dash groups: {name}");
    }

    #[test]
    fn auto_name_with_index() {
        let name = generate_filename("generate", "image/png", 2);
        assert!(name.contains("-3"), "index+1 suffix: {name}");
        assert!(name.ends_with(".png"));
    }

    #[test]
    fn auto_name_jpeg_ext() {
        let name = generate_filename("edit", "image/jpeg", 0);
        assert!(name.ends_with(".jpg"), "got {name}");
    }

    #[test]
    fn write_auto_named_is_absolute() {
        let dir = tempdir();
        let res = write_image_result(b"png-data", "image/png", "", "generate", 0).unwrap();
        // written into CWD; move on so we don't leave it — clean up by abs path.
        assert!(
            Path::new(&res.path).is_absolute(),
            "expected abs path: {}",
            res.path
        );
        assert!(res.path.contains("naba-generate-"));
        assert!(res.path.ends_with(".png"));
        assert_eq!(std::fs::read(&res.path).unwrap(), b"png-data");
        std::fs::remove_file(&res.path).ok();
        drop(dir);
    }

    // ---- extension reconciliation ----

    #[test]
    fn reconcile_png_request_jpeg_response() {
        let dir = tempdir();
        let out = dir.join("hero.png");
        let res = write_image_result(
            b"jpeg-bytes",
            "image/jpeg",
            out.to_str().unwrap(),
            "generate",
            0,
        )
        .unwrap();
        assert!(
            res.corrected,
            "png path + jpeg response should be corrected"
        );
        assert_eq!(res.requested_format, "png");
        assert_eq!(res.actual_format, "jpeg");
        assert!(res.path.ends_with(".jpg"), "got {}", res.path);
        assert!(Path::new(&res.path).exists());
    }

    #[test]
    fn no_correction_when_match() {
        let dir = tempdir();
        let out = dir.join("ok.jpg");
        let res =
            write_image_result(b"d", "image/jpeg", out.to_str().unwrap(), "generate", 0).unwrap();
        assert!(!res.corrected);
        assert_eq!(res.path, abs_path(out.to_str().unwrap()));
    }

    #[test]
    fn jpg_jpeg_equivalent() {
        let dir = tempdir();
        let out = dir.join("x.jpeg");
        let res =
            write_image_result(b"d", "image/jpeg", out.to_str().unwrap(), "generate", 0).unwrap();
        assert!(!res.corrected, ".jpeg and image/jpeg are equivalent");
    }

    #[test]
    fn auto_named_has_no_requested_format() {
        let dir = tempdir();
        let out = dir.join("");
        // exercise auto-name inside a tempdir by passing "" — writes to CWD, so just clean up.
        let _ = out;
        let res = write_image_result(b"d", "image/jpeg", "", "generate", 0).unwrap();
        assert_eq!(res.requested_format, "");
        assert_eq!(res.actual_format, "jpeg");
        assert!(res.path.ends_with(".jpg"));
        assert!(!res.corrected);
        std::fs::remove_file(&res.path).ok();
    }

    // ---- dedup ----

    #[test]
    fn dedup_suffixing() {
        let dir = tempdir();
        let out = dir.join("dup.png");
        let out_s = out.to_str().unwrap();
        let first = write_image_result(b"a", "image/png", out_s, "generate", 0).unwrap();
        assert_eq!(first.path, abs_path(out_s));
        let second = write_image_result(b"b", "image/png", out_s, "generate", 0).unwrap();
        assert!(second.path.ends_with("dup-1.png"), "got {}", second.path);
    }

    #[test]
    fn dedup_multiple_conflicts() {
        let dir = tempdir();
        for name in ["test.png", "test-1.png", "test-2.png"] {
            std::fs::write(dir.join(name), b"x").unwrap();
        }
        let res = write_image_result(
            b"y",
            "image/png",
            dir.join("test.png").to_str().unwrap(),
            "generate",
            0,
        )
        .unwrap();
        assert!(res.path.ends_with("test-3.png"), "got {}", res.path);
    }

    // ---- multi-index naming ----

    #[test]
    fn multi_index_naming() {
        let dir = tempdir();
        let out = dir.join("multi.png");
        let res =
            write_image_result(b"d", "image/png", out.to_str().unwrap(), "generate", 1).unwrap();
        assert!(res.path.contains("multi-2"), "got {}", res.path);
    }

    // ---- perms ----

    #[cfg(unix)]
    #[test]
    fn file_and_dir_perms() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempdir();
        let out = dir.join("perm").join("img.png");
        let res =
            write_image_result(b"d", "image/png", out.to_str().unwrap(), "generate", 0).unwrap();
        let fmode = std::fs::metadata(&res.path).unwrap().permissions().mode() & 0o777;
        assert_eq!(fmode, 0o644, "file mode");
        let dmode = std::fs::metadata(dir.join("perm"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(dmode, 0o755, "dir mode");
    }

    // ---- format helpers ----

    #[test]
    fn mime_and_format_helpers() {
        assert_eq!(mime_type_to_ext("image/png"), ".png");
        assert_eq!(mime_type_to_ext("image/jpeg"), ".jpg");
        assert_eq!(mime_type_to_ext("image/gif"), ".gif");
        assert_eq!(mime_type_to_ext("image/webp"), ".webp");
        assert_eq!(mime_type_to_ext("image/unknown"), ".png");

        assert_eq!(ext_for_format("png"), ".png");
        assert_eq!(ext_for_format("jpeg"), ".jpg");
        assert_eq!(ext_for_format("jpg"), ".jpg");
        assert_eq!(ext_for_format("PNG"), ".png");
        assert_eq!(ext_for_format("unknown"), ".png");

        assert_eq!(format_from_ext(".JPG"), "jpeg");
        assert_eq!(format_from_ext(".png"), "png");
        assert_eq!(format_from_ext(".xyz"), "");
        assert_eq!(format_from_mime("image/jpeg"), "jpeg");
        assert_eq!(format_from_mime("image/png"), "png");
    }

    #[test]
    fn correction_note_string() {
        assert_eq!(
            extension_correction_note("png", "jpeg", "hero.jpg"),
            "Note: requested .png output but API returned jpeg; saved as hero.jpg"
        );
    }

    // ---- OutputPath ----

    #[test]
    fn output_path_empty_dir() {
        assert_eq!(output_path("", "generate", "image/png"), "");
    }

    #[test]
    fn output_path_with_dir() {
        let got = output_path("/tmp/out", "generate", "image/png");
        assert!(got.starts_with("/tmp/out/naba-generate-"), "got {got}");
        assert!(got.ends_with(".png"));
    }

    #[test]
    fn output_path_jpeg() {
        let got = output_path("/tmp/out", "edit", "image/jpeg");
        assert!(got.starts_with("/tmp/out/naba-edit-"), "got {got}");
        assert!(got.ends_with(".jpg"));
    }

    // ---- Result JSON shape (SPEC-JSON-001/002/003) ----

    #[test]
    fn result_json_shape_and_indent() {
        let r = Result {
            path: "/tmp/test.png".into(),
            command: "generate".into(),
            prompt: "a cat".into(),
            elapsed_ms: 42,
            ..Default::default()
        };
        let json = to_json(&r);
        // 2-space indent.
        assert!(
            json.contains("\n  \"command\""),
            "2-space indent expected:\n{json}"
        );
        // exact field names present.
        for key in ["\"path\"", "\"command\"", "\"prompt\"", "\"elapsed_ms\""] {
            assert!(json.contains(key), "missing {key}:\n{json}");
        }
        // omitempty: no params/format keys when empty.
        assert!(
            !json.contains("\"params\""),
            "empty params must be omitted:\n{json}"
        );
        assert!(!json.contains("\"requested_format\""));
        assert!(!json.contains("\"actual_format\""));

        // field ORDER: path < command < prompt < elapsed_ms.
        let idx = |k: &str| json.find(k).unwrap();
        assert!(
            idx("\"path\"") < idx("\"command\"")
                && idx("\"command\"") < idx("\"prompt\"")
                && idx("\"prompt\"") < idx("\"elapsed_ms\"")
        );

        // parses back with expected scalar types.
        let v: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["path"], "/tmp/test.png");
        assert_eq!(v["elapsed_ms"], 42);
    }

    #[test]
    fn result_json_includes_nonempty_params_and_formats() {
        let mut params = Map::new();
        params.insert("size".into(), Value::from("1K"));
        let r = Result {
            path: "/p.png".into(),
            command: "generate".into(),
            prompt: "x".into(),
            elapsed_ms: 1,
            params,
            requested_format: "png".into(),
            actual_format: "jpeg".into(),
        };
        let json = to_json(&r);
        assert!(json.contains("\"params\""));
        assert!(json.contains("\"requested_format\": \"png\""));
        assert!(json.contains("\"actual_format\": \"jpeg\""));
    }

    #[test]
    fn new_result_measures_elapsed() {
        let start = Instant::now() - Duration::from_millis(50);
        let r = Result::new("/p", "generate", "prompt", start);
        assert_eq!(r.path, "/p");
        assert_eq!(r.command, "generate");
        assert_eq!(r.prompt, "prompt");
        assert!(r.elapsed_ms >= 50, "elapsed {}", r.elapsed_ms);
        assert!(r.params.is_empty());
    }

    #[test]
    fn single_vs_array() {
        let one = [Result {
            path: "/a.png".into(),
            command: "generate".into(),
            ..Default::default()
        }];
        // Single → object (starts with '{').
        assert!(to_json(&one[0]).trim_start().starts_with('{'));
        let many = vec![
            one[0].clone(),
            Result {
                path: "/b.png".into(),
                command: "edit".into(),
                ..Default::default()
            },
        ];
        // Multi → array (starts with '[').
        assert!(to_json_multi(&many).trim_start().starts_with('['));
    }

    #[test]
    fn story_always_array_even_for_one() {
        // SPEC-JSON-003: story uses to_json_multi even for a single frame → array.
        let one = vec![Result {
            path: "/frame.png".into(),
            command: "story".into(),
            ..Default::default()
        }];
        let json = to_json_multi(&one);
        assert!(
            json.trim_start().starts_with('['),
            "story must always be an array:\n{json}"
        );
    }

    // ---- doctor envelope (SPEC-JSON-004) ----

    #[test]
    fn doctor_envelope_shape() {
        let checks = vec![
            DoctorCheck::new("version", status::PASS, "naba 0.1.0"),
            DoctorCheck::new("api_key", status::FAIL, "GEMINI_API_KEY not set"),
            DoctorCheck::new("api_live", status::WARN, "offline"),
        ];
        let env = DoctorEnvelope::from_checks(checks);
        assert!(!env.ok);
        assert_eq!(env.failed, 1);
        let json = env.to_json();
        let v: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["ok"], false);
        assert_eq!(v["failed"], 1);
        assert_eq!(v["checks"][0]["name"], "version");
        assert_eq!(v["checks"][0]["status"], "pass");
        assert_eq!(v["checks"][1]["detail"], "GEMINI_API_KEY not set");
        // 2-space indent.
        assert!(json.contains("\n  \"failed\""), "2-space indent:\n{json}");
    }

    #[test]
    fn doctor_envelope_all_pass_ok() {
        let env = DoctorEnvelope::from_checks(vec![DoctorCheck::new("a", status::PASS, "")]);
        assert!(env.ok);
        assert_eq!(env.failed, 0);
    }

    // ---- MCP output-dir resolution (SPEC-CFGSCHEMA-004/005) ----

    #[test]
    fn mcp_resolve_prefers_env() {
        // Guarded env manipulation; this test owns the var.
        std::env::set_var(ENV_OUTPUT_DIR, "/env/dir");
        assert_eq!(
            resolve_output_dir(Some("/cfg/dir")),
            Some("/env/dir".into())
        );
        std::env::remove_var(ENV_OUTPUT_DIR);
        assert_eq!(
            resolve_output_dir(Some("/cfg/dir")),
            Some("/cfg/dir".into())
        );
        assert_eq!(resolve_output_dir(None), None);
    }

    #[test]
    fn mcp_output_dir_falls_back_to_xdg() {
        std::env::remove_var(ENV_OUTPUT_DIR);
        let d = mcp_output_dir(None);
        assert!(
            d.ends_with(".local/share/naba/images") || d.is_empty(),
            "got {d}"
        );
    }

    // ---- test helper ----

    /// A minimal unique tempdir under the system temp dir (no external dep). Auto-removed.
    struct Temp(PathBuf);
    impl Temp {
        fn join(&self, p: &str) -> PathBuf {
            self.0.join(p)
        }
    }
    impl std::ops::Deref for Temp {
        type Target = PathBuf;
        fn deref(&self) -> &PathBuf {
            &self.0
        }
    }
    impl Drop for Temp {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).ok();
        }
    }
    fn tempdir() -> Temp {
        use std::sync::atomic::{AtomicU32, Ordering};
        static N: AtomicU32 = AtomicU32::new(0);
        let n = N.fetch_add(1, Ordering::Relaxed);
        let p = std::env::temp_dir().join(format!("naba-output-test-{}-{}", std::process::id(), n));
        std::fs::create_dir_all(&p).unwrap();
        Temp(p)
    }
}
