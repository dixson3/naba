// Build-time version injection (SPEC-VERSION-BUILD-001, M3) + skill two-tree render
// (plan-008 Epic 3). Captures Version / Commit / Date into compile-time env vars (replacing
// Go's ldflags), and renders the single `skills/` source into `cli/` + `mcp/` trees under
// $OUT_DIR (dual-purpose skills — one authored source, two embedded variants).
use std::path::Path;
use std::process::Command;

fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8(out.stdout).ok()?.trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

fn utc_date() -> String {
    // Shell out to `date -u` to avoid pulling a date/time crate (near-zero-dep posture).
    Command::new("date")
        .args(["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

fn main() {
    // Prefer a version tag (`vX.Y.Z`) reachable from HEAD, with a `-N-g<sha>[-dirty]` suffix
    // for dev builds. `--match 'v[0-9]*'` ignores non-version tags (e.g. transient backup
    // tags) so they can never pollute the reported version. When no version tag is reachable
    // — notably a shallow CI checkout that did not fetch tags — fall back to the crate version
    // from Cargo.toml (`v<CARGO_PKG_VERSION>`) rather than a bare commit sha, so a release
    // binary always reports a parseable semver (self-update depends on it). `dev` is only used
    // if even that is unavailable (never under cargo).
    let version =
        git(&["describe", "--tags", "--match", "v[0-9]*", "--dirty"]).unwrap_or_else(|| {
            match std::env::var("CARGO_PKG_VERSION") {
                Ok(v) if !v.is_empty() => format!("v{v}"),
                _ => "dev".to_string(),
            }
        });
    let commit = git(&["rev-parse", "--short", "HEAD"]).unwrap_or_else(|| "none".to_string());
    let date = utc_date();

    // The compile target triple (e.g. aarch64-apple-darwin) — cargo sets $TARGET for build
    // scripts. `naba self update` matches this against the dist-manifest artifact target_triples
    // (SPEC-SELF-004). Fallback `unknown` should never occur under cargo.
    let host_triple = std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());

    println!("cargo:rustc-env=NABA_VERSION={version}");
    println!("cargo:rustc-env=NABA_COMMIT={commit}");
    println!("cargo:rustc-env=NABA_DATE={date}");
    println!("cargo:rustc-env=NABA_HOST_TRIPLE={host_triple}");

    // Re-run when the checked-out commit changes so the injected values stay fresh.
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs");

    // Render the dual-purpose skill trees (plan-008 Epic 3).
    render_skill_trees();
}

/// Render the single `skills/` source into two variants under `$OUT_DIR` (plan-008 Epic 3):
/// `$OUT_DIR/cli/<skill>/…` (the tree `embed.rs` embeds + `skills install` deploys) and
/// `$OUT_DIR/mcp/<skill>/…` (served by the MCP resource surface). `SKILL.md` is a minijinja
/// template gated by `{% if cli %}` / `{% if mcp %}`; every other file is copied verbatim. The
/// CLI render is authored to be byte-identical to the source (trim/lstrip block whitespace
/// control), so the pinned `embed.rs` tree hash is preserved.
fn render_skill_trees() {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let src_root = Path::new(&manifest).join("skills");
    let out = std::env::var("OUT_DIR").expect("OUT_DIR");
    let cli_root = Path::new(&out).join("cli");
    let mcp_root = Path::new(&out).join("mcp");

    // Re-run whenever any skill source file changes.
    println!("cargo:rerun-if-changed={}", src_root.display());

    let mut env = minijinja::Environment::new();
    env.set_trim_blocks(true);
    env.set_lstrip_blocks(true);
    // Preserve the source's final newline (Jinja strips one by default) — required for the
    // CLI render to be byte-identical to the source and keep the pinned tree hash.
    env.set_keep_trailing_newline(true);

    // Fresh render each build: clear stale output.
    for root in [&cli_root, &mcp_root] {
        let _ = std::fs::remove_dir_all(root);
        std::fs::create_dir_all(root).expect("mkdir out skill root");
    }

    render_dir(&env, &src_root, &src_root, &cli_root, &mcp_root);
}

/// Recursively render/copy `dir` (under `src_root`) into the `cli`/`mcp` output roots.
fn render_dir(
    env: &minijinja::Environment,
    src_root: &Path,
    dir: &Path,
    cli_root: &Path,
    mcp_root: &Path,
) {
    for entry in std::fs::read_dir(dir).expect("read skills dir") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        // Mirror Go's //go:embed exclusion of dotfiles / underscore-prefixed entries.
        if name.starts_with('.') || name.starts_with('_') {
            continue;
        }
        if path.is_dir() {
            render_dir(env, src_root, &path, cli_root, mcp_root);
            continue;
        }
        let rel = path.strip_prefix(src_root).expect("rel under src_root");
        let cli_dst = cli_root.join(rel);
        let mcp_dst = mcp_root.join(rel);
        for dst in [&cli_dst, &mcp_dst] {
            if let Some(parent) = dst.parent() {
                std::fs::create_dir_all(parent).expect("mkdir out subdir");
            }
        }
        // `SKILL.md` is the only templated file; everything else is copied verbatim.
        if name == "SKILL.md" {
            let source = std::fs::read_to_string(&path).expect("read SKILL.md");
            let cli = env
                .render_str(&source, minijinja::context! { cli => true, mcp => false })
                .expect("render SKILL.md (cli)");
            let mcp = env
                .render_str(&source, minijinja::context! { cli => false, mcp => true })
                .expect("render SKILL.md (mcp)");
            std::fs::write(&cli_dst, cli).expect("write cli SKILL.md");
            std::fs::write(&mcp_dst, mcp).expect("write mcp SKILL.md");
        } else {
            std::fs::copy(&path, &cli_dst).expect("copy cli file");
            std::fs::copy(&path, &mcp_dst).expect("copy mcp file");
        }
    }
}
