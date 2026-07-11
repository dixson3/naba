# Red-Team Review — pass 2

**Plan:** plan-004-james-dixson-9a7b16
**Date:** 2026-07-11

## Verdict: APPROVE

Second red-team cycle, re-reviewing the pass-1 REVISE resolutions. Every pass-1
resolution landed in plan.md and is substantively adequate (not merely claimed). The
revisions introduced no dependency cycle, no orphaned dependency edge, and no broken
issue reference. The dependency DAG is acyclic with every edge pointing at a real
issue. Ready for execution.

## Verification of pass-1 resolutions (all confirmed)

C1 (2.1→SPEC edge), C2 (live smoke pulled to 2.6 + 5.3 depends-on 2.6 + accepted-risk
stated 3×), C3 (per-provider quality in 2.2/2.5/SPEC §5/1.3), C4 (Issue 4.0 embed infra
+ post-cutover skills upgrade), C5 (zero-rewrite migration default), C6 (multi-key
reroute documented), M1 (list_images owned by 4.4), M2 (skills filesystem tests in
4.3), M3 (version-injection in 2.1), M4 (composites note in 1.1), U1 (upstream table
with naba-a3a carry-forward) — all present and verified against the Go source.

## Strengths

- Dependency graph internally consistent and acyclic. The 5.3→2.6→2.4 chain gates the
  OpenRouter-enabling cutover on the live smoke while leaving mocked API-surface work
  (Epic 2/4) ungated — precisely the C2 fix.
- The C2 accepted-risk escape hatch is explicit in three places, so a permanently-
  blocked 2.6 does not deadlock the plan.
- 4.0 correctly sequenced ahead of its consumer 4.3.

## Concerns

1. **[low] Two stale references to retired Issue 5.1** — the live-keys gate header and
   one risk bullet named 5.1 (now 2.6). *Rec:* change both to 2.6.
   **RESOLVED in place (2026-07-11):** gate header → "blocks Issue 2.6"; risk bullet →
   "live-key smoke tests (2.6)". Both edits applied before freeze.

## Missing

None. All four pass-1 Missing items are now owned by concrete issues.

## Gate Assessment

Three gates, no inflation. Start Gate standard; provider-layer gate RESOLVED (BESPOKE);
live-keys gate now tightened — blocks 2.6, and 5.3's OpenRouter cutover transitively
depends on 2.6, closing the "ships without a real call" gap. Test condition valid and
executable. (Stale header label fixed per Concern 1.)

## Upstream Assessment

Scan complete and reasonable. GitHub #1–3 closed, no open issues; local bead naba-a3a
gets a sound `include (carry-forward)` disposition wired to concrete issues (2.3, SPEC
§4, 2.4). No "superseded" dispositions expected — no open feature backlog to supersede.
Adequate.

## Operator Resolutions

| # | Concern (sev) | Resolution | Status |
|:--|:--|:--|:--|
| 1 | Stale Issue 5.1 references (low) | Gate header + risk bullet changed 5.1 → 2.6 before freeze. | resolved |

**Final status:** frozen — APPROVE, sole low-severity concern resolved 2026-07-11.
