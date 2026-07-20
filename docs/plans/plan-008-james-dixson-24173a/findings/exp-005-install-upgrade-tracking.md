# Finding E5: Install/upgrade tracking + `--surface`→`--harness` migration

**Date:** 2026-07-20
**Experiment:** E5
**Confidence:** HIGH (full read of the named modules + SPEC, code quoted with file:line).

## Headline

**naba has no skills install receipt today.** The only record that a skill was installed is the
per-`SKILL.md` integrity marker `<!-- naba-skills: v=<ver> tree=<hash> -->` (`src/embed.rs:38,
164-166`), injected after frontmatter (`src/skills.rs:240-243`). It records **only** version +
tree hash — **no surface/scope/harness**; that is implicit in the file's path, and nothing
enumerates paths. So "upgrade all previously-installed harnesses" requires a **new receipt**.

## Current behavior

- `skills upgrade` does **not** discover prior installs. `run(Mode::Upgrade)` resolves **one**
  dest from flags (`resolve_dest(scope, surface, target)`, `src/skills.rs:129`) and rewrites all
  embedded skills into it. With clap defaults (`scope=user`, `surface=claude`), an unqualified
  `naba skills upgrade` **only touches `$HOME/.claude/skills`** (`src/cli.rs:402-407`).
- The only pseudo-discovery is `post_update_skills_refresh` → `present_surfaces`
  (`src/self_cmd/update.rs:342-373`): a dir-exists heuristic over `$HOME/.claude` / `$HOME/.agents`,
  **user-scope only**, that fires `skills upgrade --surface <s>` per present dir. This is the seam
  to replace with receipt-driven enumeration.
- Surfaces in use today: only `claude` and `agents`.

## Design: target receipt

Introduce `<config_dir>/skills-install.json` (mirror the binary-installer receipt helpers,
`src/dirs.rs:96-109`) — a **set of install targets**, upsert-keyed on `(harness, scope, path)`:

```json
{
  "version": 1,
  "targets": [
    { "harness": "claude-code", "scope": "user",    "path": "/home/u/.claude/skills" },
    { "harness": "codex",       "scope": "user",    "path": "/home/u/.agents/skills" },
    { "harness": "claude-code", "scope": "project", "path": "/repo/.claude/skills", "anchor": "/repo" }
  ]
}
```

- Fields: `harness` (canonical), `scope`, `path` (absolute resolved dest), optional `anchor`
  (git root for project scope), optional `installed_version`/`updated_at` for diagnostics.
- **Decouples logical `harness` from physical `path`** — important (see migration).
- Enumeration rule for `upgrade`:
  - Explicit `--harness`/`--scope`/`--target` given → behave as today for that dest, **and** upsert it.
  - `--harness` absent → load receipt, run `deploy_skill(prune=true)` against each `target.path`;
    skip-and-report any target whose `path` no longer exists (moved/deleted repo).
  - `--harness` repeatable → resolve+dedupe+upsert each.
- `post_update_skills_refresh` collapses to a single `naba skills upgrade` (receipt authoritative);
  the SPEC-SELF-005 loop and the new unqualified upgrade share one "enumerate targets" fn.
- The per-`SKILL.md` marker is **unchanged** — it still drives freshness (`embed::skill_status`,
  `src/embed.rs:235-262`); the receipt is orthogonal/additive.

## `--surface`→`--harness` migration

**No legacy receipt to rewrite** (none exists). Migration is three parts:

1. **Flag alias.** Keep `--surface <v>` as a deprecated alias that maps its value through a
   name table then feeds `--harness`. Table: `claude → claude-code`, `agents → codex`(?) — see
   note. Threaded through `SkillsArgs`/`DoctorArgs`/`preflight::Opts` (`src/cli.rs:391,406-407`).
2. **Physical-layout decision (must pin before impl).**
   - **(b) relabel-only (RECOMMENDED):** claude-code keeps the physical `.claude/skills` dir; the
     rename is label-only in flags/receipt → **no file moves, old installs keep working**.
   - (a) physical relayout to `.claude-code/skills` → orphans the old tree, needs move/reinstall.
   Because the receipt stores logical `harness` **and** physical `path` separately, (b) is
   expressible now and (a) stays a deliberate future option. Note this is **consistent with E1–E4**:
   opencode/pi/codex get their own idiomatic physical subpaths; only the claude→claude-code
   rename is relabel-only on the *same* `.claude/skills` dir.
3. **Receipt synthesis (makes old installs upgrade cleanly).** On first post-rename
   `upgrade`/`install` (and lazily in `preflight`), if the receipt is absent, **synthesize it**
   by disk-scanning legacy locations (`$HOME/.claude`, `$HOME/.agents`, current git root) the way
   `present_surfaces` does, mapping each present dir to its canonical harness, writing one target
   per discovered install. Idempotent (union/dedupe).

**Preflight** (`src/preflight.rs:161`) must resolve via the receipt's targets, not only
`$HOME/.claude/skills`, or a user who installed to `agents` sees a false "not installed" gate.

## Open decision for E1–E4 reconciliation

E1–E4 maps codex → `.agents/skills`. Legacy naba `agents` surface also → `.agents/skills`. So the
old `agents` surface should map to **codex** (or a portable `.agents` pseudo-harness), **not**
a literal `agents` harness. Pin the `surface→harness` table in the harness SPEC:
`claude → claude-code`; `agents → codex` (or `agents → portable/.agents mode`). **Needs an
approach decision** (recorded as a risk/decision, resolved in PLAN).

## Idempotency + failure

- Marker injection already idempotent (`inject_marker_bytes` strips existing first,
  `src/embed.rs:172-173`; `marker_round_trip` test). Preserve.
- Receipt writes are **upserts**, keyed `(harness, scope, path)`; synthesis unions/dedupes.
- Multi-target upgrade loop must be **continue-on-error** (collect per-target outcomes into the
  existing `--json` envelope, `src/skills.rs:63-68`), non-zero exit if any failed — improving on
  SPEC-SELF-005's fail-fast (`src/self_cmd/update.rs:354-358`).
- Stale project `path` → skip-and-report, never recreate at a stale absolute path.
- Missing/corrupt receipt → fail-soft to legacy disk-scan synthesis, never panic.
- `--target` override installs (`src/skills.rs:93-94`) should also be recorded (`harness:"custom"`
  or resolved harness + explicit path) so unqualified upgrade reaches them.

## Files

`src/skills.rs` (resolve_dest 92-106, upgrade loop 128-149, deploy_skill+marker 208-271, prune
275-290); `src/embed.rs` (marker 38,164-214; skill_status 235-262); `src/preflight.rs` (161-171);
`src/dirs.rs` (receipt helpers 96-109); `src/self_cmd/update.rs` (present_surfaces 342-373);
`src/cli.rs` (surface defaults 391,406-407); `SPEC.md` (SPEC-SKILLS-003 294-296, SPEC-EMBED 724-726,
SPEC-SELF-005 857-861, SPEC-PREFLIGHT 869-884).
