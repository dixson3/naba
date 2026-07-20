# Plan: Rename `--surface` → `--harness`, idiomatic per-harness skills install, harness-layout SPEC, SPEC consolidation, dual-purpose CLI/MCP skills

**ID:** plan-008-james-dixson-24173a
**Author:** james-dixson
**Created:** 2026-07-20
**Status:** approved
**Fingerprint:** 1c740f70c64af025f039363e84567c14060c6e146a6509c19809377eaf1ee8b9
**Phase log:**
- 2026-07-20 scoping: initial scope captured
- 2026-07-20 investigating: 8 experiments identified (E1-E8)
- 2026-07-20 drafting: 4 findings synthesized; 3 design decisions resolved
- 2026-07-20 drafting: plan v1 complete (6 epics, 3 gates)
- 2026-07-20 review: red-team pass-1 — REVISE (1 high, 3 medium, 1 low; specification gaps)
- 2026-07-20 review: red-team pass-2 — REVISE (1 blocker: prose-only sequencing; 2 low refinements)
- 2026-07-20 review: red-team pass-3 — APPROVE (blocker closed via edges; no cycle; 2 optional polish applied)
- 2026-07-20 ready-for-approval: ready-check green — pass-3 APPROVE + audit pass
- 2026-07-20 approved: operator approved

## Objective

Evolve naba's skills-install surface from a single uniform directory prefix into a
harness-aware installer, and consolidate the project's specification docs:

1. **Rename** the CLI option `--surface` → `--harness` (shared by `skills` and `doctor`),
   keeping `--surface` as a deprecated alias.
2. **Idiomatic per-harness install.** `--harness` selects a target harness — `claude-code`,
   `opencode`, `pi` (pi.dev), `codex` — each with its own idiomatic user- and project-scoped
   skills layout (today's uniform `.<surface>/skills/` becomes one harness's layout, not all).
   `--harness` may be given multiple times to install for several harnesses at once.
3. **Upgrade-all.** `naba skills upgrade` upgrades previously-installed skills for **every**
   harness they were installed into (tracked, not re-specified).
4. **Harness-layout SPEC** in `docs/specifications/` that defines each harness's layout; naba
   implements and validates against it.
5. **Consolidate + reconcile the specs.** Split the root `SPEC.md` §1–§18 into per-domain spec
   files under `docs/specifications/`, retire the stale PRD/TODO/EDD/IG docs, and then
   **reconcile every remaining spec so it accurately reflects the current Rust implementation**
   (a spec-vs-impl audit — no spec left describing behavior the code no longer has, or missing
   behavior the code now has).
6. **Dual-purpose skills.** Factor skills so they serve both as naba-CLI skills and as
   optimal-invocation skills for the naba MCP tools (architecture TBD by investigation:
   handlebars-like template engine rendered per-target, vs two skill sets).
7. **Integrate `docs/diary/`** into `docs/specifications/` or a proto-plan as appropriate.
8. **Remove all lingering Go infrastructure.** The Go implementation is fully
   deprecated/replaced by the Rust binary; purge remaining Go remnants — doc references
   (README migration note, PRD/TODO Go-isms), the `Makefile` `*-go` comment, and any
   Go-oriented vestige in the `tests/parity/` suite (repurpose it as a pure Rust/golden suite
   or retire it) — so nothing in the repo implies a Go build path.

## Motivation

`--surface` currently means "a directory-prefix name" (`resolve_dest` →
`<anchor>/.<surface>/skills/`, SPEC-SKILLS-003) — a uniform layout that only happens to match
Claude Code's convention. As naba positions itself as a portable, harness-agnostic image tool
(GH #12), users on other agent harnesses (opencode, pi.dev, codex) need skills installed where
*their* harness looks for them, with the right per-harness structure and scoping. The word
"surface" also conflates "directory prefix" with "target harness"; `--harness` names the real
concept. Simultaneously, the spec docs have drifted: an authoritative 59 KB root `SPEC.md`
coexists with a stale Go/Gemini-era PRD/TODO/EDD/IG under `docs/specifications/`, so there is no
single trustworthy spec home. This plan is the first concrete realization of the GH #12
"portable agent-tools SPEC" seed (skills self-management + MCP-over-CLI axes).

## Upstream Issues

| Issue | Title | Disposition | Notes | Resolved By |
|:------|:------|:------------|:------|:------------|
| [#12](https://github.com/dixson3/naba/issues/12) | plan-008 seed: portable agent-tools SPEC (skills self-mgmt + MCP-over-CLI + `--json`), reconcile with yoshiko-flow | partial | This plan advances the **skills-self-management** (harness-layout SPEC + per-harness install) and **MCP-skills** (dual-purpose skills) axes. It does **not** cover the `--json` agent-output work or the yoshiko-flow reconciliation — those remain open on #12. | (partial) |

## Scope Decisions (operator-confirmed)

- **Harness breadth:** SPEC **and** implement all four harnesses (claude-code, opencode, pi,
  codex) in this plan. Requires research into opencode/pi/codex skill conventions.
- **Backward compatibility:** keep `--surface` as a **deprecated alias** of `--harness`, and
  migrate existing install receipts/markers so `upgrade` still finds prior installs.
- **Dual-purpose architecture:** **investigate then recommend** — evaluate a template engine
  vs two skill sets against embed size, MCP-resource rendering, and drift, then choose in-plan.
- **SPEC consolidation depth:** **split + retire stale + reconcile-to-impl** — split `SPEC.md`
  §1–§18 into per-domain files under `docs/specifications/`, add the harness-layout SPEC,
  retire the stale PRD/TODO/EDD/IG, then audit every surviving spec against the Rust code so
  the spec set is an accurate mirror of the implementation.
- **Go deprecation:** remove all lingering Go infrastructure; the repo must contain no Go
  build path or Go-implies-current references (the Rust binary is the sole implementation).

### Extensibility

The listed four are the initial supported set; the design should make adding a harness a
data/SPEC change (a harness-layout descriptor), not a structural rewrite (the objective's
"etc." — cursor/windsurf/aider/… are future additions, not in scope now).

## Out of Scope

- `--json` agent-output work and the yoshiko-flow reconciliation from #12 (stay open on #12).
- Harnesses beyond the four named (design for extensibility, but do not implement others).
- Any change to the image-generation pipeline, providers, or self-update mechanics.

## Open Questions → Investigation

- **E1 — claude-code layout (baseline):** confirm today's `.claude/skills/` user + project
  layout is the canonical claude-code idiom, and factor it as the reference harness descriptor.
- **E2 — opencode conventions:** where does opencode look for user- and project-scoped skills
  (dir structure, manifest, naming)? Does it have a "skills" concept or an equivalent?
- **E3 — pi (pi.dev) conventions:** same questions for pi.
- **E4 — codex conventions:** same questions for codex (AGENTS.md-centric?).
- **E5 — install/upgrade tracking + migration:** how the current receipt/integrity-marker
  mechanism (SPEC-EMBED / SPEC-PREFLIGHT) records installs; how to record *which harnesses* a
  skill was installed into so `upgrade` re-hits all of them; and the `--surface`→`--harness`
  receipt/marker migration path.
- **E6 — dual-purpose skills architecture:** prototype/evaluate template-engine vs two-skill-set
  against embed size, MCP resource rendering (`SPEC-MCP` skills-as-resources), and drift;
  return a recommendation.
- **E7 — SPEC consolidation + reconciliation inventory:** map `SPEC.md` §1–§18 and the
  existing `docs/specifications/` files (PRD, TODO, EDD/CORE, IG/*) to current-vs-stale; for
  each surviving spec section, note where it diverges from the Rust implementation (so the
  reconcile epic has a punch-list); propose the target `docs/specifications/` structure and the
  diary disposition.
- **E8 — Go-remnant inventory:** enumerate every lingering Go reference/artifact (README
  migration note, `Makefile` `*-go` comment, `tests/parity/` Go-oriented pieces, PRD/TODO
  Go-isms, any `.github`/script vestige) and classify each as remove vs repurpose, so the
  Go-purge epic is fully specified.

## Investigation Findings

Full findings in `findings/`. Summary:

- **E1–E4 harness conventions** (`findings/exp-001-harness-conventions.md`): all four harnesses
  have a first-class `SKILL.md` directory-per-skill concept; the install **unit + required
  frontmatter is identical** — only path data differs, so **no per-harness content transform is
  needed**. The uniform `.<x>/skills/` rule is only correct for claude-code. opencode/pi/codex
  also honor cross-harness `.agents/skills/`; codex's *official* path is `.agents/skills` (not
  `.codex/skills`, which is unverified). Harness = a data descriptor with **split** user/project
  subpaths + anchors (opencode's `~/.config/opencode` breaks any `$HOME/.<id>` shortcut).
- **E5 install/upgrade tracking** (`findings/exp-005-install-upgrade-tracking.md`): **no skills
  install receipt exists today** — only per-`SKILL.md` markers (version+tree). Unqualified
  `upgrade` only touches `$HOME/.claude/skills`. Multi-harness upgrade needs a new
  `<config_dir>/skills-install.json` target registry; migration synthesizes it from a legacy
  disk scan. The `present_surfaces` heuristic (`self_cmd/update.rs`) is the seam to replace.
- **E6 dual-purpose skills** (`findings/exp-006-dual-purpose-skills.md`): the shared
  prompt-engineering core is byte-identical CLI↔MCP; the CLI shell is mostly *subtractive* for
  MCP. Render-at-install would break the `deployed==embedded` hash invariant → **render at build
  time into two embedded trees (`cli/`+`mcp/`)**. Also fixes a real defect: MCP currently serves
  CLI-flavored `SKILL.md` telling hosts to shell out via Bash (SPEC-MCP-014/015).
- **E7+E8 specs + Go** (`findings/exp-007-spec-and-go-inventory.md`): `SPEC.md` is the current
  Rust contract (only header framing + 3 clause lags stale); `docs/specifications/` PRD/TODO/EDD
  are Go-era stale; IG/* are duplicates with stale Go samples. **No Go source/build artifacts
  remain**; the parity suite is already a pure Rust golden suite (only framing mentions Go). Hard
  deletes: `.golangci.yml` + the three Go-era spec docs. Diary → retire/archive.

### Resolved design decisions (operator, 2026-07-20)

- **`.agents` model:** ship a first-class **portable `agents` harness** (writes `.agents/skills`,
  covers opencode+pi+codex readers in one shot) **plus** native rows for claude-code/opencode/pi/
  codex. Legacy `--surface agents` → the `agents` harness (no behavior change).
- **claude layout:** **relabel-only** — claude-code keeps the physical `.claude/skills/` dir; the
  rename is logical (flags + receipt). No file moves; the receipt decouples logical harness from
  physical path.
- **Template engine:** **minijinja** as a `[build-dependencies]` entry (zero runtime/binary cost),
  rendering the single skill source into `cli/`+`mcp/` at build time.

## Approach

1. **Harness-as-data.** A `HarnessDescriptor` table (rows: claude-code, opencode, pi, codex, and
   portable `agents`) carries `id`, split `user_anchor`/`user_subpath` + `project_anchor`/
   `project_subpath`, `manifest_file=SKILL.md`, `frontmatter_required`, `name_transform`.
   `resolve_dest` becomes harness-aware. Adding a harness = a new data row + SPEC row.
2. **Rename + alias.** `--harness` (repeatable, `Vec`) replaces `--surface`; `--surface` stays as
   a deprecated alias mapping values (`claude→claude-code`, `agents→agents`). Shared by `skills`,
   `doctor`, `preflight`.
3. **Install receipt.** New `<config_dir>/skills-install.json` target registry (upsert-keyed on
   `(harness, scope, path)`). Install upserts; unqualified `upgrade` enumerates all recorded
   targets (**continue-on-error**, skip-and-report stale project paths, per-target `--json`).
   Migration synthesizes the first receipt from a legacy disk scan; `preflight` and the
   post-self-update refresh both drive off the receipt.
4. **Dual-purpose skills.** Templatize the single `skills/naba` source with coarse `{% if cli %}`
   / `{% if mcp %}` gates; a `build.rs` step (minijinja build-dep) renders `cli/` + `mcp/`
   embedded trees. `embed.rs` installs the `cli/` tree (marker/hash unchanged); MCP resources
   serve the `mcp/` tree — fixing SPEC-MCP-014/015.
5. **Harness-layout SPEC + validation.** Author the descriptor table + discovery/scope/migration
   rules as a spec section; naba validates against it (per-harness path-assertion tests, and a
   check that the shipped descriptor matches the SPEC).
6. **SPEC consolidation + reconciliation.** Split `SPEC.md` §1–§18 into per-domain files under
   `docs/specifications/` (stable clause IDs), reframe the header, retire PRD/TODO/EDD, merge
   IG/* then retire, retire/archive the diary, and update every `SPEC.md` reference
   (Makefile, DRIFT-CHECK.md, AGENTS.md, skill spec-edge). Then reconcile every surviving spec to
   the Rust impl (fix §1 → 15 groups incl `self`, §3.11 → add `preflight`, provider help → add
   bedrock) and fold the new harness/dual-purpose/receipt behavior into the specs.
7. **Go purge.** Delete `.golangci.yml`; scrub the parity suite's Go-baseline framing; trim the
   `Makefile`/`DRIFT-CHECK.md`/`mcp.rs` Go comments; keep the README migration note.

## Epics

### Epic 1: Harness model + `--surface` → `--harness` rename
Foundation for everything else.
- Issue 1.1: `HarnessDescriptor` table (5 rows) + harness resolution (split anchors/subpaths).
- Issue 1.2: `--harness` flag (repeatable) + deprecated `--surface` alias + value mapping;
  thread through `skills`/`doctor`/`preflight` Opts (`cli.rs`, `commands.rs`).
  - depends-on: 1.1
- Issue 1.3: Rewrite `resolve_dest` harness-aware; per-harness/scope path unit tests.
  - depends-on: 1.1
- Issue 1.4: Update CLI help text/prose **and `doctor` display output** (drop "surface"; document
  harnesses; `doctor` currently surfaces installed surfaces — update its prose/labels).
  - depends-on: 1.2
- Issue 1.5: Reconcile **web docs** for the rename (red-team C + operator directive) — update the
  `web/content/pages/*` pages that reference `--surface`/skills install, **explicitly the
  `usage`, `skills`, and `mcp` sections** (plus `config`/`install` where `--surface` appears), for
  `--harness` + multi-harness install (the four idiomatic harnesses + portable `agents`), and the
  new MCP `mcp/`-tree behavior. Satisfies the DRIFT-CHECK `e-web-skills-lifecycle` /
  `e-web-install-methods` fixed-authority edges (cli-source → web) that FAIL when `cli.rs` gains
  `--harness`. **Sequence after the impl + validations land** (per the operator: update web docs
  once the `--harness` changes and validations are complete) — so `depends-on` the harness impl
  and its validation gate.
  - depends-on: 1.2, 1.3, 1.4, 4.2

### Epic 2: Install receipt + multi-harness upgrade + migration
- depends-on: Epic 1
- Issue 2.1: `skills-install.json` schema + read/write (upsert) + `config_dir` helper.
- Issue 2.2: Install upserts target(s); `--harness` repeatable installs to each; `--target`
  override recorded too.
  - depends-on: 2.1
- Issue 2.3: Unqualified `upgrade` enumerates receipt targets (continue-on-error, stale-skip,
  per-target `--json`). **Dedupe by resolved absolute path before deploy/prune** (red-team E) —
  portable `agents` and `codex` both resolve to `.agents/skills`, so `--harness codex --harness
  agents` must deploy/prune that dir **once**, not twice.
  - depends-on: 2.1
- Issue 2.4: Migration — synthesize receipt from legacy disk scan (`.claude`, `.agents`, git
  root); map old surfaces → harnesses; idempotent.
  - depends-on: 2.1
- Issue 2.5: Replace `present_surfaces` heuristic + drive `preflight` off the receipt.
  - depends-on: 2.3, 2.4

### Epic 3: Dual-purpose skills (build-time two-tree render)
- depends-on: Epic 1 (shared skills structure); may overlap Epic 2.
- Issue 3.0 (**decision, do first**): Pin the embed-root restructure before any render code
  (red-team A). Decide: (a) render target — `$OUT_DIR` (re-point `include_dir!`, no committed
  render, no gitignore churn) **[recommended]** vs a committed `skills/{cli,mcp}/` tree; (b) how
  `skill_names`/`skill_files`/`SKILLS` compute the skill root so they still enumerate `naba`, not
  `cli`/`mcp`; (c) **byte-identical `cli/` render vs accepted forced-upgrade** — pursue
  byte-identical (Jinja whitespace-control) to keep the pinned embedded-tree hash
  (`embed.rs:16-22`); if unachievable, re-baseline the pin and accept a one-time forced re-upgrade
  across **all** recorded receipt targets (Epic 2), documented as such. Output: a short decision
  note in the epic + the re-baseline procedure if taken.
- Issue 3.0b: DRIFT-CHECK.md update for the skill-node restructure (red-team B). **Unconditional:**
  update the `e-installer-skillset` contract text (no longer "one dir per `skills/*/`" /
  `include_dir!("skills")`), note that the `skill-md` source **is now a template** (Jinja-tagged;
  the installer deploys the *rendered* `cli/` tree, not the source — so field-set comparisons read a
  template, not deployed content), and perform the **§0 re-approval**. **Conditional on 3.0's
  outcome:** re-glob the `skill-md`/`commands` nodes **only if** 3.0 chose a committed
  `skills/{cli,mcp}/` layout; under the recommended `$OUT_DIR` render the source does not move, so no
  re-glob is needed. Wired before 3.2 via that issue's `depends-on`.
  - depends-on: 3.0
- Issue 3.1: Add minijinja `[build-dependencies]`; `build.rs` render pipeline scaffold.
  - depends-on: 3.0
- Issue 3.2: Templatize `skills/naba` source with `{% if cli %}`/`{% if mcp %}` gates (MCP mostly
  subtractive: drop router/preflight/composite/flags).
  - depends-on: 3.1, 3.0b (DRIFT-CHECK manifest updated + §0 re-approved **before** this edit fires
    the on-edit engine)
- Issue 3.3: Emit `cli/`+`mcp/` embedded trees; `embed.rs` installs `cli/`, MCP resources read
  `mcp/`; preserve the `deployed==embedded` parity test.
  - depends-on: 3.2
- Issue 3.4: MCP serves `mcp/` tree (fix SPEC-MCP-014/015); update MCP resource tests.
  - depends-on: 3.3
- Issue 3.5 (stretch): fold param/enum inventory to one source (skill core ↔ `mcp.rs` golden).
  **Filed as a standalone follow-on bead** so the pre-existing skill-md↔`mcp.rs` drift survives if
  3.5 is descoped (red-team Missing).
  - depends-on: 3.4

### Epic 4: Harness-layout SPEC + naba validates against it
- depends-on: Epic 1; lands into the new docs structure (Epic 5, Issue 5.1).
- Issue 4.1: Author the harness-layout SPEC (descriptor table + discovery/scope/migration rules)
  in `docs/specifications/skills.md`. **Document the codex↔`agents` `.agents/skills` path overlap**
  (red-team E) and the resolved-path dedupe rule.
  - depends-on: 5.1
- Issue 4.2: naba validates against the SPEC — a **descriptor-matches-SPEC** check (the shipped
  descriptor table agrees with the SPEC). This is *verification*, separate from the CI path gate
  (which is sourced from Issue 1.3, below) and does **not** gate Epic 2.
  - depends-on: 4.1, 1.3
- Issue 4.3: **Live harness discovery smoke-test** (local-tier, gated on `command -v <harness>`) —
  install naba's skill to each harness's idiomatic path, then confirm the *running* harness
  discovers/lists it. All four target harnesses are locally runnable and were verified against
  env credentials (claude-code; opencode→Bedrock; pi→OpenRouter; codex→OpenRouter via
  `-c model_provider`). CI lacks these harnesses, so this tier **skips** there and the portable
  path-assertion tests (Issue 1.3) remain the CI baseline. Record the exact per-harness invocations
  in `references/` so the test is reproducible.
  - depends-on: 4.1, 1.3, 2.2

### Epic 5: SPEC consolidation + reconciliation
- Issue 5.1: Split `SPEC.md` §1–§18 → per-domain files under `docs/specifications/` (stable
  clause IDs); reframe header (drop Go→Rust port framing). Leave a **`SPEC.md` redirect stub**
  (pointer to the new files) rather than hard-deleting it, until Issue 5.3 re-points the
  DRIFT-CHECK `skill-spec` node — avoids a dangling-node window (red-team pass-3 P2).
  **Structural; can start early.**
- Issue 5.2: Retire PRD/TODO/EDD; merge IG/* current content into the new files then retire;
  retire/archive the diary.
  - depends-on: 5.1, 5.3 (DRIFT-CHECK nodes re-pointed + §0 re-approved **before** these deletions,
    so the engine never trips on dangling nodes)
- Issue 5.3: Update all `SPEC.md` references to the new paths — Makefile, AGENTS.md, skill
  spec-edge, and the **DRIFT-CHECK.md manifest**: re-point the `skill-spec`/`ig-configuration`/
  `edd-core` nodes (their `docs/specifications/IG/*`, `EDD/CORE.md` targets are deleted/merged in
  5.2) and perform the **§0 re-approval** the manifest convention requires after node changes
  (red-team B). Sequence before 5.2's file deletions so the on-edit engine does not FAIL.
  - depends-on: 5.1
- Issue 5.4: Reconcile-to-impl — fix §1 (15 groups incl `self`), §3.11 (`preflight`), provider
  help (bedrock); audit each surviving section vs `src/` per the E7 punch-list.
  - depends-on: 5.1
- Issue 5.5: Fold the new harness/dual-purpose/receipt behavior into the specs so they mirror the
  final implementation.
  - depends-on: 5.4, Epic 1, Epic 2, Epic 3, Epic 4

### Epic 6: Go-remnant purge
- Mostly independent; the IG/diary Go cleanup is handled in Epic 5.
- Issue 6.1: Delete `.golangci.yml`.
- Issue 6.2: Scrub `tests/parity/` Go-baseline framing (README, docstrings, `cases/*.yaml`
  prose); re-verify the mcp xfail against the Rust binary.
- Issue 6.3: Trim `Makefile:2`, `DRIFT-CHECK.md:26`, `mcp.rs:701-702` Go comments; keep the
  README:408 migration note (optionally time-boxed).

## Gates

### Start Gate (mandatory)
- Type: human
- Approvers: operator

### Capability Gate: Harness path validation
- Type: auto
- Approvers: automated (CI — the Test is the approver)
- Condition: `resolve_dest` produces the correct idiomatic path for every harness × scope,
  matching the SPEC descriptor. **Two tiers:** (1) **portable path-assertion tests** (Issue 1.3,
  the CI baseline — no harness needed); (2) an optional **live discovery smoke-test** (Issue 4.3,
  local-only, gated on `command -v <harness>`) that installs into each harness and confirms the
  running harness discovers the skill. All four harnesses were verified locally runnable.
- Test: `cargo test resolve_dest_harness_paths` (Issue 1.3) green in CI; the live smoke-test
  (Issue 4.3) is a local tier that self-skips where a harness is absent.
- Blocks: Issue 4.2 (descriptor↔SPEC verification). **Does not block Epic 2** — the receipt/upgrade
  work needs correct `resolve_dest` (Issue 1.3), not the SPEC authoring.

### Capability Gate: Embed parity preserved
- Type: auto
- Approvers: automated (CI — the Test is the approver)
- Condition: build-time two-tree render keeps the `deployed==embedded` hash invariant and
  `naba skills status` is clean after install.
- Test: `cargo test` embed parity tests pass + `naba skills install` then `naba skills status`
  reports up-to-date/unmodified.
- Blocks: Issue 3.4, Issue 5.5.

### Reconcile Gate (upstream #12 partial)
- Type: auto (all execution beads closed)
- Approvers: automated (all execution beads closed)
- Condition: comment on GH #12 recording the partial progress (skills-self-mgmt + MCP axes done;
  `--json` output + yoshiko-flow reconciliation still open).
- Blocks: reconcile step.

## Risks & Mitigations

| Risk | Mitigation |
|:-----|:-----------|
| opencode/pi/codex not present in CI | Portable **path-assertion tests** (Issue 1.3) are the CI baseline. All four harnesses are **locally runnable** (verified: opencode→Bedrock, pi→OpenRouter, codex→OpenRouter), so Issue 4.3 adds a **live discovery smoke-test** as a local tier (self-skips where absent) — the descriptor table is the single source both impl and SPEC assert against. |
| codex `.codex/skills` unconfirmed | Use codex's **official `.agents/skills`** (E1–E4); do not add a `.codex/skills` row without OpenAI-doc confirmation. |
| **Embed-root restructure** breaks `skill_names`/hash pin (red-team A) | Issue **3.0** pins the render target (`$OUT_DIR`), keeps the skill root computing `naba`, and pursues a **byte-identical `cli/` render**; fallback is a re-baselined hash pin + documented one-time forced re-upgrade across all receipt targets. |
| **DRIFT-CHECK churn** across Epic 3 + 5 FAILs the on-edit engine (red-team B) | Issues **3.0b** (contract-text update + §0 re-approval unconditional; *conditional* skill-node re-glob only if 3.0 picks a committed layout — `depends-on` wired into 3.2) and **5.3** (node re-point + §0 re-approval, `depends-on` wired into 5.2) sequence the manifest update **ahead of the triggering edits via dependency edges**, not prose. |
| **Web-docs drift** on `--surface`→`--harness` (red-team C) | Issue **1.5** updates the four `web/content/pages/*` pages to satisfy the `e-web-*` fixed-authority edges. |
| **Receipt path-overlap** (`agents`↔`codex` both `.agents/skills`) (red-team E) | Issue **2.3** dedupes upgrade enumeration by **resolved absolute path** before deploy/prune; documented in the harness SPEC (4.1). |
| Breaking existing `.claude`/`.agents` installs | **Relabel-only** claude layout + **receipt synthesis** from legacy disk scan + idempotent upsert marker injection → old installs upgrade cleanly on first run. |
| minijinja adds a dependency | **`[build-dependencies]`** only → zero runtime/binary cost; render happens at build time. |
| SPEC reconciliation scope creep | Bound reconciliation to the **E7 divergence punch-list**; clause IDs never renumbered. |
| SPEC.md path move breaks DRIFT-CHECK / references | Issue 5.3 updates the `DRIFT-CHECK.md` manifest + every `SPEC.md` reference atomically with the split. |
| Multi-target upgrade partial failure | **Continue-on-error** loop with per-target `--json` outcomes + non-zero exit if any failed; stale project paths skipped-and-reported. |
| Legacy `agents`→harness ambiguity | Resolved: portable `agents` harness writes `.agents/skills`; documented in the harness SPEC. |

## Success Criteria

- `naba skills install --harness claude-code --harness opencode` installs to **both** idiomatic
  paths; `--harness` repeatable; `agents` portable harness writes `.agents/skills`.
- `naba skills upgrade` with **no flags** upgrades **every** previously-installed harness target
  (receipt-driven), continue-on-error, per-target `--json`.
- `--surface` still works as a **deprecated alias** (mapped correctly); legacy `.claude`/`.agents`
  installs upgrade cleanly via receipt synthesis.
- A **harness-layout SPEC** exists in `docs/specifications/`; naba **validates against it** (tests
  assert per-harness/scope paths and descriptor↔SPEC agreement).
- **Extensibility proven:** adding a harness is a **single data row + SPEC row** — a test/fixture
  demonstrates a new descriptor row resolving correctly with no structural code change.
- **Live discovery smoke-test** (local tier) confirms opencode/pi/codex/claude-code each discover
  a naba skill installed to their idiomatic path (self-skips where a harness is absent); the
  reproducible per-harness invocations are recorded in `references/`.
- **Web docs** (`web/content/pages/*`) — the **`usage`, `skills`, and `mcp` sections** (plus
  `config`/`install` where `--surface` appears) — document `--harness`/multi-harness install and
  the new MCP tree behavior, updated **after** the impl + validations land; the DRIFT-CHECK
  `e-web-*` edges pass.
- MCP `skill://naba/SKILL.md` serves **MCP-flavored** content (no router/preflight/Bash mechanics);
  the `deployed==embedded` parity test passes; `naba skills status` clean after install.
- `docs/specifications/` holds the **split, reconciled** specs; root `SPEC.md` removed/redirected;
  stale PRD/TODO/EDD + diary retired; IG merged; every surviving spec **matches the impl** (15
  groups incl `self`, `preflight` in §3.11, bedrock in provider help; no Go framing).
- **No Go build path/config remains** (`.golangci.yml` gone; parity Go-framing scrubbed);
  `cargo build`, `cargo test`, `cargo clippy -D warnings`, `cargo fmt --check`, and the parity
  suite all green.
- GH **#12** updated with a partial-progress comment.
