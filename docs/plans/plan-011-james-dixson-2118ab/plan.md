# Plan: Author genuine MCP-specific skill/resource guidance for naba (how to invoke the MCP tools generate_image/edit_image/etc.), replacing the current mcp/ render that just re-exposes CLI slash-command guidance; keep context minimal/lazy-loaded; correct web/ mcp docs to match

**ID:** plan-011-james-dixson-2118ab
**Author:** james-dixson
**Created:** 2026-07-21
**Status:** complete
**Epic:** naba-mol-699
**Fingerprint:** 3ec39d8f010d74b05ff296045727473d3e8295e5eb3618748847688a9854cdb6
**Phase log:**
- 2026-07-21 scoping: initial scope captured
- 2026-07-21 investigating: 3 scope decisions captured; investigating build.rs authoring layout + test blast radius
- 2026-07-21 drafting: synthesizing plan v1
- 2026-07-21 review: plan v1 presented
- 2026-07-21 review: red-team pass-2 — APPROVE (pass-1 REVISE concerns C1-C6 resolved; 1 low C7 folded)
- 2026-07-21 ready-for-approval: ready-check green — pass-2 APPROVE + audit pass
- 2026-07-21 approved: operator approved
- 2026-07-21 intake: epic naba-mol-699 poured
- 2026-07-21 executing: start gate resolved
- 2026-07-21 reconciling: post-execution reconciliation
- 2026-07-21 complete: plan complete — authored MCP render landed to local main; merged-tree validation green (cargo 280, parity 128, traceability, embed hash); push + issue #15 closure pending operator authorization

## Objective

Make naba's MCP resource surface serve **genuine MCP-tool guidance** — how to invoke
`generate_image` / `edit_image` / `generate_icon` / … and their params — instead of the current
`mcp/` render, which is just the CLI skill with its CLI-only sections stripped (still framed
around `/naba <subcommand>` and `--flags`). Keep always-loaded context minimal: thin pointers in
tool descriptions, detailed guidance lazily fetched as `skill://` resources. Correct the web/ MCP
docs to describe the real behavior.

## Motivation

naba exposes its image pipeline to shell-less assistants (Claude Desktop, Cursor) as **MCP
tools**, and advertises an embedded skill tree as lazy `skill://` **resources** so the assistant
can fetch usage guidance on demand. The intent of that resource surface is to give the assistant
*skill-like guidance for the MCP tools* — the MCP analogue of the CLI's `commands/*.md`.

Today it does not. `build.rs` templates only `SKILL.md` and copies every `commands/*.md`
**verbatim** into both the `cli/` and `mcp/` renders, and `skills/naba/SKILL.md` contains **zero
`{% if mcp %}` blocks**. So an MCP client is handed guidance about `/naba generate`, `--style`,
and Bash — none of which exist in an MCP session. The guidance is not just thin, it is **wrong for
the context**. The SPEC itself codifies this as a "subtractive, MCP-flavored" render
(SPEC-EMBED-005, SPEC-MCP-014), so the gap is at the specification level, not merely the code.

This was surfaced while reviewing the plan-006 website's MCP page: the docs accurately described
the mechanism but revealed it re-exposes the CLI skill rather than MCP-specific guidance. Affected:
every MCP consumer of naba (desktop assistants), plus anyone reading the web docs who would be
misled about what the resource surface provides.

## Scope Decisions (operator-confirmed, 2026-07-21)

- **Hybrid delivery.** A thin, one-line pointer appended to each MCP tool's `description` (so the
  assistant discovers that guidance exists) **plus** detailed **lazy** `skill://` resources fetched
  on demand. Not fat always-loaded tool descriptions (rejected: cuts against minimal context), not
  resources-only (rejected: the assistant may never look).
- **One MCP usage guide + per-tool files only where needed.** A single MCP-framed `skill://naba`
  entry (tool catalog, prompt-engineering, `NABA_OUTPUT_DIR`/output-dir resolution, quality
  semantics, `file://` result links) plus a per-tool guidance file only for tools with genuine
  specifics. Not one file per tool unconditionally (rejected: authoring/sync churn).
- **Replace CLI command docs in the mcp tree.** The `mcp/` render carries MCP-authored content
  only — no `/naba`, no `--flags`. The `cli/` render keeps `commands/*.md` unchanged and
  byte-identical to source.

## Out of Scope

- **Changing the MCP tool surface itself** — tool `name`s, `inputSchema`, param enums (the
  plan-010 single-source), required sets, and generation behavior are unchanged. Only each tool's
  `description` gains a pointer.
- **The CLI skill / `cli/` render** — `skills install`, the deployed tree, and the SPEC-EMBED-002
  tree hash stay byte-identical; existing installs must not be forced to `upgrade`.
- **Adding new MCP tools or resources beyond the guidance content** (e.g. no new `file://`
  behavior).

## Upstream Issues
| Issue | Title | Disposition | Notes | Resolved By |
|:------|:------|:------------|:------|:------------|
| _(none — no open GitHub issue; a single tracking issue is filed at intake per project convention)_ | | | | |

## Investigation Findings

Full detail in [`findings/exp-001-mcp-render-mechanics.md`](findings/exp-001-mcp-render-mechanics.md). Summary:

- **The gap is real and SPEC-level.** `build.rs` templates only `SKILL.md`; `commands/*.md` are
  copied verbatim to both trees; `SKILL.md` has **zero `{% if mcp %}` blocks**. SPEC-EMBED-005 /
  SPEC-MCP-014 define the mcp render as *"subtractive"* — so the SPEC must change too.
- **Key de-risker: the tree hash covers the `cli/` tree only** (`embedded_tree_hash` →
  `read_skill_file` → `$OUT_DIR/cli`). The `mcp/` tree is neither hashed nor deployed by `skills
  install`. **Authoring MCP content into the mcp render cannot change install-integrity status or
  force an upgrade.** The only constraint is that the **cli render stays byte-identical to source**
  (so MCP content must render to nothing under `cli`).
- **build.rs authoring layout (design fork).** To emit a different file set for mcp vs cli:
  (1) `{% if mcp %}` guide in `SKILL.md` + an **mcp-only source subtree** (`skills/naba/mcp/…`)
  that build.rs routes to the mcp render only, while **excluding `commands/*.md` from the mcp**
  render; or (2) template every file with `{% if %}` gates. **Approach 1 recommended** (clean
  separation; cli stays byte-identical because the mcp subtree and `{% if mcp %}` blocks vanish
  under `cli`).
- **Blast radius (must move together):** SPEC (`skills.md` SPEC-EMBED-005, `mcp.md`
  SPEC-MCP-014/015) · impl (`build.rs`, `skills/naba/SKILL.md` + new `skills/naba/mcp/…`,
  `src/mcp.rs` tool-description pointers) · **`src/mcp.rs` Rust unit tests**
  (`read_skill_resource_returns_file_and_index`, `skill_resources_enumerate_files_and_index`) ·
  tests/goldens (`test_mcp.py` `EXPECTED` descriptions + `golden/mcp/tools.json` regen; the
  **resources/list enumeration oracle** — hand-review the new `skill://` set, never
  blind-regenerate) · **traceability** (`tests/parity/check_traceability.py` /
  `traceability_exemptions.yaml:98`, a separate `make traceability` target) · docs
  (`web/content/pages/mcp.md`, and verify `README.md` MCP section) · `DRIFT-CHECK.md` (approved
  manifest; `mcp-source`/`mcp-spec`/`skill-spec`/`web-mcp` nodes + `e-web-mcp-tools` edge; new
  content files may need a node/glob and the "subtractive" text re-approved).
- **Landmines:** the resource-enumeration oracle is a *guard* — update it to the **intended** set,
  not by blind `--update-golden`. The `quality` `<QUALITY_DESC>` golden normalization must not
  collide with the new pointer text. Keep tool `name`/`inputSchema` untouched.

## Approach

Author real MCP guidance content, teach `build.rs` to emit it into the mcp render only, wire a thin
pointer into each tool description, and bring the SPEC, parity oracle, drift-check manifest, and web
docs into agreement — in that dependency order, each guarded by the parity suite.

1. **SPEC first (source of truth).** Amend SPEC-EMBED-005 (skills.md) from a *subtractive* mcp
   render to an **authored** MCP render with an mcp-only source subtree, preserving the cli
   byte-identity/tree-hash invariant. Amend SPEC-MCP-014/015 (mcp.md): the served resource set is
   the MCP-authored files, and add the tool-`description` pointer clause (SPEC-MCP-013-adjacent).
   Re-approve the `DRIFT-CHECK.md` manifest text for the authored-render model.
2. **build.rs render routing.** Extend the two-tree render so `skills/naba/mcp/…` maps into the
   **mcp** tree only and `commands/*.md` is **excluded** from the mcp tree; `SKILL.md` gains
   `{% if mcp %}` handling. Prove the **cli/ render stays byte-identical** (SPEC-EMBED-002 hash
   test still green).
3. **Author the MCP content.** A `{% if mcp %}` MCP-framed guide in `SKILL.md` (tool catalog,
   prompt-engineering, `NABA_OUTPUT_DIR`, quality semantics, `file://` results) + per-tool
   `skills/naba/mcp/…` files only where a tool needs specifics. No `/naba`/`--flags`.
4. **Thin tool-description pointers.** Append a one-line pointer (e.g. *"Usage guidance:
   `skill://naba`"*) to each of the 8 tool descriptions in `src/mcp.rs`.
5. **Parity oracle + goldens.** Update `test_mcp.py` `EXPECTED` descriptions and **hand-review** the
   `skill://` resource-enumeration assertions to the intended new set; regen `golden/mcp/tools.json`.
   Confirm `cargo test` + the parity suite green.
6. **Web docs.** Rewrite `web/content/pages/mcp.md` "Lazy-loading skills as resources" to describe
   MCP-authored guidance and *why the user cares*; verify links/anchors + `make validate`.

## Epics

### Epic 1: SPEC + render mechanics (the authored mcp render)
Make the specification and `build.rs` support an authored, MCP-only content set while preserving
the cli byte-identity/tree-hash invariant. No user-visible MCP content yet.
- Issue 1.1: Amend **SPEC-EMBED-005** (`docs/specifications/skills.md`) — the mcp render is
  **authored** (not subtractive): `SKILL.md` gains `{% if mcp %}`; an mcp-only source subtree
  (`skills/naba/mcp/…`) renders into the mcp tree only; `commands/*.md` is excluded from the mcp
  tree; the cli render stays byte-identical (SPEC-EMBED-002 preserved). Amend **SPEC-MCP-014/015**
  (`docs/specifications/mcp.md`): served resource set = the MCP-authored files; drop "subtractive".
  Add the tool-`description` pointer clause. **Traceability (C2):** any new/amended `[NEW]`/`[PINNED]`
  clause id must be either cited by a `test_mcp.py` docstring or exempted in
  `tests/parity/traceability_exemptions.yaml`; update that file's stale line-98 SPEC-EMBED-005
  entry ("mcp/ subtractive") to the authored-render wording so `make traceability` passes.
- Issue 1.2: Extend `build.rs` render routing to implement 1.1 — mcp-only subtree → mcp tree;
  `commands/*.md` excluded from mcp; `{% if mcp %}` handled. **Prove cli byte-identity (exit
  criterion, C4):** `cargo test embed::` (the SPEC-EMBED-002 embedded-hash test) stays green and
  `skills status` reports up-to-date.
  - depends-on: 1.1

### Epic 2: Authored MCP content + tool-description pointers
The user-visible change: real MCP guidance, lazily served, with thin discovery pointers.
- depends-on: Epic 1
- Issue 2.1: Re-author `skills/naba/SKILL.md` as a genuine dual render. Today only
  `## Preflight` / `## Router` / `### Global flags` are `{% if cli %}`-gated; the **frontmatter
  `description`** ("naba CLI, invoked as `/naba …`"), the **intro line**, **`allowed-tools:
  [Bash, Read, Agent]`**, and the **dispatch table** still render into BOTH trees. Gate **every**
  CLI-specific element with `{% if cli %}` and author `{% if mcp %}` counterparts: an MCP-framed
  `description` and intro (MCP tools, no `/naba`), MCP-appropriate frontmatter, and an MCP-framed
  guide body (tool catalog by MCP tool name, prompt-engineering, `NABA_OUTPUT_DIR`/output-dir
  resolution, quality semantics, `file://` result links). Add per-tool `skills/naba/mcp/…` files
  only where a tool has real specifics. **Design decision (C1):** keep the frontmatter `name`
  shared but gate the `description` **body** via `{% if cli %}`/`{% if mcp %}` — minijinja renders
  the frontmatter as text so the deployed cli render stays valid YAML; the raw source frontmatter
  carrying jinja is an accepted, documented tradeoff. **Exit criteria:** (a) `cargo test embed::`
  green (cli byte-identity preserved — see the capability gate); (b) render + read back the mcp
  tree and confirm it contains **no `/naba` and no `--flag`** token; (c) the cli render is
  byte-identical to source.
  - depends-on: 1.2
- Issue 2.2: Append a thin one-line pointer (e.g. *"Usage guidance: `skill://naba`"*) to the tool
  `description`s in `src/mcp.rs`. **Make a conscious choice** whether the pointer belongs on all 8
  tools or only the 7 generation tools (excluding the `list_images` utility) — either is
  defensible; whichever is chosen, the `test_mcp.py` `EXPECTED` update in 2.3 must match. Tool
  `name`/`inputSchema` unchanged.
  - depends-on: 1.2
- Issue 2.3: Update the guards to the intended surface (hand-reviewed, not blind-regenerated):
  (a) `tests/parity/test_mcp.py` `EXPECTED` descriptions (pointer appended) and the `skill://`
  resource-enumeration assertions (the new MCP file set — the current assertion hard-codes
  `commands/generate.md`/`commands/edit.md`); regen `golden/mcp/tools.json` via `--update-golden`;
  (b) the **`src/mcp.rs` Rust unit tests** `read_skill_resource_returns_file_and_index` (asserts the
  mcp `SKILL.md` contains `### Prompt engineering` and NOT `## Router`/`## Preflight`) and
  `skill_resources_enumerate_files_and_index` (iterates `skill_files_mcp`) — update to the new MCP
  framing/enumeration; (c) `make traceability` (`check_traceability.py`) passes. `cargo test` +
  the parity suite + `make traceability` green.
  - depends-on: 2.1, 2.2

### Epic 3: Docs + drift-check reconciliation
Bring the human-facing docs and the drift manifest into agreement with the new behavior.
- depends-on: Epic 2
- Issue 3.1: Rewrite `web/content/pages/mcp.md` "Lazy-loading skills as resources" to describe
  MCP-authored guidance (not "the same CLI skill via resources") and add the *why the user cares*
  framing + the skill-vs-MCP distinction already added elsewhere. **Also verify the `README.md` MCP
  section (≈L331/371-374, bound by `e-readme-web-install`) is still accurate** and edit if needed.
  Lint (GFM subset), verify links/anchors resolve, `make validate` passes.
  - depends-on: 2.1
- Issue 3.2: Re-approve/adjust `DRIFT-CHECK.md` for the authored-render model — update the
  "subtractive" manifest text, add a node/glob for the new `skills/naba/mcp/…` content if the
  manifest requires it, and run the drift-check over the touched edges
  (`mcp-source`/`mcp-spec`/`skill-spec`/`web-mcp`, `e-web-mcp-tools`) to confirm agreement.
  **Intermixed-source risk (C5):** the `skill-md` node globs the raw source `skills/naba/SKILL.md`
  which now carries both cli+mcp prose; if the prose-comparison edges (`e-skill-spec`,
  `e-readme-desc`, `e-web-skills-subcommands`) false-positive on MCP-only content, scope those
  manifest contracts to the cli-framed sections.
  - depends-on: 3.1, 2.3

## Gates
### Start Gate (mandatory)
- Type: human
- Approvers: operator

### Capability Gate: cli byte-identity preserved
- Type: auto
- Condition: the `cli/` render stays byte-identical to `skills/naba/` source; SPEC-EMBED-002
  embedded-tree-hash test is green and `skills status` reports up-to-date after the render change.
- Test: `cargo test embed::` (the embed hash / status tests) — must be green.
- Blocks / exit criterion (C4): green **before** Issue 2.1 begins AND as an **exit criterion of
  both Issue 1.2 and Issue 2.1** — because the `{% if mcp %}` blocks are authored into the *shared*
  `skills/naba/SKILL.md`, cli byte-identity is re-endangered during 2.1 (a malformed block or bad
  whitespace-control can leak into the cli render), not only at the render split.
- Instructions: If the cli hash changed, the render routing or a template block leaked mcp-only
  content (or a whitespace artifact) into the cli tree — fix so the cli render is byte-for-byte the
  source.

_No reconcile gate: no upstream issues incorporated (the single tracking issue is filed at intake,
not reconciled per-bead)._

## Risks & Mitigations

| Risk | Mitigation |
|:-----|:-----------|
| **cli render drifts** (mcp-only content or a template artifact leaks into cli, breaking the SPEC-EMBED-002 hash → forces every install to re-`upgrade`) | The capability gate above blocks Epic 2 until the embed-hash test is green; E1 confirmed the hash covers only the cli tree, so a correct split is provably safe. `{% if mcp %}` blocks + an mcp-only subtree render to nothing under `cli`. |
| **Blind golden regeneration blesses a wrong resource set** | The resource-enumeration assertion is an **independent oracle/guard** — Issue 2.3 hand-reviews it to the *intended* `skill://` set; only `golden/mcp/tools.json` (mechanical schema snapshot) is `--update-golden`-regenerated. |
| **Drift-check manifest fights the change** (approved manifest says the mcp render is "subtractive"; new content files may be unglobbed nodes) | Issue 3.2 explicitly re-approves the manifest text and adds a node/glob for `skills/naba/mcp/…`; drift-check is run over the touched edges as the closing check. |
| **Tool-description pointer collides with the `<QUALITY_DESC>` golden normalization or bloats context** | Pointer is one short line on the tool `description` (not params); E1 flagged the normalization; keep the pointer text distinct and minimal. |
| **Over-authoring re-creates the CLI skill in MCP clothes** | Scope pins "one guide + per-tool only where needed"; content is authored against MCP tool names/params with an explicit no-`/naba`/no-`--flags` rule, read back from the rendered mcp tree to verify framing. |

## Success Criteria

- The rendered **mcp/** tree serves **MCP-framed guidance** — tool names (`generate_image`, …),
  params, `NABA_OUTPUT_DIR`, `file://` results — with **no `/naba` slash commands and no `--flag`
  tokens** (verified by grepping the rendered mcp `SKILL.md` and `skills/naba/mcp/…`).
- The **cli/** render is unchanged and byte-identical to `skills/naba/` source; the SPEC-EMBED-002
  tree hash is preserved (`cargo test embed::` green) and `skills status` reports up-to-date (no
  forced upgrade).
- Each of the 8 MCP tool descriptions carries a thin pointer to the lazy `skill://` guidance;
  tool `name`/`inputSchema` (incl. the plan-010 enum single-source) are unchanged.
- SPEC (SPEC-EMBED-005, SPEC-MCP-014/015), `build.rs`, the authored content, `src/mcp.rs` (incl.
  its `skill://` unit tests), the parity oracle + `golden/mcp/tools.json`, `DRIFT-CHECK.md`, and
  `web/content/pages/mcp.md` all **agree**; `cargo test`, the parity suite, **`make traceability`**,
  drift-check over the touched edges, and `make validate` are green.
- Always-loaded MCP context stays minimal — detailed guidance is fetched **on demand** via
  `resources/read`, not baked into tool schemas.
