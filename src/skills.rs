//! `skills` command group (SPEC-SKILLS-001..006, §3.11): install / upgrade / remove /
//! status of the binary-embedded skill trees. Ports Go's `internal/cli/skills.go`.
//!
//! The embed primitives (hashing, marker injection, status) live in [`crate::embed`]
//! (Issue 4.0); this module is the command behavior + message strings + destination
//! resolution layered on top. `resolve_dest` is shared with `doctor` (Issue 4.3).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::embed;
use crate::error::{AppError, AppResult};
use crate::output;
use crate::version;

/// One of the three write/remove deployment modes driven by [`run`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Install,
    Upgrade,
    Remove,
}

impl Mode {
    /// The action verb for the `--json` envelope (SPEC-JSON-006).
    fn action(self) -> &'static str {
        match self {
            Mode::Install => "install",
            Mode::Upgrade => "upgrade",
            Mode::Remove => "remove",
        }
    }
}

/// Resolved flags a `skills` invocation carries (mirrors Go's package-level `skills*` vars).
#[derive(Debug, Clone)]
pub struct Opts {
    pub scope: String,
    pub surface: String,
    pub target: String,
    pub dry_run: bool,
    pub quiet: bool,
    /// Emit the universal `--json` envelope (SPEC-JSON-006) instead of the human lines.
    pub json: bool,
}

/// One `install`/`upgrade`/`remove` outcome row for the `--json` envelope (SPEC-JSON-006).
#[derive(Debug, Clone, serde::Serialize)]
struct SkillActionItem {
    name: String,
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    files: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    removed: Option<bool>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pruned: Vec<String>,
}

/// The `install`/`upgrade`/`remove` `--json` payload (SPEC-JSON-006).
#[derive(Debug, Clone, serde::Serialize)]
struct SkillsActionReport {
    action: &'static str,
    destination: String,
    dry_run: bool,
    skills: Vec<SkillActionItem>,
}

/// One `status` row for the `--json` envelope (SPEC-JSON-006).
#[derive(Debug, Clone, serde::Serialize)]
struct SkillStatusItem {
    name: String,
    path: String,
    installed: bool,
    up_to_date: bool,
    complete: bool,
    unmodified: bool,
}

/// The `status` `--json` payload (SPEC-JSON-006).
#[derive(Debug, Clone, serde::Serialize)]
struct SkillsStatusReport {
    destination: String,
    skills: Vec<SkillStatusItem>,
}

/// `resolve_dest` mirrors the legacy installer's destination resolution: an explicit
/// `target` wins; otherwise the anchor is `$HOME` (user scope) or the git root / cwd
/// (project scope), joined with `.<surface>/skills` (SPEC-SKILLS-003). Shared by
/// `naba skills` and `naba doctor`.
pub fn resolve_dest(scope: &str, surface: &str, target: &str) -> AppResult<PathBuf> {
    if !target.is_empty() {
        return Ok(PathBuf::from(target));
    }
    let anchor = if scope == "project" {
        git_root_or_cwd()
    } else {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .filter(|p| !p.as_os_str().is_empty())
            .ok_or_else(|| AppError::general("$HOME is not defined"))?;
        home
    };
    Ok(anchor.join(format!(".{surface}")).join("skills"))
}

/// Git repository root (`git rev-parse --show-toplevel`), falling back to the current
/// working directory when not in a repo (matches Go's `gitRootOrCwd`).
fn git_root_or_cwd() -> PathBuf {
    if let Ok(out) = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
    {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !s.is_empty() {
                return PathBuf::from(s);
            }
        }
    }
    std::env::current_dir().unwrap_or_default()
}

/// Run install / upgrade / remove over every embedded skill (Go's `runSkills`). Under `--json`
/// (incl. the piped auto-enable) emits the universal envelope (SPEC-JSON-006) instead of the
/// human lines.
pub fn run(mode: Mode, opts: &Opts) -> AppResult<()> {
    let dest = resolve_dest(&opts.scope, &opts.surface, &opts.target)?;
    let mut items = Vec::new();
    for name in embed::skill_names() {
        let item = match mode {
            Mode::Remove => remove_skill(&name, &dest, opts)?,
            _ => deploy_skill(&name, &dest, mode == Mode::Upgrade, opts)?,
        };
        items.push(item);
    }
    if opts.json {
        output::print_ok_json(SkillsActionReport {
            action: mode.action(),
            destination: dest.display().to_string(),
            dry_run: opts.dry_run,
            skills: items,
        });
    } else if !opts.dry_run && !opts.quiet {
        println!("Destination: {}", dest.display());
    }
    Ok(())
}

/// `skills status`: print a one-line status per embedded skill (Go's `skillsStatusCmd`), or the
/// universal envelope under `--json` (SPEC-JSON-006).
pub fn status(opts: &Opts) -> AppResult<()> {
    let dest = resolve_dest(&opts.scope, &opts.surface, &opts.target)?;
    let mut items = Vec::new();
    for name in embed::skill_names() {
        let st = embed::skill_status(&name, &dest.join(&name));
        if opts.json {
            items.push(SkillStatusItem {
                name: name.clone(),
                path: dest.join(&name).display().to_string(),
                installed: st.installed,
                up_to_date: st.up_to_date,
                complete: st.complete,
                unmodified: st.unmodified,
            });
        } else {
            println!("{}", status_line(&name, &st, &dest));
        }
    }
    if opts.json {
        output::print_ok_json(SkillsStatusReport {
            destination: dest.display().to_string(),
            skills: items,
        });
    }
    Ok(())
}

/// One-line human status (Go's `SkillStatusResult.Line`). Not installed →
/// `<name>: not installed (<path>)`; else `<name>: <flags> (<path>)`.
fn status_line(name: &str, st: &embed::SkillStatus, dest: &Path) -> String {
    let path = dest.join(name);
    if !st.installed {
        return format!("{name}: not installed ({})", path.display());
    }
    let flags = format!(
        "{} {} {}",
        bool_flag("up-to-date", st.up_to_date),
        bool_flag("complete", st.complete),
        bool_flag("unmodified", st.unmodified),
    );
    format!("{name}: {flags} ({})", path.display())
}

fn bool_flag(label: &str, ok: bool) -> String {
    if ok {
        format!("\u{2713}{label}") // ✓
    } else {
        format!("\u{2717}{label}") // ✗
    }
}

/// Write an embedded skill's tree to `<dest>/<name>/`, injecting a fresh integrity marker
/// into SKILL.md. With `prune=true` (upgrade) it also removes dest files absent from the
/// embed (rsync --delete parity). Ports Go's `deploySkill`. Returns the outcome row for the
/// `--json` envelope; human lines print only when not `--json`.
fn deploy_skill(name: &str, dest: &Path, prune: bool, opts: &Opts) -> AppResult<SkillActionItem> {
    let dest_dir = dest.join(name);
    let rels = embed::skill_files(name);
    let hash = embed::embedded_tree_hash(name);
    let marker = embed::format_marker(version::VERSION, &hash);

    if opts.dry_run {
        if !opts.json {
            println!(
                "(dry run) would write {} file(s) of {:?} -> {}",
                rels.len(),
                name,
                dest_dir.display()
            );
            if prune {
                println!("(dry run) would prune dest files absent from the embed");
            }
        }
        return Ok(SkillActionItem {
            name: name.to_string(),
            path: dest_dir.display().to_string(),
            files: Some(rels.len()),
            removed: None,
            pruned: Vec::new(),
        });
    }

    for rel in &rels {
        let bytes = embed::read_skill_file(name, rel)
            .ok_or_else(|| AppError::file_io(format!("embedded file missing: {name}/{rel}")))?;
        let data = if rel == "SKILL.md" {
            embed::inject_marker_bytes(bytes, &marker)
        } else {
            bytes.to_vec()
        };
        let path = dest_dir.join(rel.replace('/', std::path::MAIN_SEPARATOR_STR));
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| AppError::file_io(e.to_string()))?;
            set_mode(parent, 0o755);
        }
        fs::write(&path, &data).map_err(|e| AppError::file_io(e.to_string()))?;
        set_mode(&path, 0o644);
    }

    let pruned = if prune {
        prune_stale(name, &dest_dir, opts)?
    } else {
        Vec::new()
    };
    if !opts.json && !opts.quiet {
        println!(
            "OK: {name} -> {} ({} files)",
            dest_dir.display(),
            rels.len()
        );
    }
    Ok(SkillActionItem {
        name: name.to_string(),
        path: dest_dir.display().to_string(),
        files: Some(rels.len()),
        removed: None,
        pruned,
    })
}

/// Remove files under `dest_dir` that are not part of the embedded skill tree (Go's
/// `pruneStale`). Returns the skill-relative paths pruned.
fn prune_stale(name: &str, dest_dir: &Path, opts: &Opts) -> AppResult<Vec<String>> {
    let want: std::collections::HashSet<String> = embed::skill_files(name).into_iter().collect();
    let mut on_disk = Vec::new();
    walk_files(dest_dir, dest_dir, &mut on_disk)?;
    let mut pruned = Vec::new();
    for (path, rel) in on_disk {
        if !want.contains(&rel) {
            fs::remove_file(&path).map_err(|e| AppError::file_io(e.to_string()))?;
            if !opts.json && !opts.quiet {
                println!("  pruned stale: {rel}");
            }
            pruned.push(rel);
        }
    }
    Ok(pruned)
}

/// Recursively collect `(absolute path, skill-relative slash path)` of files under `dir`.
fn walk_files(root: &Path, dir: &Path, out: &mut Vec<(PathBuf, String)>) -> AppResult<()> {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(AppError::file_io(e.to_string())),
    };
    for entry in entries {
        let entry = entry.map_err(|e| AppError::file_io(e.to_string()))?;
        let path = entry.path();
        let ft = entry
            .file_type()
            .map_err(|e| AppError::file_io(e.to_string()))?;
        if ft.is_dir() {
            walk_files(root, &path, out)?;
        } else {
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            out.push((path, rel));
        }
    }
    Ok(())
}

/// Remove an installed skill directory (Go's `removeSkill`). Absent → `absent: <dir>`;
/// dry-run → `(dry run) would remove <dir>`; else recursive delete + `removed: <dir>`. Returns
/// the outcome row for the `--json` envelope (`removed` = whether the tree was actually deleted).
fn remove_skill(name: &str, dest: &Path, opts: &Opts) -> AppResult<SkillActionItem> {
    let dest_dir = dest.join(name);
    let item = |removed: bool| SkillActionItem {
        name: name.to_string(),
        path: dest_dir.display().to_string(),
        files: None,
        removed: Some(removed),
        pruned: Vec::new(),
    };
    if !dest_dir.exists() {
        if !opts.json && !opts.quiet {
            println!("absent: {}", dest_dir.display());
        }
        return Ok(item(false));
    }
    if opts.dry_run {
        if !opts.json {
            println!("(dry run) would remove {}", dest_dir.display());
        }
        return Ok(item(false));
    }
    fs::remove_dir_all(&dest_dir).map_err(|e| AppError::file_io(e.to_string()))?;
    if !opts.json && !opts.quiet {
        println!("removed: {}", dest_dir.display());
    }
    Ok(item(true))
}

/// Set unix file mode (no-op semantics elsewhere). Mirrors Go's explicit `0o644`/`0o755`.
#[cfg(unix)]
fn set_mode(path: &Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;
    let _ = fs::set_permissions(path, fs::Permissions::from_mode(mode));
}

#[cfg(not(unix))]
fn set_mode(_path: &Path, _mode: u32) {}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts(target: &Path) -> Opts {
        Opts {
            scope: "user".into(),
            surface: "claude".into(),
            target: target.to_string_lossy().into_owned(),
            dry_run: false,
            quiet: false,
            json: false,
        }
    }

    #[test]
    fn resolve_dest_target_wins() {
        let d = resolve_dest("user", "claude", "/tmp/explicit").unwrap();
        assert_eq!(d, PathBuf::from("/tmp/explicit"));
    }

    #[test]
    fn resolve_dest_user_scope_joins_surface_skills() {
        // With an explicit HOME the user anchor is $HOME/.<surface>/skills.
        let prev = std::env::var_os("HOME");
        std::env::set_var("HOME", "/home/tester");
        let d = resolve_dest("user", "agents", "").unwrap();
        assert_eq!(d, PathBuf::from("/home/tester/.agents/skills"));
        match prev {
            Some(v) => std::env::set_var("HOME", v),
            None => std::env::remove_var("HOME"),
        }
    }

    #[test]
    fn install_then_status_round_trip() {
        let dir = std::env::temp_dir().join(format!("naba-skills-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let o = opts(&dir);

        run(Mode::Install, &o).unwrap();

        // SKILL.md exists and carries an injected marker.
        let skill_md = dir.join("naba").join("SKILL.md");
        assert!(skill_md.is_file());
        let content = fs::read_to_string(&skill_md).unwrap();
        assert!(content.contains(embed::MARKER_PREFIX));

        // status: fresh install is up-to-date/complete/unmodified.
        let st = embed::skill_status("naba", &dir.join("naba"));
        assert!(st.installed && st.up_to_date && st.complete && st.unmodified);
        let line = status_line("naba", &st, &dir);
        assert!(line.contains("\u{2713}up-to-date"));
        assert!(line.contains("\u{2713}unmodified"));

        // remove clears the tree.
        run(Mode::Remove, &o).unwrap();
        assert!(!dir.join("naba").exists());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn upgrade_prunes_stale_files() {
        let dir = std::env::temp_dir().join(format!("naba-skills-prune-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let o = opts(&dir);

        run(Mode::Install, &o).unwrap();
        // Plant a stale file not present in the embed.
        let stale = dir.join("naba").join("stale.txt");
        fs::write(&stale, b"junk").unwrap();
        assert!(stale.is_file());

        run(Mode::Upgrade, &o).unwrap();
        assert!(!stale.exists(), "upgrade should prune stale files");

        let _ = fs::remove_dir_all(&dir);
    }
}
