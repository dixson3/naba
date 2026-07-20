//! The skills-install **target registry** (`<config_dir>/skills-install.json`, plan-008 Epic 2).
//!
//! `naba skills install` can now write to several idiomatic per-harness destinations
//! (Issue 1.3 `resolve_dest`). To let an unqualified `naba skills upgrade` re-hit **every**
//! previously-installed target without re-specifying flags, each install **upserts** a
//! `Target { harness, scope, path }` row here (Issue 2.2); `upgrade` enumerates the rows
//! (Issue 2.3), and migration synthesizes the first registry from a legacy disk scan
//! (Issue 2.4).
//!
//! This registry is naba's own artifact — distinct from the cargo-dist `naba-receipt.json`
//! (which the vendor installer owns; see [`crate::self_cmd::receipt`]). Writes are atomic
//! (temp + rename); a missing file reads as an empty registry.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::dirs;
use crate::error::{AppError, AppResult};

/// Current on-disk schema version. Bump only on a breaking layout change.
pub const SCHEMA_VERSION: u32 = 1;

/// One recorded install destination. The upsert key is the whole triple
/// `(harness, scope, path)` — the resolved absolute `path` disambiguates harnesses that share a
/// directory (e.g. `codex` and the portable `agents` both resolve to `.agents/skills`), while
/// keeping `harness`/`scope` for reporting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Target {
    /// The harness id this target was installed for (`claude-code`, `opencode`, …, `agents`).
    pub harness: String,
    /// The scope the install used (`user` or `project`).
    pub scope: String,
    /// The resolved absolute destination directory the skills were written to.
    pub path: String,
}

impl Target {
    pub fn new(
        harness: impl Into<String>,
        scope: impl Into<String>,
        path: impl Into<String>,
    ) -> Self {
        Target {
            harness: harness.into(),
            scope: scope.into(),
            path: path.into(),
        }
    }

    /// The upsert identity: `(harness, scope, path)`.
    fn key(&self) -> (&str, &str, &str) {
        (&self.harness, &self.scope, &self.path)
    }
}

/// The skills-install registry document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registry {
    /// Schema version (`SCHEMA_VERSION`). `#[serde(default)]` tolerates an older/missing field.
    #[serde(default)]
    pub version: u32,
    /// Recorded install targets, in insertion order.
    #[serde(default)]
    pub targets: Vec<Target>,
}

impl Default for Registry {
    fn default() -> Self {
        Registry {
            version: SCHEMA_VERSION,
            targets: Vec::new(),
        }
    }
}

impl Registry {
    /// Load the registry from the default path ([`crate::dirs::skills_install_path`]). A missing
    /// file is an **empty** registry (not an error); a malformed file surfaces as [`AppError`].
    pub fn load() -> AppResult<Registry> {
        Self::load_from(&dirs::skills_install_path())
    }

    /// Load from an explicit path (test seam). Missing → empty registry.
    pub fn load_from(path: &Path) -> AppResult<Registry> {
        match std::fs::read(path) {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .map_err(|e| AppError::general(format!("parse skills-install registry: {e}"))),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Registry::default()),
            Err(e) => Err(AppError::file_io(format!(
                "read skills-install registry: {e}"
            ))),
        }
    }

    /// Insert or update a target by its `(harness, scope, path)` key. Returns `true` if a new
    /// row was added, `false` if an existing row was matched (idempotent — same triple twice is
    /// a no-op). Insertion order is preserved.
    pub fn upsert(&mut self, target: Target) -> bool {
        if let Some(existing) = self.targets.iter_mut().find(|t| t.key() == target.key()) {
            *existing = target;
            false
        } else {
            self.targets.push(target);
            true
        }
    }

    /// Remove a target by its `(harness, scope, path)` key. Returns `true` if a row was removed.
    pub fn remove(&mut self, harness: &str, scope: &str, path: &str) -> bool {
        let before = self.targets.len();
        self.targets.retain(|t| t.key() != (harness, scope, path));
        self.targets.len() != before
    }

    /// Save to the default path atomically ([`crate::dirs::skills_install_path`]).
    pub fn save(&self) -> AppResult<()> {
        self.save_to(&dirs::skills_install_path())
    }

    /// Save to an explicit path, atomically (temp + rename). Creates the parent dir. Normalizes
    /// `version` to `SCHEMA_VERSION` on write.
    pub fn save_to(&self, path: &Path) -> AppResult<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                AppError::file_io(format!("mkdir for skills-install registry: {e}"))
            })?;
        }
        let mut doc = self.clone();
        doc.version = SCHEMA_VERSION;
        let data = serde_json::to_string_pretty(&doc)
            .map_err(|e| AppError::general(format!("serialize skills-install registry: {e}")))?;
        let tmp = tmp_sibling(path);
        std::fs::write(&tmp, data.as_bytes())
            .map_err(|e| AppError::file_io(format!("write skills-install tmp: {e}")))?;
        std::fs::rename(&tmp, path).map_err(|e| {
            let _ = std::fs::remove_file(&tmp);
            AppError::file_io(format!("rename skills-install registry: {e}"))
        })?;
        Ok(())
    }
}

/// A `<path>.tmp-<pid>` sibling for the atomic-rename write.
fn tmp_sibling(path: &Path) -> PathBuf {
    let mut s = path.as_os_str().to_owned();
    s.push(format!(".tmp-{}", std::process::id()));
    PathBuf::from(s)
}

// ---------------------------------------------------------------------------
// Migration: synthesize the first registry from a legacy disk scan (Issue 2.4)
// ---------------------------------------------------------------------------

/// The two historically-shipped surface dirs and the harness id each maps to. The pre-plan
/// installer only ever wrote `.claude/skills` (surface `claude` → `claude-code`) and
/// `.agents/skills` (surface `agents` → portable `agents`).
const LEGACY_SURFACES: &[(&str, &str)] = &[
    ("claude-code", ".claude/skills"),
    ("agents", ".agents/skills"),
];

/// Legacy install candidates to probe, at both scopes: `(harness, scope, dir)`.
fn legacy_candidates(home: &Path, git_root: Option<&Path>) -> Vec<(String, String, PathBuf)> {
    let mut out = Vec::new();
    for (harness, sub) in LEGACY_SURFACES {
        out.push((harness.to_string(), "user".to_string(), home.join(sub)));
    }
    if let Some(root) = git_root {
        for (harness, sub) in LEGACY_SURFACES {
            out.push((harness.to_string(), "project".to_string(), root.join(sub)));
        }
    }
    out
}

/// True if `dir` holds an installed naba skill (any embedded skill present as a
/// `<dir>/<name>/SKILL.md`). Cheap existence probe — no hashing.
fn has_installed_skill(dir: &Path) -> bool {
    crate::embed::skill_names()
        .iter()
        .any(|name| dir.join(name).join("SKILL.md").is_file())
}

impl Registry {
    /// Synthesize registry rows from a legacy on-disk scan (Issue 2.4). For each legacy surface
    /// dir that actually contains an installed skill and is not already recorded, upsert a
    /// target. Returns the number of rows added. Idempotent — a second call adds nothing.
    pub fn synthesize_from_legacy(&mut self, home: &Path, git_root: Option<&Path>) -> usize {
        let mut added = 0;
        for (harness, scope, dir) in legacy_candidates(home, git_root) {
            if has_installed_skill(&dir) {
                let path = dir.display().to_string();
                if self.upsert(Target::new(harness, scope, path)) {
                    added += 1;
                }
            }
        }
        added
    }
}

/// Load the registry and, **if it is empty**, synthesize it from a legacy disk scan of the real
/// `$HOME` + git root, persisting any rows found (Issue 2.4). A non-empty registry is returned
/// untouched. This is the migration seam `upgrade`/`preflight` drive off (Issue 2.5).
pub fn load_or_migrate() -> AppResult<Registry> {
    let mut reg = Registry::load()?;
    if reg.targets.is_empty() {
        let home = std::env::var_os("HOME").map(PathBuf::from);
        if let Some(home) = home {
            let git_root = git_root();
            let added = reg.synthesize_from_legacy(&home, git_root.as_deref());
            if added > 0 {
                reg.save()?;
            }
        }
    }
    Ok(reg)
}

/// Git repository root (`git rev-parse --show-toplevel`), or `None` when not in a repo.
fn git_root() -> Option<PathBuf> {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (!s.is_empty()).then(|| PathBuf::from(s))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_path(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "naba-skills-install-{}-{}.json",
            tag,
            std::process::id()
        ))
    }

    #[test]
    fn missing_file_loads_empty() {
        let p = tmp_path("missing");
        let _ = std::fs::remove_file(&p);
        let reg = Registry::load_from(&p).unwrap();
        assert_eq!(reg.targets.len(), 0);
    }

    #[test]
    fn upsert_is_idempotent_on_key() {
        let mut reg = Registry::default();
        assert!(reg.upsert(Target::new("claude-code", "user", "/h/.claude/skills")));
        // Same triple again → no new row.
        assert!(!reg.upsert(Target::new("claude-code", "user", "/h/.claude/skills")));
        assert_eq!(reg.targets.len(), 1);
        // Different harness at the same path → distinct row (codex vs agents overlap case).
        assert!(reg.upsert(Target::new("agents", "user", "/h/.agents/skills")));
        assert!(reg.upsert(Target::new("codex", "user", "/h/.agents/skills")));
        assert_eq!(reg.targets.len(), 3);
    }

    #[test]
    fn save_then_load_round_trips_and_normalizes_version() {
        let p = tmp_path("roundtrip");
        let _ = std::fs::remove_file(&p);
        let mut reg = Registry::default();
        reg.upsert(Target::new("opencode", "project", "/repo/.opencode/skills"));
        reg.save_to(&p).unwrap();

        let loaded = Registry::load_from(&p).unwrap();
        assert_eq!(loaded.version, SCHEMA_VERSION);
        assert_eq!(loaded.targets, reg.targets);

        // Atomic write leaves no temp sibling.
        let dir = p.parent().unwrap();
        let leftovers: Vec<_> = std::fs::read_dir(dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .contains("skills-install-roundtrip")
                    && e.file_name().to_string_lossy().contains(".tmp-")
            })
            .collect();
        assert!(leftovers.is_empty(), "atomic write left a temp file");
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn remove_by_key() {
        let mut reg = Registry::default();
        reg.upsert(Target::new("pi", "user", "/h/.pi/agent/skills"));
        assert!(reg.remove("pi", "user", "/h/.pi/agent/skills"));
        assert!(!reg.remove("pi", "user", "/h/.pi/agent/skills"));
        assert_eq!(reg.targets.len(), 0);
    }

    #[test]
    fn synthesize_from_legacy_maps_surfaces_and_is_idempotent() {
        let root = std::env::temp_dir().join(format!("naba-migrate-{}", std::process::id()));
        let home = root.join("home");
        let repo = root.join("repo");
        let _ = std::fs::remove_dir_all(&root);
        // Plant a legacy user-scope .claude install and a project-scope .agents install.
        for dir in [
            home.join(".claude/skills/naba"),
            repo.join(".agents/skills/naba"),
        ] {
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join("SKILL.md"), b"---\nname: naba\n---\n").unwrap();
        }
        // A legacy .agents user dir with NO installed skill must not be recorded.
        std::fs::create_dir_all(home.join(".agents/skills")).unwrap();

        let mut reg = Registry::default();
        let added = reg.synthesize_from_legacy(&home, Some(&repo));
        assert_eq!(added, 2, "claude-code(user) + agents(project)");
        let mapped: Vec<(&str, &str)> = reg
            .targets
            .iter()
            .map(|t| (t.harness.as_str(), t.scope.as_str()))
            .collect();
        assert!(mapped.contains(&("claude-code", "user")));
        assert!(mapped.contains(&("agents", "project")));

        // Idempotent: a second scan adds nothing.
        assert_eq!(reg.synthesize_from_legacy(&home, Some(&repo)), 0);
        assert_eq!(reg.targets.len(), 2);

        let _ = std::fs::remove_dir_all(&root);
    }
}
