# Plan: Consolidate 10 naba-* skills into a single /naba <subcommand> skill with subagent dispatch and de-duplicated boilerplate

**ID:** plan-002-james-dixson-a508e7
**Author:** james-dixson
**Created:** 2026-06-13
**Status:** approved
**Epic:** naba-mol-azx
**Phase log:**
- 2026-06-13 scoping: initial scope captured
- 2026-06-13 drafting: plan v1 presented
- 2026-06-13 review: plan v1 presented
- 2026-06-13 review: pass-2 APPROVE (residual N1/N2 folded in)
- 2026-06-13 approved: operator approved
- 2026-06-13 intake: epic naba-mol-azx poured

## Objective

Replace the 10 separate `skills/naba-*` skills with a single `skills/naba` skill invoked
as `/naba <subcommand> …`. The unified skill routes each subcommand, runs single-call
commands inline, dispatches the multi-call composites to subagents for context isolation,
and carries the shared prompt-engineering / anti-pattern / global-flag guidance exactly
once. Tooling (`install.py`), docs (`README.md`, `AGENTS.md`), and the drift contract
(`DRIFT-CHECK.md`) are reconciled to the new single-skill layout.

## Motivation

The `skills/naba-*` set has grown to 10 directories (generate, edit, restore, icon,
pattern, diagram, story, storyboard, batch, brand-kit). Each ships a near-identical
`SKILL.md` and a thin stub `README.md`. The Explore audit found three blocks duplicated
**verbatim across all 10 skills**: the Global Flags table, the Anti-Patterns section, and
the "subject + composition + style + lighting + details" prompt framing. Frontmatter is
~95% identical (`skill-group: naba`, `depends-on-tool: [naba]`, same `allowed-tools`).

The operator wants one skill with `/naba <subcommand>` invocation, using subagents to
implement features and control context. The duplication is a maintenance liability (any
guidance change is a 10-file edit) and the 10-command namespace is more surface than the
underlying CLI warrants. This plan was triggered by the operator's request to "figure out
a more optimal way to construct the naba skills."

Affected: the skill author (James Dixson / Yoshiko Studios), and anyone who has installed
the naba skills (the `/naba-*` slash commands change to `/naba <subcommand>` — a breaking
change for muscle memory and any external references).

## Scope Decisions (operator-confirmed 2026-06-13)

| #   | Decision               | Choice                                                                  |
| :-: | :--------------------- | :--------------------------------------------------------------------- |
| 1   | Dispatch model         | **Hybrid.** Single-call subcommands (generate, edit, restore, icon, pattern, diagram, story) run inline via the router reading `commands/<sub>.md`. Composites (storyboard, batch, brand-kit) dispatch to a subagent that runs the loop and returns a summary. |
| 2   | Backward compatibility | **Clean break.** All 10 `skills/naba-*` directories are removed; only `/naba <subcommand>` remains. Documented as a breaking change. |
| 3   | `install.py` machinery | **Keep, minimal touch.** Frontmatter-driven discovery already handles one skill dir. Retain group/depends-on machinery (dormant); only verify single-skill install and update README examples. |

Out of scope:
- No Go CLI changes. `batch`, `brand-kit`, `storyboard` remain skill-level orchestration
  of existing `naba` verbs — they are **not** added as cobra subcommands.
- No new image-generation features; this is a packaging/structure refactor only.

## Upstream Issues

None. `gh issue list --repo dixson3/naba --state open` returned empty on 2026-06-13.

## Investigation Findings

No separate INVESTIGATE phase was run; a single read-only Explore sweep of the repo
supplied the facts below.

- **CLI verb coverage.** 7 of the 10 subcommands map 1:1 to existing cobra commands
  (`generate, edit, restore, icon, pattern, story, diagram` in `internal/cli/*.go`). The
  other 3 (`batch, brand-kit, storyboard`) are composites with no Go command — they
  orchestrate the existing verbs. This matches the hybrid dispatch split exactly.
- **Duplication.** Global Flags table, Anti-Patterns section, and prompt-engineering
  framing are verbatim in all 10 `SKILL.md` files. Per-command unique content is small:
  a usage line, a command-specific flag table, and a few command-specific prompt notes /
  examples.
- **READMEs are stubs.** All 10 skill `README.md` files are 14–22 line metadata mirrors
  (description + usage + identical Prerequisites/Install boilerplate). No unique content
  worth preserving beyond the per-command usage line.
- **Composite dependency graph.** `brand-kit → {icon, pattern, generate}`,
  `storyboard → {story, edit}`. No cycles. Collapses to internal dispatch.
- **Blast radius beyond the skills dir:** `README.md` "Claude Code Skills" section (install
  examples + 10-row available-skills table + namespacing note); `AGENTS.md` "Claude Code
  Skills" section (`skills/naba-*`, `/naba-*`); `DRIFT-CHECK.md` (entire manifest is built
  around 10 `skills/naba-*` nodes and a "one `/naba-*` row per dir (10 rows)" contract);
  `install.py` (works as-is for one skill, but README examples reference per-skill names).
- **Specifications gap.** `docs/specifications/` is entirely CLI/MCP — `PRD.md`
  (FS-001…FS-010 map to cobra commands), `EDD/CORE.md` (Go package architecture), and the
  `IG/` guides document the binary. **No spec covers the Claude-facing skill layer.** The
  consolidation does not conflict with any existing spec (no Go changes), but AGENTS.md
  elevates `docs/specifications/*` to source-of-truth, so the operator chose to close the
  gap with a new IG guide (Issue 2.5).

## Approach

### Target layout

```
skills/naba/
  SKILL.md            # frontmatter + router + shared guidance (authored ONCE)
  README.md           # single README; subcommand table
  commands/
    generate.md       # inline   — usage, unique flags, command-specific prompt notes, examples
    edit.md           # inline
    restore.md        # inline
    icon.md           # inline
    pattern.md        # inline
    diagram.md        # inline
    story.md          # inline
    storyboard.md     # composite (subagent) — orchestrates story → edit
    batch.md          # composite (subagent) — sequential set generation
    brand-kit.md      # composite (subagent) — icon + pattern + hero trio
```

### SKILL.md responsibilities (single source of truth)

1. **Frontmatter** — one skill:
   - `name: naba`
   - `description:` a single comprehensive TRIGGER/SKIP block covering all the
     natural-language intents the 10 descriptions used to cover (create/edit/restore/
     enhance image, icon, pattern, diagram image, image series/storyboard, asset set/brand
     kit), plus the explicit `/naba <subcommand>` invocation. SKIP for editable diagram
     **source** (`diagram-authoring` / `mermaid`).
   - `user-invocable: true`, `skill-group: naba`, `depends-on-tool: [naba]`
   - `allowed-tools:` union across all subcommands plus the subagent-spawn tool —
     `[Bash, Read, Glob, Write, Task]`. (Exact spawn-tool name to be verified at execution
     — see Risks.)
2. **Router** — parse the first token as the subcommand; validate against the dispatch
   table; on unknown/missing/`help`, print the subcommand list. Inline subcommands: read
   `commands/<sub>.md` and execute. Composite subcommands: dispatch a subagent.
3. **Shared guidance (authored once)** — the prompt-engineering order, the Anti-Patterns
   list, and the Global Flags table. Lives in SKILL.md so it is in context for every
   `/naba` invocation; composite subagents are told to read SKILL.md for this guidance.

### commands/<sub>.md responsibilities

Only the per-command unique content: usage line, command-specific flag table, any
command-specific prompt notes, and examples. No repetition of shared guidance.

### Composite dispatch (subagent)

For `storyboard`, `batch`, `brand-kit`, the router spawns a subagent and runs the sequence
of `naba` CLI calls in the child, returning a compact summary (file paths / manifest) so
intermediate per-image output stays out of the parent context.

**Context delivery (resolves red-team C2).** A freshly spawned subagent inherits none of the
parent's context, and after install the skill lives at the *deployed* path
(`~/.claude/skills/naba/…`, `.agents/…`, or a `--target` dir) — never the repo path
`skills/naba/…`. So the router must NOT tell the subagent to "read `skills/naba/SKILL.md`".
Instead, at dispatch the router resolves its own skill base directory (the directory the
running `SKILL.md` lives in) and passes into the subagent prompt EITHER (a) the resolved
**absolute** path of `commands/<sub>.md`, or (b) the shared guidance inlined directly. The
spike (Issue 0.1) picks (a) vs (b); 1.1 documents the chosen mechanism.

## Epics

### Epic 0: Verify the dispatch contract (de-risk before authoring)

- Issue 0.1: Verification spike (resolves red-team C1, C2, C6). Confirm, before authoring
  10 command files, that the invocation model is real:
  1. A `user-invocable` skill receives `<subcommand> <args>` from `/naba generate foo` in a
     parseable way, and a markdown-body router can branch on the first token. (Use a
     throwaway one-line skill or the `claude-code-guide` agent to confirm the contract.)
  2. The skill can resolve its own deployed base directory so the router can hand a
     subagent an **absolute** `commands/<sub>.md` path (settles C2 option a vs b).
  3. `naba story` is a single CLI invocation (even if it emits N files) — confirms `story`
     belongs in the inline tier, not the subagent tier (C6).
  4. Where the composite's `Bash`/`Write`/`Glob` grant must live — parent-skill frontmatter
     vs the dispatched subagent's own tool grant (resolves red-team N1). The parent's
     `allowed-tools` only needs the spawn tool + the inline tier's `Bash`/`Read`; the
     composite's file-writing grant lives in the child. Pin this down before authoring 1.1.
  - A negative result on (1) is a design pivot, not a tweak — surface to operator.

### Epic 1: Author the consolidated `skills/naba` skill

- Issue 1.1: Write `skills/naba/SKILL.md` — frontmatter, router/dispatch table, and the
  shared guidance (prompt-engineering, anti-patterns, global flags) authored once. The
  single `description` MUST preserve every **external** SKIP boundary the 10 descriptions
  carried (notably: diagram *image* vs editable diagram *source* → `diagram-authoring` /
  `mermaid`); keep a short natural-language phrase list and sanity-check the merged
  description against it (resolves C4). Document the chosen subagent context-delivery
  mechanism from Issue 0.1 (resolves C2).
  - depends-on: 0.1
- Issue 1.2: Write the 10 `skills/naba/commands/<sub>.md` reference files, extracting the
  unique per-command content from each old `skills/naba-*/SKILL.md`. Mark inline vs
  composite per the hybrid split.
  - depends-on: 1.1
- Issue 1.3: Write the single `skills/naba/README.md` with the subcommand table and
  install pointer.
  - depends-on: 1.1
- Issue 1.4: Remove the 10 `skills/naba-*` directories (clean break).
  - depends-on: 1.1, 1.2, 1.3

### Epic 2: Reconcile tooling, docs, and the drift contract

- Issue 2.1: Verify `install.py` installs the single `skills/naba` correctly
  (`--dry-run`, `--list-groups`, explicit-name install `./install.sh naba`); adjust only
  if the single-skill case misbehaves.
  - depends-on: Epic 1
- Issue 2.2: Update `README.md` "Claude Code Skills" section — install examples, collapse
  the 10-row available-skills table to a subcommand table, rewrite the namespacing note
  for `/naba <subcommand>`, and add the breaking-change note. The breaking-change note MUST
  tell existing users to run `./install.sh --uninstall` **before** updating (after the repo
  dirs are deleted, the installer can no longer discover/remove the old `/naba-*` skills —
  resolves red-team M1).
  - depends-on: Epic 1
- Issue 2.3: Update `AGENTS.md` "Claude Code Skills" section (`skills/naba` single dir;
  `/naba <subcommand>` invocation).
  - depends-on: Epic 1
- Issue 2.4: Rewrite `DRIFT-CHECK.md` manifest for the single-skill layout (resolves C3).
  Enumerate the **edge-by-edge disposition**, not a vague "rewrite":
  - `e-index-table`: collapse the "one `/naba-*` row per dir (10 rows)" contract to "the
    README subcommand table lists exactly the subcommands in the SKILL.md dispatch table /
    `commands/` dir".
  - `e-depends-on-skill`: **delete** (composites no longer have sibling skills; the
    dependency is now intra-skill router logic).
  - `e-cli-subcommand`: **retarget** from per-`SKILL.md` to "every CLI verb a
    `commands/*.md` invokes is a real cobra command in `internal/cli/*.go`".
  - README/installer edges (resolves red-team N2): `e-readme-prereqs`, `e-readme-usage`,
    `e-readme-desc`, `e-installer-frontmatter` collapse from per-skill to the single
    `skills/naba/SKILL.md` + `commands/*.md` sources — explicitly retarget each;
    `e-readme-usage` now asserts the README subcommand table covers every `commands/*.md`.
  - Nodes: single `skills/naba/SKILL.md` + `skills/naba/commands/*.md` + single
    `skills/naba/README.md`; update trigger-scope globs accordingly.
  - Add a new edge `e-skill-spec`: the consolidated `skills/naba/SKILL.md` dispatch table /
    subcommand set agrees with `docs/specifications/IG/skills.md` (keeps the new spec in
    sync). Add `docs/specifications/IG/skills.md` as a node and to the trigger-scope globs.
  - Drop the manifest to `approved: no` until the operator re-approves the rewritten
    manifest (rewriting an enforced, approved contract requires re-approval).
  - depends-on: Epic 1, 2.5
- Issue 2.5: Author `docs/specifications/IG/skills.md` — the spec for the Claude-facing
  skill layer: invocation (`/naba <subcommand> <args>`), the subcommand→CLI-verb map (7
  inline + 3 composite), the hybrid dispatch model, and where shared guidance lives.
  Follows the existing `IG/` guide style. PRD/EDD are left unchanged (skills are a
  packaging layer over the already-specified CLI).
  - depends-on: Epic 1

### Epic 3: Validate

- Issue 3.1: Install to a throwaway `--target` dir; confirm `skills/naba` lands with its
  `commands/` subdir, the router resolves a sample inline subcommand (`/naba generate`),
  AND one composite path runs its subagent through an **end-to-end** `naba` call (not
  review-only) so a subagent permission / `allowed-tools` / nesting failure is caught
  (resolves red-team C5). The composite exercised MUST **write a file** (e.g. batch's
  `-o "<dir>/<name>.png"` loop) so a child-side `Write`/`Glob` permission gap surfaces,
  not just a `Bash naba` call (resolves red-team N1).
  - depends-on: Epic 1, Epic 2
- Issue 3.2: Run `markdown-lint` over the new skill `.md` files and `drift-check` against
  the rewritten manifest; resolve findings.
  - depends-on: Epic 2, 3.1

## Gates

### Start Gate (mandatory)
- Type: human
- Approvers: operator

### Reconcile Gate
- Not needed — no upstream issues incorporated.

## Risks & Mitigations

| Risk                                  | Mitigation                                                                |
| :------------------------------------ | :----------------------------------------------------------------------- |
| **Auto-trigger precision drops.** Collapsing 10 fine-grained `description` TRIGGER/SKIP blocks into one may make the skill fire less precisely on natural-language requests. | Operator's primary path is explicit `/naba <subcommand>`. Author the single `description` to enumerate every intent the 10 covered. Acceptable tradeoff per scope decision. |
| **Subagent-spawn tool name / permission.** The exact tool a skill must declare in `allowed-tools` to spawn a subagent (e.g. `Task`) must be correct or composites fail. | Verify at execution (Issue 1.1): confirm the spawn-tool name and that `allowed-tools` permits it; smoke-test one composite in Issue 3.1. |
| **DRIFT-CHECK rewrite introduces a broken manifest.** The manifest is approved and enforced; a malformed rewrite could wedge drift-check. | Issue 2.4 follows the `drift-check` skill's manifest schema; Issue 3.2 runs drift-check to confirm the new manifest parses and passes. |
| **Lost per-command nuance.** Extracting unique content could drop a command-specific note. | Issue 1.2 extracts directly from each old `SKILL.md`; old dirs are removed only after extraction (1.4 depends on 1.2). Git history preserves the originals. |
| **Breaking change for existing installs.** `/naba-*` commands disappear. | Clean break is the operator's explicit choice; document prominently in README and the commit/version note. Tell users to `./install.sh --uninstall` before updating (M1). |
| **New invocation model proves unworkable post-merge.** | Rollback path: the 10 `skills/naba-*` dirs are recoverable from git (`git revert` the consolidation commit or restore the dirs from the prior tag); the consolidation lands as a single, revertable commit/PR. |

## Success Criteria

1. A single `skills/naba/` directory exists with `SKILL.md`, `README.md`, and
   `commands/<sub>.md` for all 10 subcommands; the 10 `skills/naba-*` dirs are gone.
2. The shared Global Flags table, Anti-Patterns section, and prompt-engineering framing
   appear exactly once (in `SKILL.md`), not per-command.
3. `/naba <subcommand>` routes correctly: inline for the 7 single-call verbs, subagent
   dispatch for the 3 composites.
4. `./install.sh` (and `--dry-run`, explicit `./install.sh naba`) installs the single
   skill cleanly.
5. `README.md`, `AGENTS.md`, and `DRIFT-CHECK.md` reflect the single-skill layout;
   `drift-check` passes against the rewritten (re-approved) manifest and `markdown-lint`
   is clean.
6. The dispatch/arg contract is **validated, not assumed**: the Issue 0.1 spike confirms
   `/naba <subcommand> <args>` reaches the router parseably and the composite subagent
   executes an end-to-end `naba` call (3.1).
7. `docs/specifications/IG/skills.md` exists and documents the consolidated skill's
   invocation, subcommand→CLI-verb map, and dispatch model; the DRIFT-CHECK `e-skill-spec`
   edge keeps it in sync with `SKILL.md`.
