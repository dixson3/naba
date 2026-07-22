---
type: Review
okf_spec: OKF-PLAN
plan: plan-012-james-dixson-a3cd13
pass: 1
verdict: REVISE
date: '2026-07-21'
---
# Red-Team Review — pass 1

**Plan:** plan-012-james-dixson-a3cd13
**Date:** 2026-07-21
**Verdict:** REVISE
**Status:** resolved

## Verdict: REVISE

## Strengths
- Investigation is concrete and line-anchored (prune_stale, run_one_target, Target triple, MARKER_PREFIX).
- Manifest-authority + marker-veto is the right shape; `#[serde(default)]` empty-vec makes "unknown = GC nothing", staging first-upgrade safety.
- Upstream mapping clean 1:1; epics genuinely independent.

## Concerns
| # | Severity | Concern | Recommendation | Status |
|:--|:---------|:--------|:---------------|:-------|
| 1 | medium | GC diff read-before-write ordering unspecified; `record_target` fires after deploy and would overwrite the row before the diff reads it → diff always empty | Add explicit ordering: load recorded skills → deploy → GC set = recorded−embedded → remove marker-owned → only then upsert row, dropping only successfully-removed names | resolved |
| 2 | medium | `targets_for` reduces registry to `(harness, PathBuf)`, discarding the `skills` field before GC sees it | Note in 3.4 that the recorded `skills` must be surfaced per resolved path; union on shared-path dedup | resolved |
| 3 | medium | GC'd dirs have no place in the report — `SkillActionItem` iterates only embedded names; `--json`/`--dry-run` has nowhere to render removals | Specify a per-target `gc: [{name, path, removed\|skipped_no_marker}]` field; add to 3.4 + success criteria | resolved |
| 4 | medium | Epic 3 (only code epic, does `remove_dir_all`) has no build/test capability gate | Add a `cargo build && cargo test` auto gate blocking 3.5/reconcile | resolved |
| 5 | medium | Issue 2.3 cargo-dist verification vague; heading-to-tag match (`0.7.0` vs `v0.7.0`) is where it silently fails | Name concrete command (`parse-changelog CHANGELOG.md 0.7.0` / dry-run announce); add heading-format check | resolved |
| 6 | low | Enumerated-upgrade `record_target` uses `opts.scope` not the row's scope; upsert key may diverge and write a duplicate/mis-scoped row | Carry each row's own scope on the enumerated path, or key on `(harness, path)` for upgrade; flag in 3.2 | resolved |
| 7 | low | Web-build gate does not name the command | Name the Pelican build command in gate Instructions | resolved |

## Missing
- Backout/irreversibility note for #18 (`remove_dir_all` runs unattended in `self update`; marker veto is the only guard) — resolved (added to Risks).
- `--json` report schema change for GC outcomes — resolved (concern 3).
- Partial-removal failure semantics — resolved (concern 1 ordering fix).

## Gate Assessment
Start gate + reconcile fine; Epic 1 web-build gate right instinct but unnamed (concern 7); **missing Epic 3 validation gate** (concern 4).

## Upstream Assessment
Sound: three includes, each resolved by one epic, `resolves-upstream` on the terminal issue. No supersedes/partials. No concerns.

## Operator Resolutions
| Concern | Resolution |
|:--------|:-----------|
| 1 | Approach Epic 3 rewritten with explicit (a)–(e) ordering; row upserted only after successful removal. |
| 2 | Issue 3.4 notes `targets_for` must surface recorded `skills` per path; union on shared-path dedup. |
| 3 | Issue 3.4 + Success Criteria specify a per-target `gc: [{name, path, outcome}]` report field. |
| 4 | Added Capability Gate: Epic 3 builds & tests (`cargo build && cargo test`), blocks 3.5. |
| 5 | Issue 2.3 names `parse-changelog CHANGELOG.md 0.7.0` + heading-vs-tag format check. |
| 6 | Issue 3.2 notes enumerated-upgrade path must carry the row's own scope (or key on `(harness, path)`). |
| 7 | Web-build gate Instructions name the Pelican build command. |
