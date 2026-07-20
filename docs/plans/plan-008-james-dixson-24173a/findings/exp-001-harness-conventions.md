# Finding E1–E4: Idiomatic skills-install layout per harness

**Date:** 2026-07-20
**Experiments:** E1 (claude-code baseline), E2 (opencode), E3 (pi), E4 (codex)
**Confidence:** claude-code/opencode/pi HIGH (official docs); codex `.agents/skills` HIGH,
`.codex/skills` LOW (third-party only, unconfirmed against OpenAI docs).

## Headline

**All four harnesses have a first-class `SKILL.md` / directory-per-skill concept**, and the
install *unit* (`skills/naba/SKILL.md` + `commands/`) plus required frontmatter
(`name`, `description`) is **identical across all four**. Only the **anchor + subpath** differ,
so naba needs **no per-harness content transform** — only path data. opencode, pi, and codex
additionally honor the cross-harness `.agents/skills/` convention; only claude-code does not read
`.agents/`.

## Per-harness paths

| Harness | User-scope path | Project-scope path | Notes |
|:--------|:----------------|:-------------------|:------|
| **claude-code** | `~/.claude/skills/<s>/SKILL.md` | `.claude/skills/<s>/SKILL.md` | Canonical origin; naba's current layout is correct. |
| **opencode** | `~/.config/opencode/skills/` (also reads `~/.claude/`, `~/.agents/`) | `.opencode/skills/` (also reads `.claude/`, `.agents/`) | User root is `~/.config/opencode`, **not** `~/.opencode` — breaks the uniform `$HOME/.<id>` rule. |
| **pi** (pi.dev) | `~/.pi/agent/skills/` (also `~/.agents/`) | `.pi/skills/` (also `.agents/`) | Frontmatter `name` must be lowercase `a-z0-9-`, ≤64 chars ("naba" already complies). |
| **codex** | `$HOME/.agents/skills/` | `$CWD/.agents/skills/` + `$REPO_ROOT/.agents/skills/` | **Official path is `.agents/skills`, not `.codex/skills`** (unverified). AGENTS.md is an orthogonal instruction layer. |

**Cross-harness convergence:** `.agents/skills/` (project) + `~/.agents/skills/` (user) is
honored by opencode + pi + codex, and is codex's official home. A single `.agents/skills` write
satisfies three harnesses; only claude-code needs its own `.claude/skills`.

## Harness descriptor shape (data, not code)

The uniform `resolve_dest(scope, surface, target) → <anchor>/.<surface>/skills` rule holds
**only for claude-code**. The descriptor must therefore carry **split** user/project subpaths
and anchors that need not be `$HOME`/`$REPO_ROOT`+`.<id>`:

```
HarnessDescriptor {
  id:                   "claude-code" | "opencode" | "pi" | "codex"
  user_anchor:          path-template   # e.g. "$HOME"
  user_subpath:         string          # e.g. ".claude/skills" | ".config/opencode/skills" | ".pi/agent/skills" | ".agents/skills"
  project_anchor:       path-template   # e.g. "$REPO_ROOT"
  project_subpath:      string          # e.g. ".claude/skills" | ".opencode/skills" | ".pi/skills" | ".agents/skills"
  skill_layout:         "dir-per-skill" # constant across all four today
  manifest_file:        "SKILL.md"      # constant
  frontmatter_required: ["name","description"]  # union-safe across all four
  name_transform:       null | "lowercase-hyphen,max64"  # inert for "naba"; future-proofing
}
```

| id | user_subpath | project_subpath | name_transform |
|:---|:-------------|:----------------|:---------------|
| claude-code | `.claude/skills` | `.claude/skills` | null |
| opencode | `.config/opencode/skills` | `.opencode/skills` | null |
| pi | `.pi/agent/skills` | `.pi/skills` | lowercase-hyphen,≤64 |
| codex | `.agents/skills` | `.agents/skills` | null |

## Implications for the plan

- The rename is a **real semantic change**, not a string swap: `resolve_dest` must gain
  **separate `user_subpath`/`project_subpath`** and a per-harness **user_anchor** (opencode's
  `~/.config/opencode` forces this). The current one-`subpath` rule cannot express three of four.
- **No content templating is required for the harness axis** — the SKILL.md unit is portable.
  (The CLI-vs-MCP dual-purpose axis, E6, is a separate question.)
- Model harness as a **data table** (four rows) → adding a harness = a data/SPEC change,
  matching the extensibility goal. The harness-layout SPEC (objective #4) is essentially this
  descriptor table + the discovery/scope rules.
- Consider an optional **portable `.agents/skills` mode** covering opencode+pi+codex in one write.
- **Do not add a `.codex/skills` row** without confirming against OpenAI's own docs — the
  official path is `.agents/skills`.

## Sources

- opencode: opencode.ai/docs/skills, /docs/config, /docs/plugins, /docs/commands
- pi: github.com/earendil-works/pi `packages/coding-agent/docs/skills.md`; pi.dev/docs
- codex: learn.chatgpt.com/docs/build-skills; developers.openai.com/codex/guides/agents-md
- claude-code: established baseline (naba's current impl)
