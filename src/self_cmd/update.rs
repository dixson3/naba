//! `naba self update` pipeline (SPEC-SELF-002..007).
//!
//! Version discovery is **not** the GitHub releases API — it reads the cargo-dist
//! `dist-manifest.json` published on the latest release, selects the `executable-zip` artifact
//! whose `target_triples` contains this binary's [`crate::version::HOST_TRIPLE`], downloads it
//! plus its `.sha256` sidecar, verifies the digest **before** any swap, extracts the binary with
//! pure-Rust flate2+tar ([`super::archive`]), and swaps it in place with `self_replace`.
//!
//! # Seams
//!
//! Network is behind the [`Fetcher`] trait and the on-disk swap behind a [`SwapFn`] closure, so
//! [`run_inner`] is fully unit-testable without a network or clobbering the test binary.
//!
//! # Source gate (SPEC-SELF-003)
//!
//! Homebrew installs are **always** refused (`brew upgrade naba`). Non-auto-updatable sources
//! (from-build / unknown) are refused **without** `--force`. Only [`Source::Vendor`] updates
//! unconditionally.

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::cli::SelfUpdateArgs;
use crate::commands::Globals;
use crate::error::{AppError, AppResult};
use crate::version;

use super::archive;
use super::receipt::{FromBuildMarker, Receipt};
use super::source::{self, Source};

/// Async network seam. The real impl is [`ReqwestFetcher`]; tests inject canned bytes.
#[async_trait::async_trait]
pub trait Fetcher {
    /// GET `url`, returning the response body bytes. Non-2xx is an error.
    async fn get_bytes(&self, url: &str) -> AppResult<Vec<u8>>;
}

/// Production [`Fetcher`] backed by reqwest (async, rustls).
pub struct ReqwestFetcher;

#[async_trait::async_trait]
impl Fetcher for ReqwestFetcher {
    async fn get_bytes(&self, url: &str) -> AppResult<Vec<u8>> {
        let resp = reqwest::get(url)
            .await
            .map_err(|e| AppError::general(format!("fetch {url}: {e}")))?;
        if !resp.status().is_success() {
            return Err(AppError::general(format!(
                "fetch {url}: HTTP {}",
                resp.status().as_u16()
            )));
        }
        let bytes = resp
            .bytes()
            .await
            .map_err(|e| AppError::general(format!("read {url}: {e}")))?;
        Ok(bytes.to_vec())
    }
}

/// The on-disk binary-swap seam. Production is `self_replace::self_replace`.
pub type SwapFn<'a> = &'a (dyn Fn(&Path) -> AppResult<()> + Sync);

// ---- dist-manifest.json (cargo-dist) ------------------------------------------------------

/// The subset of cargo-dist's `dist-manifest.json` the updater reads.
#[derive(Debug, Clone, Default, Deserialize)]
struct DistManifest {
    #[serde(default)]
    announcement_tag: Option<String>,
    #[serde(default)]
    artifacts: std::collections::HashMap<String, Artifact>,
}

/// One dist-manifest artifact. `name` is the release-asset filename; `checksum` names the
/// artifact holding this one's `.sha256` sidecar.
#[derive(Debug, Clone, Default, Deserialize)]
struct Artifact {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    target_triples: Vec<String>,
    #[serde(default)]
    checksum: Option<String>,
}

/// Select the `executable-zip` artifact whose `target_triples` contains `host_triple`.
fn select_artifact<'a>(manifest: &'a DistManifest, host_triple: &str) -> Option<&'a Artifact> {
    manifest.artifacts.values().find(|a| {
        a.kind.as_deref() == Some("executable-zip")
            && a.target_triples.iter().any(|t| t == host_triple)
    })
}

// ---- version comparison -------------------------------------------------------------------

/// Parse the `x.y.z` core of a version/tag string (strips a leading `v` and any
/// `-prerelease`/`+build`/git-describe suffix). `None` when the first three dot components are
/// not all numeric (e.g. `"dev"`).
fn parse_core(s: &str) -> Option<(u64, u64, u64)> {
    let s = s.trim();
    let s = s.strip_prefix('v').unwrap_or(s);
    let core = s.split(['-', '+']).next().unwrap_or(s);
    let mut it = core.split('.');
    let a = it.next()?.parse().ok()?;
    let b = it.next()?.parse().ok()?;
    let c = it.next()?.parse().ok()?;
    Some((a, b, c))
}

/// Whether `latest` is strictly newer than `current`. When `current` is unparseable (e.g. a
/// `dev`/dirty build) any parseable `latest` counts as newer.
fn is_newer(latest: &str, current: &str) -> bool {
    match (parse_core(latest), parse_core(current)) {
        (Some(l), Some(c)) => l > c,
        (Some(_), None) => true,
        _ => false,
    }
}

// ---- source detection ---------------------------------------------------------------------

/// Detect the running binary's install source (path-primary, SPEC-SELF-002): canonicalized
/// `current_exe()` + the receipt's vendor prefix + the from-build marker.
pub fn detect_source() -> AppResult<Source> {
    let exe = std::env::current_exe()
        .map(|p| std::fs::canonicalize(&p).unwrap_or(p))
        .map_err(|e| AppError::general(format!("resolve current_exe: {e}")))?;
    let vendor_prefix = Receipt::load()?.and_then(|r| r.canonical_install_prefix());
    let from_build = FromBuildMarker::exists();
    Ok(source::classify(&exe, vendor_prefix.as_deref(), from_build))
}

// ---- entry point --------------------------------------------------------------------------

/// The manifest URL on the latest release (SPEC-SELF-004).
fn manifest_url() -> String {
    format!(
        "{}/releases/latest/download/dist-manifest.json",
        env!("CARGO_PKG_REPOSITORY")
    )
}

/// A release-asset download URL for `asset_name` at `tag`.
fn asset_url(tag: &str, asset_name: &str) -> String {
    format!(
        "{}/releases/download/{tag}/{asset_name}",
        env!("CARGO_PKG_REPOSITORY")
    )
}

/// Production entry: real fetcher + `self_replace` swap, detected source, this binary's version.
pub async fn run(args: &SelfUpdateArgs, globals: &Globals) -> AppResult<()> {
    let fetcher = ReqwestFetcher;
    let swap: SwapFn = &|new_bin: &Path| {
        self_replace::self_replace(new_bin)
            .map_err(|e| AppError::file_io(format!("swap binary: {e}")))
    };
    let source = detect_source()?;
    run_inner(
        args,
        globals,
        &fetcher,
        swap,
        source,
        version::VERSION,
        &manifest_url(),
    )
    .await
}

/// The outcome of an update run, for the `--json` envelope.
struct Outcome {
    status: &'static str,
    source: Source,
    current: String,
    latest: Option<String>,
    guidance: Option<String>,
}

impl Outcome {
    fn to_json(&self) -> String {
        let mut obj = serde_json::Map::new();
        obj.insert("command".into(), "self update".into());
        obj.insert("status".into(), self.status.into());
        obj.insert("source".into(), self.source.tag().into());
        obj.insert("current".into(), self.current.clone().into());
        if let Some(l) = &self.latest {
            obj.insert("latest".into(), l.clone().into());
        }
        if let Some(g) = &self.guidance {
            obj.insert("guidance".into(), g.clone().into());
        }
        serde_json::to_string_pretty(&serde_json::Value::Object(obj)).unwrap_or_default()
    }
}

/// The testable pipeline core (SPEC-SELF-002..005). `swap` and `fetcher` are seams; `source` and
/// `current_version` are injected so tests exercise every branch.
#[allow(clippy::too_many_arguments)]
pub async fn run_inner<F: Fetcher>(
    args: &SelfUpdateArgs,
    globals: &Globals,
    fetcher: &F,
    swap: SwapFn<'_>,
    source: Source,
    current_version: &str,
    manifest_url: &str,
) -> AppResult<()> {
    // 1. Source gate. Homebrew always refuses; non-auto-updatable refuses without --force.
    if source == Source::Homebrew || (!source::auto_updatable(source) && !args.force) {
        let guidance = source::refusal_guidance(source);
        let outcome = Outcome {
            status: "refused",
            source,
            current: current_version.to_string(),
            latest: None,
            guidance: Some(guidance.clone()),
        };
        emit(globals, &outcome);
        return Err(AppError::general(guidance));
    }

    // 2. Fetch + parse the dist-manifest.
    let manifest_bytes = fetcher.get_bytes(manifest_url).await?;
    let manifest: DistManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|e| AppError::general(format!("parse dist-manifest: {e}")))?;
    let tag = manifest
        .announcement_tag
        .clone()
        .ok_or_else(|| AppError::general("dist-manifest has no announcement_tag"))?;
    let latest = tag.trim_start_matches('v').to_string();

    // 3. Up-to-date short-circuit (unless --force).
    if !args.force && !is_newer(&latest, current_version) {
        let outcome = Outcome {
            status: "up-to-date",
            source,
            current: current_version.to_string(),
            latest: Some(latest.clone()),
            guidance: None,
        };
        emit(globals, &outcome);
        if !globals.json && !globals.quiet {
            println!("naba is up to date ({current_version}).");
        }
        return Ok(());
    }

    // 4. Select the host artifact.
    let artifact = select_artifact(&manifest, version::HOST_TRIPLE).ok_or_else(|| {
        AppError::general(format!(
            "no release artifact for this platform ({})",
            version::HOST_TRIPLE
        ))
    })?;
    let asset_name = artifact
        .name
        .clone()
        .ok_or_else(|| AppError::general("selected artifact has no name"))?;

    // 5. --check stops here (report availability, no download/swap).
    if args.check {
        let outcome = Outcome {
            status: "available",
            source,
            current: current_version.to_string(),
            latest: Some(latest.clone()),
            guidance: None,
        };
        emit(globals, &outcome);
        if !globals.json && !globals.quiet {
            println!("update available: {current_version} -> {latest} (run `naba self update`)");
        }
        return Ok(());
    }

    // 6. Download the archive + its sha256 sidecar; verify BEFORE any swap.
    let archive_bytes = fetcher.get_bytes(&asset_url(&tag, &asset_name)).await?;
    let checksum_asset = artifact
        .checksum
        .clone()
        .unwrap_or_else(|| format!("{asset_name}.sha256"));
    let sha_bytes = fetcher.get_bytes(&asset_url(&tag, &checksum_asset)).await?;
    let expected = archive::parse_sha256_file(&String::from_utf8_lossy(&sha_bytes))
        .ok_or_else(|| AppError::general("malformed .sha256 sidecar"))?;
    let actual = archive::sha256_hex(&archive_bytes);
    if actual != expected {
        return Err(AppError::general(format!(
            "checksum mismatch: expected {expected}, got {actual}"
        )));
    }

    // 7. Extract + 8. swap.
    let scratch = scratch_dir();
    let new_bin = archive::extract_binary(&archive_bytes, "naba", &scratch)?;
    swap(&new_bin)?;
    let _ = std::fs::remove_dir_all(&scratch);

    // A forced update of a from-build install clears its marker (SPEC-SELF-001).
    if source == Source::FromBuild {
        let _ = FromBuildMarker::remove();
    }

    // 9. Post-update skills refresh (B.6) unless --binary-only.
    if !args.binary_only {
        post_update_skills_refresh(globals)?;
    }

    let outcome = Outcome {
        status: "updated",
        source,
        current: current_version.to_string(),
        latest: Some(latest.clone()),
        guidance: None,
    };
    emit(globals, &outcome);
    if !globals.json && !globals.quiet {
        println!("updated naba: {current_version} -> {latest}");
    }
    Ok(())
}

/// Post-update skills refresh — implemented in Issue B.6. Scaffold no-op for B.5.
fn post_update_skills_refresh(_globals: &Globals) -> AppResult<()> {
    Ok(())
}

/// A unique scratch dir under the system temp for extraction.
fn scratch_dir() -> PathBuf {
    std::env::temp_dir().join(format!("naba-self-update-{}", std::process::id()))
}

/// Emit the JSON envelope when `--json` is in effect.
fn emit(globals: &Globals, outcome: &Outcome) {
    if globals.json {
        println!("{}", outcome.to_json());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    fn globals(json: bool) -> Globals {
        Globals {
            json,
            output: None,
            quiet: true,
            model: None,
            no_input: true,
            provider: None,
        }
    }

    fn args(check: bool, force: bool, binary_only: bool) -> SelfUpdateArgs {
        SelfUpdateArgs {
            check,
            force,
            binary_only,
        }
    }

    /// A canned fetcher mapping URL shape to byte payloads.
    struct MockFetcher {
        manifest: Vec<u8>,
        archive: Vec<u8>,
        sha: Vec<u8>,
    }

    #[async_trait::async_trait]
    impl Fetcher for MockFetcher {
        async fn get_bytes(&self, url: &str) -> AppResult<Vec<u8>> {
            if url.contains("dist-manifest.json") {
                Ok(self.manifest.clone())
            } else if url.ends_with(".sha256") {
                Ok(self.sha.clone())
            } else {
                Ok(self.archive.clone())
            }
        }
    }

    fn make_targz(inner: &str, contents: &[u8]) -> Vec<u8> {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;
        let mut b = tar::Builder::new(Vec::new());
        let mut h = tar::Header::new_gnu();
        h.set_size(contents.len() as u64);
        h.set_mode(0o755);
        h.set_cksum();
        b.append_data(&mut h, inner, contents).unwrap();
        let tar = b.into_inner().unwrap();
        let mut e = GzEncoder::new(Vec::new(), Compression::default());
        e.write_all(&tar).unwrap();
        e.finish().unwrap()
    }

    fn manifest_json(tag: &str, host: &str, asset: &str) -> Vec<u8> {
        format!(
            r#"{{
              "announcement_tag": "{tag}",
              "artifacts": {{
                "{asset}": {{
                  "name": "{asset}",
                  "kind": "executable-zip",
                  "target_triples": ["{host}"],
                  "checksum": "{asset}.sha256"
                }},
                "{asset}.sha256": {{ "name": "{asset}.sha256", "kind": "checksum", "target_triples": [] }}
              }}
            }}"#
        )
        .into_bytes()
    }

    #[tokio::test]
    async fn homebrew_is_refused_even_with_force() {
        let f = MockFetcher {
            manifest: vec![],
            archive: vec![],
            sha: vec![],
        };
        let noop_swap: SwapFn = &|_p| Ok(());
        let err = run_inner(
            &args(false, true, false),
            &globals(false),
            &f,
            noop_swap,
            Source::Homebrew,
            "0.1.0",
            "http://x/dist-manifest.json",
        )
        .await
        .unwrap_err();
        assert!(err.message.contains("brew upgrade naba"));
    }

    #[tokio::test]
    async fn from_build_refused_without_force() {
        let f = MockFetcher {
            manifest: vec![],
            archive: vec![],
            sha: vec![],
        };
        let noop_swap: SwapFn = &|_p| Ok(());
        let err = run_inner(
            &args(false, false, false),
            &globals(false),
            &f,
            noop_swap,
            Source::FromBuild,
            "0.1.0",
            "http://x/dist-manifest.json",
        )
        .await
        .unwrap_err();
        assert!(err.message.contains("--from-build"));
    }

    #[tokio::test]
    async fn up_to_date_short_circuits() {
        let host = version::HOST_TRIPLE;
        let f = MockFetcher {
            manifest: manifest_json("v0.1.0", host, "naba-x.tar.gz"),
            archive: vec![],
            sha: vec![],
        };
        let noop_swap: SwapFn = &|_p| Ok(());
        run_inner(
            &args(false, false, false),
            &globals(false),
            &f,
            noop_swap,
            Source::Vendor,
            "0.1.0",
            "http://x/dist-manifest.json",
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn check_reports_available_without_swapping() {
        let host = version::HOST_TRIPLE;
        let asset = "naba-x.tar.gz";
        let targz = make_targz("naba-triple/naba", b"new-binary");
        let f = MockFetcher {
            manifest: manifest_json("v0.2.0", host, asset),
            sha: format!("{}  {asset}", archive::sha256_hex(&targz)).into_bytes(),
            archive: targz,
        };
        let swapped = Mutex::new(false);
        let swap: SwapFn = &|_p| {
            *swapped.lock().unwrap() = true;
            Ok(())
        };
        run_inner(
            &args(true, false, false), // --check
            &globals(false),
            &f,
            swap,
            Source::Vendor,
            "0.1.0",
            "http://x/dist-manifest.json",
        )
        .await
        .unwrap();
        assert!(!*swapped.lock().unwrap(), "--check must not swap");
    }

    #[tokio::test]
    async fn full_update_verifies_and_swaps() {
        let host = version::HOST_TRIPLE;
        let asset = "naba-x.tar.gz";
        let payload = b"the-new-naba";
        let targz = make_targz("naba-triple/naba", payload);
        let good_sha = format!("{}  {asset}", archive::sha256_hex(&targz));
        let f = MockFetcher {
            manifest: manifest_json("v0.2.0", host, asset),
            sha: good_sha.into_bytes(),
            archive: targz,
        };
        let captured: Mutex<Option<Vec<u8>>> = Mutex::new(None);
        let swap: SwapFn = &|p| {
            *captured.lock().unwrap() = Some(std::fs::read(p).unwrap());
            Ok(())
        };
        run_inner(
            &args(false, false, false),
            &globals(false),
            &f,
            swap,
            Source::Vendor,
            "0.1.0",
            "http://x/dist-manifest.json",
        )
        .await
        .unwrap();
        assert_eq!(captured.lock().unwrap().as_deref(), Some(payload.as_ref()));
    }

    #[tokio::test]
    async fn checksum_mismatch_bails_before_swap() {
        let host = version::HOST_TRIPLE;
        let asset = "naba-x.tar.gz";
        let targz = make_targz("naba-triple/naba", b"payload");
        let f = MockFetcher {
            manifest: manifest_json("v0.2.0", host, asset),
            sha: format!("{}  {asset}", "0".repeat(64)).into_bytes(), // wrong digest
            archive: targz,
        };
        let swapped = Mutex::new(false);
        let swap: SwapFn = &|_p| {
            *swapped.lock().unwrap() = true;
            Ok(())
        };
        let err = run_inner(
            &args(false, false, false),
            &globals(false),
            &f,
            swap,
            Source::Vendor,
            "0.1.0",
            "http://x/dist-manifest.json",
        )
        .await
        .unwrap_err();
        assert!(err.message.contains("checksum mismatch"));
        assert!(
            !*swapped.lock().unwrap(),
            "must not swap on checksum mismatch"
        );
    }

    #[test]
    fn version_compare() {
        assert!(is_newer("0.2.0", "0.1.0"));
        assert!(is_newer("v0.2.0", "0.1.9"));
        assert!(!is_newer("0.1.0", "0.1.0"));
        assert!(!is_newer("0.1.0", "0.2.0"));
        // unparseable current (dev build) → any release counts as newer.
        assert!(is_newer("0.1.0", "dev"));
        // a git-describe build ahead of a tag shares the tag's core (0.1.0) → the released
        // 0.1.0 is NOT newer; a later release is.
        assert!(!is_newer("0.1.0", "0.1.0-5-gabcdef-dirty"));
        assert!(is_newer("0.2.0", "0.1.0-5-gabcdef-dirty"));
    }

    #[test]
    fn refused_json_envelope_shape() {
        let o = Outcome {
            status: "refused",
            source: Source::Homebrew,
            current: "0.1.0".into(),
            latest: None,
            guidance: Some("run brew".into()),
        };
        let v: serde_json::Value = serde_json::from_str(&o.to_json()).unwrap();
        assert_eq!(v["command"], "self update");
        assert_eq!(v["status"], "refused");
        assert_eq!(v["source"], "homebrew");
        assert_eq!(v["guidance"], "run brew");
    }
}
