# Plan: Fold naba's param/enum inventory to a single source of truth with a golden check, resolving the skill-md (skills/naba/commands/*.md) <-> src/mcp.rs enum drift (bead naba-lca)

**ID:** plan-010-james-dixson-b96a19
**Author:** james-dixson
**Created:** 2026-07-21
**Status:** approved
**Fingerprint:** c3015ded2ab9b3bf883ee5b9f0de123c2a4ceae22bb36e0b3ba4bd1e4343327a
**Phase log:**
- 2026-07-21 scoping: initial scope captured
- 2026-07-21 investigating: 1 experiment: full enum inventory across 3 sites + golden-test parsing feasibility
- 2026-07-21 drafting: synthesizing plan; plan v1 presented
- 2026-07-21 review: red-team pass-1 — APPROVE (4 concerns folded: 1 medium, 3 low)
- 2026-07-21 ready-for-approval: ready-check green — pass-1 APPROVE + audit pass
- 2026-07-21 approved: operator approved

## Objective
Fold naba's param/enum inventory to a single source of truth with a golden check, resolving the skill-md (skills/naba/commands/*.md) <-> src/mcp.rs enum drift (bead naba-lca)

## Motivation

naba's image commands share a set of **param enums** (`style`, `aspect`, `resolution`,
`quality`, plus per-domain ones: icon `corners`/`background`, pattern `density`, story
`transition`, icon-style, pattern-style). The valid-value lists are written **by hand in three
places that can silently drift apart**:

1. `src/mcp.rs` `tools()` — hand-built JSON-Schema `"enum": [...]` arrays (what an MCP client sees).
2. `skills/naba/commands/*.md` — markdown tables of valid values (what the agent/skill reads).
3. `src/cli.rs` — clap `--help` doc-comments (what a CLI user sees).

Add, retire, or reword an enum value and all three must be edited by hand; nothing enforces
agreement, so the **skill core (`skills/naba`) and the MCP schemas (`src/mcp.rs`) can disagree
about legal values** — the pre-existing skill-md ↔ mcp.rs drift this plan resolves.
`aspect`/`resolution` are *already* single-sourced (`VALID_ASPECT_RATIOS`/`VALID_IMAGE_SIZES`
constants in `src/provider/mod.rs`, consumed by mcp.rs) — proof the pattern works; this plan
extends it to the remaining per-domain enums. Deferred from plan-008 Issue 3.5; tracked as
bead `naba-lca`.

## Scope Decisions (operator-confirmed, 2026-07-21)

- **Approach: Rust constants + golden test.** Hoist every per-domain enum to shared Rust
  constants (extending the `VALID_ASPECT_RATIOS` pattern); `mcp.rs` consumes the constants
  instead of inline literals. Add a **parity/golden test** asserting the `commands/*.md` tables
  and `cli.rs --help` doc-comments match the constants. Markdown stays hand-authored; the test
  catches drift. **Rejected:** full build-time generation of `commands/*.md` — it fights
  `build.rs`'s byte-identical `cli/` render invariant (the `SPEC-EMBED-002` tree hash), a
  disproportionate redesign.
- **Surface scope: all three copies.** The golden check covers `src/mcp.rs` schemas,
  `skills/naba/commands/*.md` tables, **and** `src/cli.rs --help` doc-comments — eliminate all
  three drift sites in one pass.

## Out of Scope

- Build-time generation of the skill markdown (rejected above; the byte-identical CLI render
  invariant stays intact).
- `aspect`/`resolution` (already single-sourced — untouched except where a test formalizes it).
- Any change to the *values* of the enums or to command behavior — this is a pure
  de-duplication + drift-guard refactor.

## Upstream Issues
| Issue | Title | Disposition | Notes | Resolved By |
|:------|:------|:------------|:------|:------------|
| _(none — no open GitHub issue; tracked locally as bead `naba-lca`)_ | | | | |

## Investigation Findings

Full detail in [`findings/exp-001-enum-inventory.md`](findings/exp-001-enum-inventory.md). Summary:

- **18 per-domain enums** need single-sourcing (style, variation, generate-format, icon
  style/corners/format, pattern style/colors/density/repeat, diagram type/style/layout/complexity/
  colors, story style/transition/layout). Each is an **inline `"enum": [...]` literal** in
  `src/mcp.rs` `tools()`, re-typed in `src/cli.rs` `--help` doc-comments and `skills/naba/commands/*.md`
  tables. `aspect`/`resolution` are already single-sourced (`VALID_ASPECT_RATIOS`/`VALID_IMAGE_SIZES`,
  `src/provider/mod.rs`) and are excluded.
- **No active drift today** — all sites agree verbatim. This plan is **regression prevention**, not
  a bug fix. That lowers risk (no behavior change to reconcile) and means the golden test's first
  run should be green.
- **Single-source home:** a new leaf module `src/enums.rs`. **Domain-qualified constant names**
  (`<COMMAND>_<PARAM>_VALUES`, e.g. `GENERATE_STYLE_VALUES`, `ICON_STYLE_VALUES`,
  `DIAGRAM_LAYOUT_VALUES`, `STORY_LAYOUT_VALUES`, `GENERATE_FORMAT_VALUES`, `ICON_FORMAT_VALUES`)
  — there is **no single `STYLE_VALUES`**: `style` occurs 5× (generate/icon/pattern/diagram/story),
  `format`/`colors`/`layout` each 2×, so a bare per-param name would collide (red-team C1). Imported
  by `mcp.rs` (as it already imports provider constants) and `cli.rs`. Leaf module → no dependency
  cycle.
- **Golden test = a Rust `#[cfg(test)]` module inside `src/`.** The crate is binary-only (no
  `src/lib.rs`), so an external `tests/*.rs` can't `use` crate items; an in-`src` test module
  references the constants directly. It parses the two *guarded* sites — clap help (via
  `CommandFactory` introspection, not source parsing) and `commands/*.md` tables (read from
  `CARGO_MANIFEST_DIR`) — and asserts each equals the constant (comma-split, ordered).
- **The core rationale for a golden test (not pure generation):** clap `///` doc-comments and md
  tables are compile-time string literals — they **cannot interpolate a `const`**. Only `mcp.rs`
  can *reference* the constant; the other two sites stay hand-authored text that the test *guards*.
- **Landmines:** (a) `quality` (`fast,high`) is **prose** in cli.rs and **absent** from md — exclude
  from the strict ordered golden (bespoke membership check at most); (b) generate `format` is
  **absent from mcp.rs** — the golden's site-map must not assert an mcp site for it; (c) hyphenated
  values require comma-split, never whitespace-split; (d) param-name mismatches across sites
  (`variation`/`variations`, `type`, `tile-size`/`size`) — the site-map handles these.
- **Fourth drift site:** `tests/parity/test_mcp.py` holds its own hardcoded `EXPECTED` enum copy —
  kept as an **independent oracle** (deliberately not folded into the single source, so it
  cross-checks the refactor rather than trusting it).

## Approach

1. **Create `src/enums.rs`** — one `pub const <NAME>_VALUES: &[&str]` per enum (18 constants),
   values verbatim from the current inline literals. Register `mod enums;` in `src/main.rs`.
2. **Refactor `src/mcp.rs`** to reference `crate::enums::*` instead of inline `"enum": [...]`
   literals for the 18 enums (the `quality` bare literal and free-form fields untouched). This is
   the genuine de-duplication: `mcp.rs` now *derives* from the single source.
3. **Add the Rust golden test** (`#[cfg(test)] mod enum_golden` in `src/enums.rs`) asserting, for
   each single-sourced enum, that the clap `--help` list (via `CommandFactory` introspection) and
   the `commands/*.md` table cell equal the constant — comma-split, ordered, `quality` excluded.
   **The golden compares constant ↔ {clap help, md cell} only** — it does **not** parse `mcp.rs`
   (mcp agreement is compile-time identity via the shared constant). The mcp-specific exceptions
   (`format` has no mcp site; `variation`/`variations`, `type`, `tile-size`/`size` name mismatches)
   govern which literals **Issue 1.2** replaces, **not** what the test asserts (red-team C4).
4. **Leave `cli.rs` help + `commands/*.md` as hand-authored text** (they can't reference a const)
   — now *guarded* by the golden test. Verify they already match (green first run).
5. **Keep `tests/parity/test_mcp.py` `EXPECTED`** as the independent oracle on the MCP schema
   output.

## Epics

### Epic 1: Single source + mcp.rs de-duplication
The core refactor. After this epic, `src/mcp.rs` derives its 18 enums from one place.
- Issue 1.1: Create `src/enums.rs` with 18 `pub const <NAME>_VALUES: &[&str]` constants (values
  verbatim from the current `mcp.rs` inline literals per the E1 inventory table) and register
  `mod enums;` in `src/main.rs`. No behavior change yet (constants unused).
- Issue 1.2: Refactor `src/mcp.rs` `tools()` to reference `crate::enums::*` for the 18 enums,
  replacing the inline `"enum": [...]` literals. Leave `quality` (bare literal) and free-form
  fields (`background`, `tile-size`/`size`) untouched. `cargo build` + `cargo test` green
  (existing `test_mcp.py` `EXPECTED` oracle still passes → proves the schema output is byte-stable
  across the refactor).
  - depends-on: 1.1

### Epic 2: Golden drift-guard test
Enforce that the two *guarded* sites (cli help, md tables) never drift from the single source.
- depends-on: Epic 1
- Issue 2.1: Add `#[cfg(test)] mod enum_golden` in `src/enums.rs`. For each single-sourced enum:
  (a) introspect clap via `<Cli as clap::CommandFactory>::command()`, walking
  `command().get_subcommands()` → `.get_arguments()` (the enum args live on subcommands, with
  `ImageConfigArgs` flattened in), matching each arg on **`Arg::get_long()`** — the clap **long
  name**, not the field-derived id (`diagram --type` has id `diagram_type`; `--tile-size` has id
  `tile_size`) so an id-keyed lookup would silently miss and false-fail (red-team C2). Extract the
  help string, take the substring between the **first `(` and `)`** as the value list (all 18
  single-sourced help strings have exactly one paren group; `quality`/`aspect`/`resolution` — the
  multi-paren/`e.g.` cases — are excluded), assert equals the constant. (b) Read the mapped
  `skills/naba/commands/<cmd>.md` from `CARGO_MANIFEST_DIR`; the table layout **varies** —
  `generate.md` is 4-col (enum in col 4), icon/pattern/diagram/story are 3-col (enum in col 3) — so
  **match the row by flag name (col 1) and take the last non-empty cell** as the value list, never a
  fixed column index (red-team C3). **Exclude `quality`** (prose in cli, absent in md). Comma-split
  + trim; ordered compare. Must be **green on first run** (E1: no active drift).
  - depends-on: 1.2
- Issue 2.2: Wire the new test into the project's check surface so CI runs it. The guard runs under
  **`cargo test`** (the in-`src` `#[cfg(test)]` module is picked up by default) — confirm the
  repo's existing CI already invokes `cargo test` (`.github/workflows/ci.yml`) so no new CI wiring
  is needed; note it in the parity/test docs if a test inventory exists. Sanity-check the guard
  actually **fails on injected drift** (temporarily edit one md cell, confirm `cargo test` goes red,
  revert) — proving the guard is correct, not merely green.
  - depends-on: 2.1

## Gates
### Start Gate (mandatory)
- Type: human
- Approvers: operator

_No capability gates: single naba repo, no cross-repo or external dependency. No upstream issues,
so no reconcile gate — completion closes bead `naba-lca`._

## Risks & Mitigations

| Risk | Mitigation |
|:-----|:-----------|
| **Golden test too brittle** — md formatting or help paraphrasing makes an exact match impossible | E1 confirmed all 18 agree verbatim today; comma-split + trim normalization; `quality` (the one prose/absent case) is excluded, not forced. The test parses via clap introspection (not source scraping) to stay refactor-proof. |
| **mcp.rs refactor changes the emitted schema** (a regression in the MCP tool output) | `tests/parity/test_mcp.py` `EXPECTED` is kept as an **independent oracle** — it still asserts the exact schema, so any accidental change to the emitted enums fails a pre-existing test. Constants are copied verbatim from the current literals. |
| **Binary-only crate blocks the test** | Confirmed (E1): use a `#[cfg(test)] mod` **inside `src/`**, not an external `tests/*.rs` — it references crate items directly. |
| **Site-map exceptions missed** (generate `format` no-mcp, name mismatches) → false test failures | E1 enumerated every exception; Issue 2.1 encodes them explicitly; Issue 2.2's injected-drift check confirms the guard is correct, not just green. |
| **Scope creep into aspect/resolution or free-form fields** | Explicitly out of scope; the 18-enum inventory (E1) is the exact work list. |

## Success Criteria

- `src/enums.rs` exists with 18 `pub const <NAME>_VALUES` constants (values verbatim from the E1
  inventory) and is registered in `src/main.rs`.
- `src/mcp.rs` references `crate::enums::*` for all 18 enums — **no inline `"enum": [...]` literal**
  remains for them; `quality` and free-form fields are unchanged.
- A Rust `#[cfg(test)] mod enum_golden` asserts the clap `--help` lists and `commands/*.md` tables
  equal the constants for every single-sourced enum, with the documented exceptions; it is **green
  on `cargo test`** and **fails on injected drift** (verified).
- `tests/parity/test_mcp.py` still passes unchanged (schema output byte-stable across the refactor).
- No change to any enum's values or command behavior. Bead `naba-lca` closed.
