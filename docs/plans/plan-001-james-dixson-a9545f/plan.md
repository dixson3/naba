# Plan: Modernize naba skill packaging: align to skill-authoring conventions, replace claude-plugin with install.{sh,py}, drift-check, and local-only beads with GitHub upstream

**ID:** plan-001-james-dixson-a9545f
**Author:** james-dixson
**Created:** 2026-06-07
**Status:** approved
**Epic:** naba-mol-9zz
**Phase log:**
- 2026-06-07 scoping: initial scope captured
- 2026-06-07 investigating: repo + reference (beads-skills) surveyed inline; no worktree experiments required
- 2026-06-07 drafting: plan v1 synthesized
- 2026-06-07 review: plan v1 presented
- 2026-06-07 approved: operator approved
- 2026-06-07 intake: epic naba-mol-9zz poured

## Objective

Modernize how the naba repository packages and ships its Claude-facing assets (skills, agents, rules) and bring its beads configuration into a clean, portable, local-only-with-GitHub-upstream state. Concretely:

1. Align every skill under `skills/` with the `skill-authoring` conventions (YAML frontmatter, token-efficient bodies, per-skill README) and the project-root instruction files with `optimal-instructions`.
2. Eliminate the Claude Code plugin (`.claude-plugin/plugin.json`, `marketplace.json`), delete the redundant top-level `agents/`, rename all skills with a `naba-` prefix, and replace plugin delivery with an `install.{sh,py}` modeled on `~/workspace/dixson3/beads-skills`.
3. Bootstrap and run `drift-check` against the new structure so the source-of-truth edges (skill frontmatter ↔ README ↔ installer ↔ project docs) provably agree.
4. Confirm and lock the beads configuration to local-only Dolt (no Dolt remote) with `.beads/issues.jsonl` tracked in git, and wire upstream issue tracking to the GitHub origin (`dixson3/naba`).

## Motivation

The naba repo currently ships its Claude assets through a Claude-Code-specific marketplace plugin (`.claude-plugin/`), with prompt-engineering guidance duplicated across nine skills *and* two top-level agent files (`agents/naba_image_assistant.md`, `agents/naba_batch_processor.md`). A prior commit (`8b3a1db`) already removed the old SessionStart "preflight" hook + `rules/`/`scripts/` symlink-delivery system and inlined rules into skills/agents, leaving a half-migrated state: skills have **no YAML frontmatter**, delivery is plugin-only (not cross-harness/portable), and the agents duplicate skill content.

This conflicts with the operator's portability requirements (cross-harness reach, no harness-specific delivery). The `beads-skills` sibling repo already demonstrates the desired end-state: a frontmatter-driven `install.py` (+ thin `install.sh`) that deploys bare skill trees to `~/.claude/skills` (or project/`.agents` via flags), companion rules surfaced from `protocols/`, a `DRIFT-CHECK.md` manifest that keeps the artifact graph in agreement, and local-only beads with GitHub upstream. This plan migrates naba to that proven pattern.

Triggered by: operator request (2026-06-07) to clean up and modernize the repository in four named areas.

**Who is affected:** the repo maintainer (James Dixson / Yoshiko Studios), and any user who installs naba's skills. Renaming slash commands (`/generate` → `/naba-generate`) is a deliberate, breaking UX change for namespacing.

## Scope Decisions (operator-confirmed 2026-06-07)

| # | Decision | Choice |
|---|----------|--------|
| 1 | `.claude-plugin/` fate | **Eliminate entirely.** Delete `plugin.json` + `marketplace.json`; delivery is solely `install.{sh,py}`. Task-1 "preflight refactor" is subsumed by the installer. |
| 2 | Top-level `agents/` fate | **Delete entirely.** Routing/prompt-engineering content already lives in skills. (Verify no unique capability is lost first — see Risks.) |
| 3 | Skill prefix scope | **Prefix directories AND slash commands.** `skills/generate` → `skills/naba-generate`; `/generate` → `/naba-generate`. All 9 skills. |
| 4 | Beads end-state | **Local-only Dolt (no remote) + GitHub upstream.** `.beads/issues.jsonl` tracked in git; `beads-upstream` pushes open/deferred beads to GitHub Issues on `dixson3/naba`. |
| 5 | Install default scope | **User scope, `.claude` surface** (`~/.claude/skills`). `--scope project` and `--surface agents` available as flags. |
| 6 | Shared prompt guidance | **Keep embedded per-skill.** SKILL.md bodies load only on trigger (not always-loaded), so duplication cost is low and skills stay self-contained. No new always-loaded rule. |

## Upstream Issues

None. `gh issue list` on `dixson3/naba` returns `[]` — there are no pre-existing GitHub issues to triage. No reconcile gate is required for this plan. (Upstream *tracking* is configured as part of Epic 4 so future beads flow to GitHub; that is forward-only and does not create a reconcile dependency here.)

## Investigation Findings

Surveyed inline (no worktree experiments needed — the unknowns were all directly observable):

- **F1 — Current skill state.** `skills/` holds 9 skills (`brand_kit`, `diagram`, `edit`, `generate`, `icon`, `pattern`, `restore`, `story`, `storyboard`), each a single `SKILL.md` with **no YAML frontmatter** (bodies start with a `#` heading). `install.py` in the reference is frontmatter-driven, so frontmatter is a hard prerequisite for the installer (Epic 1 must precede Epic 3).
- **F2 — Plugin + agents.** `.claude-plugin/` contains `plugin.json` + `marketplace.json` (v1.1.0). `agents/` holds `naba_image_assistant.md` (full routing table + per-command guidance) and `naba_batch_processor.md`. Commit `8b3a1db` ("Embed rules into skills and agents, remove preflight hook") already deleted the old `rules/`, `scripts/plugin-preflight.sh`, `.claude-plugin/preflight.json`, and the symlink system — so "the preflight" the request references no longer exists as code; the residual work is finishing the migration to an installer.
- **F3 — Reference pattern (`beads-skills`).** `install.sh` is a thin wrapper (~15 lines) with a `uv`-on-PATH guard, then `exec uv run "$(dirname "$0")/install.py" "$@"`. `install.py` (PEP 723 / `uv run`): parses SKILL.md frontmatter (`skill-group`, `depends-on-tool`, `depends-on-skill`), computes the install set with transitive `depends-on-skill` closure, checks `depends-on-tool` against PATH (warn, or abort under `--strict`), resolves dest by `--scope {user,project}` × `--surface {claude,agents}` (or explicit `--target`), and rsyncs each skill (`rsync -a --delete --exclude=.gitignore`). Companion rules are copied from each skill's `protocols/*.md` to the sibling `rules/` dir (keep-existing unless `--force`). `--dry-run`, `--list-groups`, `--strict` supported. **No plugin/marketplace, and no `--uninstall`** (verified absent) — naba's installer adds uninstall as net-new design, not adaptation. The reference also ships **zero per-skill `protocols/` dirs that naba would reuse**; naba intends to ship **zero companion rules** of its own, so the installer's `protocols/*.md → rules/` path is present-but-unused for naba. A repo-root `DRIFT-CHECK.md` (`approved: yes`) declares nodes/edges/trigger-globs; per-skill `protocols/manifest.json` tracks rule hashes/versions.
- **F4 — Beads state (task 4, partially already true).** `bd dolt remote list` → **none** (already local-only). `bd doctor` → 0 errors, 8 warnings: pending schema migration (now resolved — see F5), outdated git hooks (`bd hooks install --force`), outdated `.beads/.gitignore` + project `.gitignore` missing patterns (`bd doctor --fix`), and `AGENTS.md`/`CLAUDE.md` user-content divergence. `.beads/issues.jsonl` does **not** exist yet (DB is empty: 0 issues); it must remain *not* gitignored so it is tracked once beads exist. `.beads/interactions.jsonl` is already git-tracked. Upstream tracking is **not yet configured** (`bd config` has no `github.owner/repo`, no `backend`). Note: `beads-upstream init` configures via `bd config set` only — it does **not** write a `## Upstream Tracking` section into project docs; the trigger contract ships as the skill's companion rule via `install.sh`. The `Remote Consistency: No remotes configured` doctor warning is **expected and intentional** under local-only and will be permanently accepted (its remediation — add a Dolt remote — would violate Decision 4).
- **F5 — Pre-work already done this session (blocker removal).** bd was wedged: a pending schema migration (DB v0.60.0 → CLI v1.0.5) was blocked by a dirty Dolt working set, which also caused `bd status --json` to error and made the bdplan preflight falsely report `bd_not_initialized`. Resolved by `bd dolt stop` (flushes + clears the in-memory working set) then `bd migrate schema` (now at v49); `.beads` perms set to 700. bd is healthy. This is recorded so a cold reader understands why the migration/gitignore items in Epic 4 are "finish + verify," not "discover."
- **F6 — Cross-references that the rename must follow.** `brand_kit` invokes `naba icon`/`pattern`/`generate` and names the `brand_kit`/`storyboard` skills; `storyboard` references the `story`/`edit` skills. CLAUDE.md's Architecture section and README enumerate command/skill names. All must be updated to the `naba-` names.

## Approach

Five epics, sequenced so the frontmatter exists before the frontmatter-driven installer, the final names are settled before the installer and drift manifest reference them, and drift-check runs last over a settled structure. Beads config (Epic 4) is largely independent and runs in parallel with the skill work; the only shared surface is the project-root instruction files, which are reconciled once in Epic 5 (so `optimal-instructions` restructuring and beads' managed `BEGIN/END BEADS INTEGRATION` sections do not fight).

Rationale tied to findings: F1 forces Epic 1 → Epic 3 ordering; F3 gives the concrete installer to adapt (no design unknowns); F4/F5 mean Epic 4 is "finish + verify + configure upstream," not greenfield; F6 scopes the rename's blast radius.

```
            ┌─────────────────────────────────────────────┐
  start ───▶│ Epic 1  Skill conventions (skill-authoring)  │
  gate  │   └───────────────┬─────────────────────────────┘
        │                   ▼
        │   ┌─────────────────────────────────────────────┐
        │   │ Epic 2  naba- prefix rename                  │
        │   └───────────────┬─────────────────────────────┘
        │                   ▼
        │   ┌─────────────────────────────────────────────┐
        │   │ Epic 3  Eliminate plugin + install.{sh,py}   │
        │   └───────────────┬─────────────────────────────┘
        │                   │
        └──▶┌─────────────────────────────────────────────┐
            │ Epic 4  Beads local-only + GitHub upstream   │ (parallel)
            └───────────────┬─────────────────────────────┘
                            ▼   (Epic 3 + Epic 4)
            ┌─────────────────────────────────────────────┐
            │ Epic 5  drift-check + project-root docs      │
            └─────────────────────────────────────────────┘
```

## Epics

### Epic 1: Skill conventions alignment (skill-authoring)

Bring all 9 skills into `skill-authoring` compliance. This is the prerequisite for the frontmatter-driven installer (F1).

- **Issue 1.1:** Add YAML frontmatter to all 9 `skills/*/SKILL.md`. Fields: `name` (current short name for now; renamed in Epic 2), `description` (with explicit trigger + skip language per skill-authoring), `user-invocable: true`, `skill-group: naba`, `depends-on-tool: [naba]` (composite skills may also note `naba` only — the CLI is the sole external tool), `allowed-tools` (Bash, Read at minimum). Composite skills (`brand_kit`, `storyboard`) add `depends-on-skill` to their component skills.
  - gate: start gate
- **Issue 1.2:** Apply the skill-authoring token-efficiency pass (Cut/Keep/Extract) to each SKILL.md body. Keep prompt-engineering guidance embedded per skill (Decision 6). Remove redundancy, tighten flag tables, ensure each body is self-contained.
  - depends-on: 1.1
- **Issue 1.3:** Add a per-skill `README.md` (human-facing one-paragraph description + usage line) to each skill dir, mirroring the reference layout. This is a drift-check edge target in Epic 5.
  - depends-on: 1.1

### Epic 2: naba- prefix rename

Rename all skills and their slash commands to the `naba-` namespace (Decision 3). Depends on Epic 1 so frontmatter exists to update in one pass.

- **Issue 2.1:** Rename the 9 skill directories (`skills/generate` → `skills/naba-generate`, `skills/brand_kit` → `skills/naba-brand-kit`, etc. — kebab-case) using `git mv`, and update each SKILL.md frontmatter `name:` to match (the `name` drives the slash command).
  - depends-on: 1.1, 1.2, 1.3
- **Issue 2.2:** Update all cross-skill references to the new names (F6): `naba-brand-kit` → `naba-icon`/`naba-pattern`/`naba-generate` and the `naba-storyboard` reference; `naba-storyboard` → `naba-story`/`naba-edit`. Update `depends-on-skill` frontmatter to the new dependency names.
  - depends-on: 2.1
- **Issue 2.3:** Update repo docs that enumerate skill/command names (README.md, CLAUDE.md Architecture section, `docs/specifications/*` where they name commands/skills) to the `naba-` names. Note the breaking slash-command change.
  - depends-on: 2.1

### Epic 3: Eliminate plugin + build install.{sh,py}

Replace plugin delivery with the frontmatter-driven installer; delete the plugin and the redundant agents. Depends on Epics 1+2 (frontmatter present, names final).

- **Issue 3.1:** Verify no unique capability is lost before deleting `agents/`. Diff `agents/naba_batch_processor.md` and `agents/naba_image_assistant.md` against the skill set; if the batch-processing capability is **not** represented by any skill, fold it into a skill (likely a new/extended composite) before deletion. Record the finding.
  - depends-on: 2.1
- **Issue 3.2:** Author `install.py` adapted from `beads-skills/install.py`: PEP 723 inline deps (`pyyaml`), frontmatter parse, install-set + `depends-on-skill` transitive closure, `depends-on-tool` PATH check (warn / `--strict` abort), dest resolution (`--scope {user,project}` default user × `--surface {claude,agents}` default claude, or `--target`), `rsync -a --delete --exclude=.gitignore` per skill, `protocols/*.md` → sibling `rules/` (keep-existing unless `--force`; present-but-unused for naba per F3), `--dry-run`, `--list-groups`. MIT license header attributing James Dixson / Yoshiko Studios LLC (2026).
  - **`--uninstall` is net-new design (not in the reference — F3, C1).** It removes only the `naba-*` skill dirs the installer owns (match by `naba-` prefix), and **leaves companion rules in place by default** (they are keep-existing/hand-editable); only `--uninstall --force` removes naba-shipped rules. Since naba ships zero companion rules (F3), the rule path is effectively a no-op but must not delete unrelated rules. Uninstall gets its own verification in 3.6.
  - depends-on: 1.1, 2.1
- **Issue 3.3:** Author `install.sh` — thin wrapper: `exec uv run "$(dirname "$0")/install.py" "$@"` with a `uv`-on-PATH check.
  - depends-on: 3.2
- **Issue 3.4:** Delete `.claude-plugin/` (`plugin.json`, `marketplace.json`) and the top-level `agents/` directory (after 3.1 confirms no capability loss).
  - depends-on: 3.1, 3.2
- **Issue 3.5:** Update `README.md` to document installation via `./install.sh` (scope/surface flags, `--dry-run`, `--uninstall`), replacing the plugin/marketplace install instructions. **Document the `naba` CLI binary as a prerequisite** for the skills to be functional (skills carry `depends-on-tool: [naba]`; the installer warns "inert until present" when the binary is absent — note this expected warning). Update CLAUDE.md if it references plugin delivery. **Verify the top-level `LICENSE` exists with correct attribution** (MIT / James Dixson / Yoshiko Studios LLC / 2026) — a `LICENSE` is already present; confirm or correct it.
  - depends-on: 3.2, 3.4
- **Issue 3.6:** Verify deployment: `./install.sh --dry-run` lists all 9 `naba-*` skills; run a real install to a throwaway `--target` (tmp dir) and confirm trees land intact and frontmatter `name` yields the `/naba-*` commands; confirm **no `rules/` dir is created** (naba ships zero companion rules — F3); run `--uninstall` and confirm clean removal of only `naba-*` skills (unrelated skills/rules untouched). Also confirm the **old `/<name>` commands** (`/generate`, `/icon`, …) are absent from installed locations (no stale plugin remnants).
  - depends-on: 3.2, 3.3, 3.4

### Epic 4: Beads local-only Dolt + GitHub upstream

Finish and lock the beads configuration (F4/F5). Largely independent; gated only on the start gate.

- **Issue 4.1:** Confirm and document local-only Dolt: `bd dolt remote list` shows none (already true); record the intent (no `bd dolt remote add`, no `bd dolt push`). Ensure `config.yaml`/`metadata.json` carry no remote.
  - gate: start gate
- **Issue 4.2:** Resolve beads hygiene from `bd doctor`: `bd hooks install --force` (update outdated hooks); `bd doctor --fix` (repair `.beads/.gitignore` + project `.gitignore` missing patterns); commit the schema-migration result. Confirm `.beads/issues.jsonl` is **not** gitignored so it is tracked once beads exist.
  - depends-on: 4.1
- **Issue 4.3:** Configure upstream tracking to GitHub via the `beads-upstream` skill. This is **`bd config set` only** — init does NOT write any `## Upstream Tracking` section into project docs (C2). Set `github.owner=dixson3`, `github.repo=naba`, `backend=github`, `custom.upstream.enabled=true`, and `dolt.local-only true` (asserts local-only intent). Verify `gh auth` works (it does). Do a scoped `--dry-run --push-only` sanity check; do not bulk-import existing issues (there are none). The always-loaded trigger contract reaches Claude via the existing global `UPSTREAM_TRACKING.md` rule, not a project file.
  - depends-on: 4.1
- **Issue 4.4:** Confirm `bd doctor` reports **0 errors** and that every remaining warning is either resolved or **explicitly accepted**. `Remote Consistency: No remotes configured` is a **permanently-accepted warning** (intentional local-only; resolving it would add a Dolt remote, violating Decision 4) — record this acceptance. Confirm the JSONL/DB round-trips after a test bead create/close.
  - depends-on: 4.2, 4.3

### Epic 5: drift-check bootstrap + project-root instruction alignment

Settle the project-root docs and prove cross-edge agreement. Runs last (structure + config final). Depends on Epics 3 and 4.

- **Issue 5.1:** Reconcile the project-root instruction files. **Order matters (C6):** first run `bd setup claude && bd setup codex` so the managed `BEGIN/END BEADS INTEGRATION` blocks are regenerated/anchored consistently (resolves the doctor "Agent Doc Divergence" warning); **then** apply `optimal-instructions` to restructure *around* the managed block — `AGENTS.md` primary, `CLAUDE.md` a thin `@`-include index, behavioral rules in the project rules surface — taking care not to strip or relocate the managed block; **then** re-run `bd doctor` to confirm no divergence reappeared. Ensure architecture/skill references reflect `naba-*` + `install.{sh,py}` (no plugin).
  - depends-on: 2.3, 3.5, 4.3
- **Issue 5.2:** Bootstrap `DRIFT-CHECK.md` for naba (adapt the `beads-skills` manifest): nodes = each skill's `SKILL.md`/`README.md`, `install.py`, project `README.md`, `docs/specifications/*`; edges = (a) SKILL.md frontmatter ↔ per-skill README, (b) `depends-on-tool` ↔ documented prerequisites, (c) the set of `skills/naba-*` dirs ↔ skill list documented in README/CLAUDE.md, (d) skill/command names ↔ `docs/specifications`. Trigger globs over `skills/*/SKILL.md`, `skills/*/README.md`, `install.py`, `README.md`. Present to operator and set `approved: yes` only on confirmation (human gate).
  - depends-on: 5.1
- **Issue 5.3:** Run `drift-check` over the declared edges; resolve every FAIL in the same pass and surface INCONCLUSIVE to the operator. Re-run until clean.
  - depends-on: 5.2

## Gates

### Start Gate (mandatory)
- Type: human
- Approvers: operator

### DRIFT-CHECK manifest approval (capability gate)
- Type: human
- Condition: operator reviews and approves the bootstrapped `DRIFT-CHECK.md` (sets `approved: yes`); drift-check is a silent no-op until then.
- Test: `grep -q '^approved: yes' DRIFT-CHECK.md`
- Blocks: Issue 5.3
- Instructions: Present the drafted manifest (nodes, edges, trigger globs). On approval, set `approved: yes`.

_No reconcile gate: no upstream issues are incorporated (Upstream Issues = none)._

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Renaming slash commands (`/generate` → `/naba-generate`) breaks existing users' muscle memory and any scripts. | Deliberate per Decision 3. Document the breaking change in README; bump version; note in commit. |
| Deleting `agents/` could drop a unique capability (esp. `naba_batch_processor`) not covered by any skill. | Issue 3.1 diffs agent content against skills *before* deletion; fold any unique capability into a skill first. |
| `optimal-instructions` restructure of CLAUDE.md/AGENTS.md fights beads' managed `BEADS INTEGRATION` sections. | Reconcile both in one issue (5.1) and run `bd setup` afterward so managed sections regenerate cleanly. |
| Accidental Dolt remote / push contradicting "local-only". | Epic 4 explicitly asserts no remote; no `bd dolt remote add`/`push` anywhere; doctor verifies. |
| `install.py` deviating from the proven reference introduces subtle deploy bugs. | Adapt the `beads-skills` installer closely for the shared paths; treat `--uninstall` as net-new design (no reference to copy — F3) with its own explicit test. Issue 3.6 verifies dry-run + real install + uninstall against a throwaway target. |
| `.beads/issues.jsonl` accidentally gitignored, losing the portable record. | Issue 4.2 explicitly confirms it is tracked; 4.4 round-trips a create/close. |
| drift-check manifest over- or under-scoped, producing noise or false confidence. | Operator approves the manifest (capability gate) before any enforcing run; start with the four high-value edges above. |

## Success Criteria

1. All 9 skills have valid `skill-authoring` frontmatter, token-trimmed bodies, and a per-skill README; renamed to `skills/naba-*` with `/naba-*` slash commands and all cross-references updated.
2. `.claude-plugin/` and top-level `agents/` are removed; `install.sh` + `install.py` deploy all `naba-*` skills to `~/.claude/skills` by default (with `--scope`/`--surface`/`--target`/`--dry-run`/`--uninstall`), verified against a throwaway target.
3. `README.md` documents installer-based deployment; no references to the marketplace plugin remain.
4. `bd doctor` reports 0 errors with hygiene warnings resolved **or explicitly accepted** (`Remote Consistency` is accepted by design — local-only); Dolt has no remote; `.beads/issues.jsonl` is git-tracked; upstream is configured via `bd config` (`github.owner=dixson3`, `github.repo=naba`, `backend=github`, `enabled=true`, `dolt.local-only=true`).
5. An operator-approved `DRIFT-CHECK.md` exists and `drift-check` passes (no FAIL) over the declared edges.
6. Project-root `AGENTS.md`/`CLAUDE.md` are reconciled (no doctor divergence) and aligned with `optimal-instructions`.
