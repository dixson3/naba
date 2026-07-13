//! Skill-embed infrastructure (SPEC-EMBED-001..004, §12).
//!
//! Embeds the `skills/` tree into the binary at compile time and reproduces Go's
//! canonical tree-hash + marker-injection byte-for-byte, so `doctor` / `skills status`
//! hash comparisons behave identically to the Go implementation. This is the Rust port
//! of the module-root `naba` package in `skills_embed.go`.
//!
//! # SPEC-EMBED-004: hashing algorithm parity with Go (content intentionally diverged)
//!
//! We **reproduce** Go's `hashTree` algorithm exactly (same file sort order, same
//! `write(rel bytes) then write(file bytes)` concatenation, no newline normalization,
//! same SKILL.md marker-strip semantics, same lowercase hex encoding). Issue 4.0 verified
//! byte-for-byte parity against the Go binary for the UNCHANGED tree
//! (`EmbeddedTreeHash("naba") == 6dfa9939…d1cf`).
//!
//! **Issue 5.2 intentionally updated the embedded skill content** (`skills/naba/SKILL.md`,
//! documenting the multi-provider surface), and **plan-005 Issue C.3** updated it again (adding
//! the `## Preflight` section), so the embedded tree now hashes to `d5b2fdfe…368c8` (see
//! [`tests::embedded_hash_matches_go_reference`], now a Rust-content regression pin). Because the
//! shipped skill **content** changed — not just the hashing algorithm — existing on-disk installs
//! read "outdated" until `naba skills upgrade`. This flips the Issue 4.0 "no forced upgrade
//! needed" conclusion, which held only while the content was unchanged.

use std::path::Path;

use include_dir::{include_dir, Dir, DirEntry, File};
use sha2::{Digest, Sha256};

use crate::error::{AppError, AppResult};

/// The embedded `skills/` tree. `include_dir!` embeds every regular file; Go's
/// `//go:embed skills` excludes dotfile / underscore-prefixed entries, so
/// [`skill_files`] replicates that exclusion when enumerating the canonical tree.
static SKILLS: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/skills");

/// Opens the hidden integrity marker injected into a deployed SKILL.md
/// (SPEC-EMBED-001). Mirrors Go's `MarkerPrefix`.
pub const MARKER_PREFIX: &str = "<!-- naba-skills:";

/// Returns the embedded skill names (immediate subdirectories of `skills/`), sorted.
pub fn skill_names() -> Vec<String> {
    let mut names: Vec<String> = SKILLS
        .dirs()
        .filter_map(|d| {
            d.path()
                .file_name()
                .and_then(|n| n.to_str())
                .map(str::to_string)
        })
        .filter(|n| !is_excluded_component(n))
        .collect();
    names.sort();
    names
}

/// Returns the file paths within an embedded skill, relative to the skill's own
/// directory (e.g. `"SKILL.md"`, `"commands/edit.md"`), slash-separated and sorted.
///
/// Dotfile / underscore-prefixed path components are excluded to match Go's
/// `//go:embed` default set exactly — the canonical tree (and therefore its hash) must
/// contain the same files Go embeds, or the digest will not match.
pub fn skill_files(name: &str) -> Vec<String> {
    let mut rels = Vec::new();
    if let Some(dir) = SKILLS.get_dir(name) {
        let mut files = Vec::new();
        collect_files(dir, &mut files);
        let prefix = format!("{name}/");
        for f in files {
            let path = f.path().to_string_lossy().replace('\\', "/");
            let rel = path.strip_prefix(&prefix).unwrap_or(&path).to_string();
            if rel.split('/').any(is_excluded_component) {
                continue;
            }
            rels.push(rel);
        }
    }
    rels.sort();
    rels
}

/// Returns the bytes of a file within an embedded skill, addressed by a skill-relative
/// slash path, or `None` when absent.
pub fn read_skill_file(name: &str, rel: &str) -> Option<&'static [u8]> {
    let path = format!("{name}/{}", rel.replace('\\', "/"));
    SKILLS.get_file(&path).map(File::contents)
}

/// Canonical hash of an embedded skill tree (marker-free — repo source carries no
/// marker). See [`hash_tree`] for the exact algorithm. Infallible: the embedded set is
/// fixed at compile time.
pub fn embedded_tree_hash(name: &str) -> String {
    let rels = skill_files(name);
    hash_tree(&rels, |rel| read_skill_file(name, rel).map(|b| b.to_vec()))
}

/// Canonical hash of a deployed skill directory on disk. The marker line is stripped
/// from SKILL.md before hashing so a marked install hashes identically to its
/// marker-free embedded source (SPEC-EMBED-002).
///
/// Returns an [`AppError`] (exit 10, FileIO) if the directory cannot be walked or a
/// file cannot be read — this is the only IO-bearing hash path, so unlike Go's
/// `(string, error)` pair it surfaces as `AppResult` for the calling command.
pub fn deployed_tree_hash(dir: &Path) -> AppResult<String> {
    let mut rels = Vec::new();
    walk_disk(dir, dir, &mut rels)?;
    rels.sort();
    let hash = hash_tree(&rels, |rel| {
        let p = dir.join(rel.replace('/', std::path::MAIN_SEPARATOR_STR));
        std::fs::read(p).ok()
    });
    Ok(hash)
}

/// The canonical digest (SPEC-EMBED-002): sha256 over, for each file sorted by relative
/// path, the relative-path bytes then the file bytes — raw, with no line-ending or
/// trailing-newline normalization. SKILL.md has its marker line stripped first so an
/// embedded (marker-free) tree and a deployed (marked) tree hash identically.
fn hash_tree<F>(rels: &[String], read: F) -> String
where
    F: Fn(&str) -> Option<Vec<u8>>,
{
    let mut h = Sha256::new();
    for rel in rels {
        let mut data = read(rel).unwrap_or_default();
        if rel == "SKILL.md" {
            data = strip_marker_bytes(&data);
        }
        h.update(rel.as_bytes());
        h.update(&data);
    }
    hex::encode(h.finalize())
}

// --- Marker primitives (SPEC-EMBED-001) ---------------------------------------------

/// Removes the first anchored `<!-- naba-skills: ... -->` line and its single newline
/// terminator, restoring the embedded original byte-for-byte. Content with no marker is
/// returned unchanged. Byte-level twin of Go's `StripMarker`.
pub fn strip_marker_bytes(content: &[u8]) -> Vec<u8> {
    let lines = split_after_lf(content);
    for (i, line) in lines.iter().enumerate() {
        let s = String::from_utf8_lossy(line);
        let trimmed = s.trim_end_matches('\n');
        if trimmed.trim().starts_with(MARKER_PREFIX) && trimmed.ends_with("-->") {
            let mut out = Vec::new();
            for (j, l) in lines.iter().enumerate() {
                if j != i {
                    out.extend_from_slice(l);
                }
            }
            return out;
        }
    }
    content.to_vec()
}

/// String-facing [`strip_marker_bytes`]: strips the marker line from SKILL.md text.
pub fn strip_marker(content: &str) -> String {
    String::from_utf8_lossy(&strip_marker_bytes(content.as_bytes())).into_owned()
}

/// Builds the single-line integrity marker for a version and tree hash
/// (SPEC-EMBED-001). Mirrors Go's `FormatMarker`.
pub fn format_marker(version: &str, tree_hash: &str) -> String {
    format!("{MARKER_PREFIX} v={version} tree={tree_hash} -->")
}

/// Inserts the integrity marker immediately after the closing line of the YAML
/// frontmatter (so the frontmatter still parses); prepends it when no frontmatter is
/// present. Any existing marker is stripped first, making injection idempotent
/// (SPEC-EMBED-001). Byte-level twin of Go's `InjectMarker`.
pub fn inject_marker_bytes(content: &[u8], marker: &str) -> Vec<u8> {
    let content = strip_marker_bytes(content);
    let mut marker_line = marker.as_bytes().to_vec();
    marker_line.push(b'\n');

    let open = b"---\n";
    if content.starts_with(open) {
        let rest = &content[open.len()..];
        if let Some(idx) = find_subslice(rest, b"\n---\n") {
            let cut = open.len() + idx + b"\n---\n".len();
            let mut out = Vec::with_capacity(content.len() + marker_line.len());
            out.extend_from_slice(&content[..cut]);
            out.extend_from_slice(&marker_line);
            out.extend_from_slice(&content[cut..]);
            return out;
        }
    }
    marker_line.extend_from_slice(&content);
    marker_line
}

/// String-facing [`inject_marker_bytes`].
pub fn inject_marker(content: &str, marker: &str) -> String {
    String::from_utf8_lossy(&inject_marker_bytes(content.as_bytes(), marker)).into_owned()
}

/// Extracts the `tree=` hash from a deployed SKILL.md's marker line, or `None` when no
/// marker is present. Mirrors Go's `ParseMarkerHash`.
pub fn parse_marker_hash(content: &str) -> Option<String> {
    for line in split_after_lf(content.as_bytes()) {
        let s = String::from_utf8_lossy(line);
        let trimmed = s.trim_end_matches('\n').trim();
        if !trimmed.starts_with(MARKER_PREFIX) {
            continue;
        }
        for field in trimmed.split_whitespace() {
            if let Some(h) = field.strip_prefix("tree=") {
                return Some(h.to_string());
            }
        }
    }
    None
}

// --- Status computation (SPEC-EMBED-003) --------------------------------------------

/// Deployed-vs-embedded comparison flags for one skill (SPEC-EMBED-003). Consumed by
/// the `skills status` / `doctor` commands (Issue 4.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillStatus {
    /// `SKILL.md` present in the destination directory.
    pub installed: bool,
    /// Marker's `tree=` hash equals `embedded_tree_hash(name)`.
    pub up_to_date: bool,
    /// Every embedded file is present on disk.
    pub complete: bool,
    /// Recomputed `deployed_tree_hash(dest)` equals `embedded_tree_hash(name)`.
    pub unmodified: bool,
}

/// Computes [`SkillStatus`] for an embedded skill against its deployed directory
/// (SPEC-EMBED-003). `dest` is the skill's on-disk directory (the one containing
/// `SKILL.md`).
pub fn skill_status(name: &str, dest: &Path) -> SkillStatus {
    let embedded = embedded_tree_hash(name);

    let skill_md = dest.join("SKILL.md");
    let installed = skill_md.is_file();

    let up_to_date = std::fs::read_to_string(&skill_md)
        .ok()
        .and_then(|c| parse_marker_hash(&c))
        .map(|h| h == embedded)
        .unwrap_or(false);

    let complete = skill_files(name).iter().all(|rel| {
        dest.join(rel.replace('/', std::path::MAIN_SEPARATOR_STR))
            .is_file()
    });

    let unmodified = deployed_tree_hash(dest)
        .map(|h| h == embedded)
        .unwrap_or(false);

    SkillStatus {
        installed,
        up_to_date,
        complete,
        unmodified,
    }
}

// --- Internal helpers ---------------------------------------------------------------

/// A path component Go's `//go:embed` excludes: dotfile or underscore-prefixed.
fn is_excluded_component(c: &str) -> bool {
    c.starts_with('.') || c.starts_with('_')
}

/// Recursively collects every embedded `File` under `dir`.
fn collect_files<'a>(dir: &'a Dir<'a>, out: &mut Vec<&'a File<'a>>) {
    for entry in dir.entries() {
        match entry {
            DirEntry::File(f) => out.push(f),
            DirEntry::Dir(d) => collect_files(d, out),
        }
    }
}

/// Recursively collects skill-relative slash paths of the files under a disk directory.
/// Unlike [`skill_files`], no dotfile exclusion is applied — this mirrors Go's
/// `DeployedTreeHash`, which walks every file on disk.
fn walk_disk(root: &Path, dir: &Path, out: &mut Vec<String>) -> AppResult<()> {
    let entries = std::fs::read_dir(dir)
        .map_err(|e| AppError::file_io(format!("read {}: {e}", dir.display())))?;
    for entry in entries {
        let entry = entry.map_err(|e| AppError::file_io(format!("{e}")))?;
        let path = entry.path();
        let ft = entry
            .file_type()
            .map_err(|e| AppError::file_io(format!("{e}")))?;
        if ft.is_dir() {
            walk_disk(root, &path, out)?;
        } else {
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            out.push(rel);
        }
    }
    Ok(())
}

/// Go `bytes.SplitAfter(data, "\n")`: splits on `\n`, keeping the separator at the end
/// of each piece, with a trailing (possibly empty) final piece.
fn split_after_lf(data: &[u8]) -> Vec<&[u8]> {
    let mut out = Vec::new();
    let mut start = 0;
    for (i, b) in data.iter().enumerate() {
        if *b == b'\n' {
            out.push(&data[start..=i]);
            start = i + 1;
        }
    }
    out.push(&data[start..]);
    out
}

/// Index of the first occurrence of `needle` in `haystack`, or `None`.
fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || needle.len() > haystack.len() {
        return None;
    }
    haystack.windows(needle.len()).position(|w| w == needle)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Rust-content regression pin for the embedded `naba` skill tree.
    ///
    /// **No longer a Go-parity anchor.** Issue 4.0 achieved byte-for-byte parity with the Go
    /// binary's embed (hash `6dfa9939…d1cf`). Issue 5.2 then **intentionally updated the
    /// embedded skill content** (`skills/naba/SKILL.md`) to document the multi-provider
    /// surface (`--provider`, per-provider `--quality`, `--model requires --provider`). That
    /// content change re-hashes the tree, so this pin now guards the NEW Rust content — it is
    /// a content-regression pin, not a Go-parity assertion. Because the shipped content
    /// changed, existing on-disk installs read "outdated" until `naba skills upgrade`
    /// (Issue 5.3 must run a post-cutover upgrade).
    // plan-005 Issue C.3 again updated the embedded content (added the `## Preflight` section
    // wiring `naba skills preflight --json` at trigger time), re-hashing the tree. On-disk
    // installs read "outdated" until `naba skills upgrade` — surfaced by the very preflight this
    // change adds (skills axis).
    const NABA_TREE_HASH: &str = "d5b2fdfe452b2d803670dd781cbf92375f1600ddac6220831cec6aae7fe368c8";

    #[test]
    fn embedded_hash_matches_go_reference() {
        assert_eq!(embedded_tree_hash("naba"), NABA_TREE_HASH);
    }

    #[test]
    fn skill_names_sorted_and_present() {
        let names = skill_names();
        assert!(names.contains(&"naba".to_string()));
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted);
    }

    #[test]
    fn skill_files_sorted_and_have_skill_md() {
        let files = skill_files("naba");
        assert!(files.contains(&"SKILL.md".to_string()));
        assert!(files.contains(&"commands/edit.md".to_string()));
        let mut sorted = files.clone();
        sorted.sort();
        assert_eq!(files, sorted);
        // Go sort order: uppercase before lowercase (byte order) -> README/SKILL first.
        assert_eq!(files[0], "README.md");
        assert_eq!(files[1], "SKILL.md");
    }

    #[test]
    fn skill_files_exclude_dotfiles_and_underscores() {
        let files = skill_files("naba");
        for f in &files {
            for comp in f.split('/') {
                assert!(!comp.starts_with('.'), "dotfile leaked: {f}");
                assert!(!comp.starts_with('_'), "underscore file leaked: {f}");
            }
        }
    }

    #[test]
    fn repo_source_is_marker_free() {
        let content = read_skill_file("naba", "SKILL.md").unwrap();
        assert!(!contains(content, MARKER_PREFIX.as_bytes()));
    }

    #[test]
    fn hash_is_deterministic() {
        assert_eq!(embedded_tree_hash("naba"), embedded_tree_hash("naba"));
    }

    #[test]
    fn strip_marker_no_marker_is_noop() {
        let input = "no marker here\nline two\n";
        assert_eq!(strip_marker(input), input);
    }

    #[test]
    fn format_marker_shape() {
        let m = format_marker("9.9.9", "abc123");
        assert_eq!(m, "<!-- naba-skills: v=9.9.9 tree=abc123 -->");
    }

    #[test]
    fn marker_round_trip() {
        let orig =
            String::from_utf8(read_skill_file("naba", "SKILL.md").unwrap().to_vec()).unwrap();
        let hash = embedded_tree_hash("naba");
        let marker = format_marker("9.9.9", &hash);
        let injected = inject_marker(&orig, &marker);

        // Marker present, frontmatter intact.
        assert!(injected.contains(MARKER_PREFIX));
        assert!(injected.starts_with("---\n"));
        // Marker landed after the frontmatter close, not inside it.
        let rest = &injected["---\n".len()..];
        let fm_end = rest.find("\n---\n").unwrap();
        assert!(rest[..fm_end].contains("name: naba"));

        // strip(inject(x)) == x, byte-for-byte.
        assert_eq!(strip_marker(&injected), orig);
        // Parse round-trips the hash.
        assert_eq!(parse_marker_hash(&injected), Some(hash.clone()));

        // Idempotent: double inject leaves exactly one marker, strips back to original.
        let double = inject_marker(&injected, &format_marker("1.0.0", &hash));
        assert_eq!(double.matches(MARKER_PREFIX).count(), 1);
        assert_eq!(strip_marker(&double), orig);
    }

    #[test]
    fn inject_without_frontmatter_prepends() {
        let content = "# Title\n\nbody\n";
        let marker = format_marker("1.0.0", "deadbeef");
        let out = inject_marker(content, &marker);
        assert_eq!(out, format!("{marker}\n{content}"));
        assert_eq!(parse_marker_hash(&out), Some("deadbeef".to_string()));
    }

    #[test]
    fn deployed_marked_tree_hashes_as_embedded() {
        let embedded = embedded_tree_hash("naba");
        let dir = std::env::temp_dir().join(format!("naba-embed-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let marker = format_marker("1.2.3", &embedded);

        for rel in skill_files("naba") {
            let bytes = read_skill_file("naba", &rel).unwrap();
            let data = if rel == "SKILL.md" {
                inject_marker_bytes(bytes, &marker)
            } else {
                bytes.to_vec()
            };
            let p = dir.join(rel.replace('/', std::path::MAIN_SEPARATOR_STR));
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(&p, &data).unwrap();
        }

        // Deployed (marked) tree hashes identically to the embedded (marker-free) tree.
        assert_eq!(deployed_tree_hash(&dir).unwrap(), embedded);

        // Status: freshly installed marked tree is installed + up-to-date + complete +
        // unmodified (SPEC-EMBED-003 semantics).
        let status = skill_status("naba", &dir);
        assert_eq!(
            status,
            SkillStatus {
                installed: true,
                up_to_date: true,
                complete: true,
                unmodified: true,
            }
        );

        // Tamper a non-SKILL file -> hash diverges, unmodified becomes false.
        std::fs::write(dir.join("README.md"), b"changed").unwrap();
        assert_ne!(deployed_tree_hash(&dir).unwrap(), embedded);
        assert!(!skill_status("naba", &dir).unmodified);

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Helper mirroring Go's `bytes.Contains`.
    fn contains(haystack: &[u8], needle: &[u8]) -> bool {
        find_subslice(haystack, needle).is_some()
    }
}
