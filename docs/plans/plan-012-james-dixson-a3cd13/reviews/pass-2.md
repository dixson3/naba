---
type: Review
okf_spec: OKF-PLAN
plan: plan-012-james-dixson-a3cd13
pass: 2
verdict: APPROVE
date: '2026-07-21'
---
# Red-Team Review — pass 2

**Plan:** plan-012-james-dixson-a3cd13
**Date:** 2026-07-21
**Verdict:** APPROVE
**Status:** resolved

## Verdict: APPROVE

## Strengths
- All 7 pass-1 concerns resolved and technically coherent against source (`record_target` at
  skills.rs:281 fires after deploy; `targets_for` 216-230 drops `skills`+`scope`; marker at embed.rs:47).
- Ordering fix real; report shape (`gc: [{name,path,outcome}]`) cleanly specified; `cargo build &&
  cargo test` gate proportionate; 2.3 sharpened to `parse-changelog` + heading-vs-tag check.

## Concerns
| # | Severity | Concern | Recommendation | Status |
|:--|:---------|:--------|:---------------|:-------|
| 8 | medium | Shared-path dedup unions recorded skills on read but write-back (step 5) is worded singular; the other row keeps the phantom name → forever `skipped_no_marker` noise (not dangerous) | Write the post-GC `skills` set back to ALL rows that dedup to the removed path — mirror read-union with write-fan-out | resolved |
| 9 | low | The `(harness, path)` fallback key collides with the existing `(harness, scope, path)` upsert key | Prefer the scope-carry branch; if `(harness,path)` chosen, note it needs an upsert variant | resolved |

## Missing
Nothing blocking. Pass-1 Missing items (backout/irreversibility, JSON schema, partial-removal) all now covered.

## Gate Assessment
Sound — Epic 1 web gate named; Epic 3 `cargo build && cargo test` gate blocks 3.5. No superfluous gates.

## Upstream Assessment
Clean and unchanged: three includes, each resolved by one epic, `resolves-upstream` on terminal issue.

## Operator Resolutions
| Concern | Resolution |
|:--------|:-----------|
| 8 | Approach step 5 + Issue 3.4 now write the post-GC skill set back to ALL registry rows that dedup to the removed path. |
| 9 | Issue 3.2/3.6 now state the scope-carry branch is preferred; the `(harness,path)` alternative is noted as needing an upsert variant. |
