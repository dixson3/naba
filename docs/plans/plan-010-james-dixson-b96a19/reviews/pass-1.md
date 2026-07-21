# Red-Team Review — pass 1

**Plan:** plan-010-james-dixson-b96a19
**Date:** 2026-07-21

## Verdict: APPROVE

## Strengths

- Enum inventory accurate — all 18 rows cross-checked against `src/mcp.rs`, `src/cli.rs`, and
  every `commands/*.md`; values, order, line numbers match. "No active drift / green on first run"
  confirmed.
- Constant-interpolation pattern proven (`VALID_ASPECT_RATIOS` at `provider/mod.rs:52` →
  `"enum": VALID_ASPECT_RATIOS` at `mcp.rs:736`); `&[&str]` constants interpolate identically incl.
  the nested `variations` `items.enum` case.
- Binary-only-crate reasoning correct (`Cargo.toml:13-15`, no `src/lib.rs`); in-`src`
  `#[cfg(test)]` module is right; `Cli` is `pub` so `CommandFactory` is reachable.
- Independent-oracle claim verified (`test_mcp.py` `EXPECTED` genuinely hand-copied).
- No hidden traceability/embed gate broken (`check_traceability.py` needs no `src/*.rs` clause;
  build.rs embeds `skills/`, not `src/`).

## Concerns

| # | Severity | Concern | Recommendation |
|:--|:--|:--|:--|
| C1 | medium | No single `STYLE_VALUES` — there are 5 `style`, 2 `format`, 2 `colors`, 2 `layout` enums; the illustrative name would collide 5 ways. Naming convention unstated. | Issue 1.1 pins a domain-qualified scheme (`GENERATE_STYLE_VALUES`, `ICON_STYLE_VALUES`, `DIAGRAM_LAYOUT_VALUES`, `STORY_LAYOUT_VALUES`, `GENERATE_FORMAT_VALUES`, `ICON_FORMAT_VALUES`, …). |
| C2 | low | clap arg lookup under-specified — must match on `Arg::get_long()` not the field-derived id (`diagram --type` id `diagram_type`; `tile-size` id `tile_size`); enum args live on subcommands with `ImageConfigArgs` flattened. | Issue 2.1 states lookups key on `get_long()`, iterate `command().get_subcommands()→get_arguments()`, and the parser takes the substring between the first `(` and `)`. |
| C3 | low | md table column layout varies — `generate.md` is 4-col (enum in col 4); icon/pattern/diagram/story are 3-col (enum in col 3). A fixed index fails. | Issue 2.1: match the row by flag name (col 1), take the **last** non-empty cell as the value list. |
| C4 | low | Plan frames the name-mismatch / `format`-no-mcp exceptions as golden-test exceptions, but they concern mcp.rs, which the golden does not parse. | Clarify: the golden compares constant ↔ {clap help, md cell} only; the mcp exceptions govern which literals Issue 1.2 replaces. |

## Missing

- Constant naming convention (C1) — the first thing an executor hits.
- Issue 2.2 should name the exact CI command it expects (`cargo test`).
- Finding's optional `VALID_ASPECT_RATIOS` relocation — correctly out of scope; keep it out.

## Gate Assessment

Appropriate and minimal. Single human Start Gate is right for a single-repo refactor; no capability
gate needed; reconcile-gate absence justified (no upstream). Issue 2.2's injected-drift check is a
genuine correctness gate on the guard itself.

## Upstream Assessment

Correct — "none, tracked locally as bead `naba-lca`" is accurate (plan-008 Issue 3.5 deferral, no
open GitHub issue). Nothing to reconcile.

## Operator Resolutions

| # | Concern | Resolution | Status |
|:--|:--|:--|:--|
| C1 | Constant naming collisions | Issue 1.1 amended: pin domain-qualified `<COMMAND>_<PARAM>_VALUES` naming; added an explicit naming-convention note. | resolved |
| C2 | clap lookup key under-specified | Issue 2.1 amended: lookups key on `Arg::get_long()`, iterate subcommand args (`ImageConfigArgs` flattened), parser takes first `(`…`)` substring. | resolved |
| C3 | md table column layout varies | Issue 2.1 amended: match row by flag name (col 1), take last non-empty cell. | resolved |
| C4 | site-map exceptions framing | Approach + Issue 2.1 clarified: golden compares constant ↔ {clap help, md cell}; mcp exceptions govern Issue 1.2's literal replacement, not test assertions. | resolved |
