//! Harness-as-data: the `HarnessDescriptor` table that turns skills-install destination
//! resolution from a single uniform `.<surface>/skills` prefix into a per-harness,
//! per-scope idiomatic layout (plan-008, Issue 1.1 / SPEC-SKILLS harness-layout).
//!
//! Each supported agent harness (claude-code, opencode, pi, codex) — plus the portable
//! `agents` harness that writes the cross-harness `.agents/skills` home — is a **data row**
//! here, not a code branch. Adding a harness is a new row (and a matching SPEC row), never a
//! structural change. `resolve_dest` (in [`crate::skills`]) is layered on top of this table.
//!
//! All user-scope anchors are `$HOME` and all project-scope anchors are the git root (or cwd);
//! only the **subpath** differs per harness/scope. The descriptor keeps split
//! `user_subpath`/`project_subpath` because three of the four real harnesses diverge between
//! user and project scope (opencode `~/.config/opencode/skills` vs `.opencode/skills`; pi
//! `~/.pi/agent/skills` vs `.pi/skills`).

/// A single harness's idiomatic skills-install layout. The table is the source of truth both
/// the installer and the harness-layout SPEC assert against (Issue 4.2 descriptor↔SPEC check).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HarnessDescriptor {
    /// Stable harness id (the `--harness` value): `claude-code`, `opencode`, `pi`, `codex`,
    /// or the portable `agents`.
    pub id: &'static str,
    /// User-scope subpath, joined onto `$HOME` (e.g. `.claude/skills`,
    /// `.config/opencode/skills`).
    pub user_subpath: &'static str,
    /// Project-scope subpath, joined onto the git root / cwd (e.g. `.opencode/skills`).
    pub project_subpath: &'static str,
    /// Frontmatter keys every deployed `SKILL.md` must carry. Union-safe across all harnesses.
    pub frontmatter_required: &'static [&'static str],
    /// Optional skill-name normalization rule. `None` = verbatim; the string tag documents the
    /// constraint (inert for `naba`, which already complies) for future harnesses.
    pub name_transform: Option<&'static str>,
}

/// Frontmatter keys required by every harness (SKILL.md `name` + `description`).
const FRONTMATTER: &[&str] = &["name", "description"];

/// The canonical harness table (five rows). Order is stable; the descriptor↔SPEC check and the
/// per-harness path-assertion tests both iterate it.
pub const HARNESSES: &[HarnessDescriptor] = &[
    HarnessDescriptor {
        id: "claude-code",
        user_subpath: ".claude/skills",
        project_subpath: ".claude/skills",
        frontmatter_required: FRONTMATTER,
        name_transform: None,
    },
    HarnessDescriptor {
        id: "opencode",
        // opencode's user root is `~/.config/opencode`, not `~/.opencode` — this is the row
        // that forces split user/project subpaths.
        user_subpath: ".config/opencode/skills",
        project_subpath: ".opencode/skills",
        frontmatter_required: FRONTMATTER,
        name_transform: None,
    },
    HarnessDescriptor {
        id: "pi",
        user_subpath: ".pi/agent/skills",
        project_subpath: ".pi/skills",
        frontmatter_required: FRONTMATTER,
        name_transform: Some("lowercase-hyphen,max64"),
    },
    HarnessDescriptor {
        id: "codex",
        // codex's official home is the cross-harness `.agents/skills` (not `.codex/skills`,
        // which is unverified against OpenAI docs) — so it overlaps the portable `agents` row.
        user_subpath: ".agents/skills",
        project_subpath: ".agents/skills",
        frontmatter_required: FRONTMATTER,
        name_transform: None,
    },
    HarnessDescriptor {
        id: "agents",
        // Portable harness: a single `.agents/skills` write is read by opencode + pi + codex.
        user_subpath: ".agents/skills",
        project_subpath: ".agents/skills",
        frontmatter_required: FRONTMATTER,
        name_transform: None,
    },
];

/// The default harness when `--harness`/`--surface` is not given (claude-code, naba's origin).
pub const DEFAULT_HARNESS: &str = "claude-code";

/// Look up a canonical harness by id. Returns `None` for unknown ids.
pub fn lookup(id: &str) -> Option<&'static HarnessDescriptor> {
    HARNESSES.iter().find(|h| h.id == id)
}

/// Map a legacy `--surface` value to its `--harness` id. The two historically-shipped surfaces
/// are `claude` (→ `claude-code`) and `agents` (→ `agents`, unchanged). Any other value is
/// passed through verbatim so an arbitrary legacy `.<surface>/skills` install still resolves to
/// the same directory (see [`resolve_subpath`]).
pub fn surface_alias(surface: &str) -> String {
    match surface {
        "claude" => "claude-code".to_string(),
        other => other.to_string(),
    }
}

/// Resolve the scope-appropriate subpath for a harness id. A canonical harness uses its
/// descriptor row; an unrecognized id falls back to the legacy uniform `.<id>/skills` layout so
/// deprecated/unknown `--surface` values keep resolving to their historical directory
/// (backward-compatibility contract, plan-008 scope decision).
pub fn resolve_subpath(scope: &str, harness: &str) -> String {
    if let Some(d) = lookup(harness) {
        let sub = if scope == "project" {
            d.project_subpath
        } else {
            d.user_subpath
        };
        return sub.to_string();
    }
    // Legacy / unknown: preserve the old `.<surface>/skills` behavior.
    format!(".{harness}/skills")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_has_five_canonical_rows() {
        let ids: Vec<&str> = HARNESSES.iter().map(|h| h.id).collect();
        assert_eq!(ids, ["claude-code", "opencode", "pi", "codex", "agents"]);
    }

    #[test]
    fn lookup_finds_and_misses() {
        assert_eq!(lookup("opencode").unwrap().project_subpath, ".opencode/skills");
        assert!(lookup("nope").is_none());
    }

    #[test]
    fn surface_alias_maps_known_and_passes_through() {
        assert_eq!(surface_alias("claude"), "claude-code");
        assert_eq!(surface_alias("agents"), "agents");
        assert_eq!(surface_alias("weird"), "weird");
    }

    #[test]
    fn resolve_subpath_split_user_vs_project() {
        assert_eq!(resolve_subpath("user", "opencode"), ".config/opencode/skills");
        assert_eq!(resolve_subpath("project", "opencode"), ".opencode/skills");
        assert_eq!(resolve_subpath("user", "pi"), ".pi/agent/skills");
        assert_eq!(resolve_subpath("project", "pi"), ".pi/skills");
    }

    #[test]
    fn codex_and_agents_overlap_on_dot_agents() {
        assert_eq!(resolve_subpath("user", "codex"), ".agents/skills");
        assert_eq!(resolve_subpath("user", "agents"), ".agents/skills");
        assert_eq!(resolve_subpath("project", "codex"), ".agents/skills");
    }

    #[test]
    fn unknown_harness_falls_back_to_legacy_uniform_layout() {
        assert_eq!(resolve_subpath("user", "claude"), ".claude/skills");
        assert_eq!(resolve_subpath("project", "custom"), ".custom/skills");
    }
}
