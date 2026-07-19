# Red-Team Review — pass 2

**Plan:** plan-007-james-dixson-6b5a59
**Date:** 2026-07-19

## Verdict: APPROVE

Second-pass review after the pass-1 REVISE revisions. All eight pass-1 concerns verified
genuinely resolved in the plan text (each traced to specific lines), no dangling Epic-6
references, findings still drive every resolution.

## Verification of pass-1 resolutions
- **#1 config-migration (high):** §Scoping "SELECTED — auto-migrate; confirmed at the Start Gate";
  Start Gate condition folds in the confirmation; "unselected" language gone. ✓
- **#2 api_key mapping (high):** Issue 1.3 spells out per-key mapping (`api_key`→`gemini` regardless
  of `provider`) + the "openrouter default + stray gemini api_key" test. ✓
- **#3 Epic 1 build seam (med/high):** Issue 1.6 pre-registry shim keeps Epic 1 green; matches
  exp-002's named call sites. ✓
- **#4 Epic 6 split (med):** removed as an epic; note + §Phasing + SC8 consistently point to
  plan-008; no dangling references. ✓
- **#5 --json envelope (med):** Issue 2.4 defines the universal envelope SPEC clause + enumerating
  parity test. ✓
- **#6 N-provider precedence (med):** Issue 2.1 ordered-scan tie-break + multi-cred tests; SPEC §5
  via 2.5. ✓
- **#7 default-model fallback (low):** Issue 1.1 compiled-in fallback. ✓
- **#8 docs 5.1 Bedrock coupling (low):** 5.1 Bedrock rows a separable sub-item gated on 3.3. ✓

## Strengths
- Revision edited load-bearing text (Scoping, Issues, Gates, Success Criteria), not bolt-on prose;
  the pass-1 "unselected vs committed" contradiction fully retired.
- Epic 6 split is clean — no orphaned dependencies; tail-coupling worry evaporated.
- Issue 1.6 is the right fix and honest that 2.1 supersedes it.
- No resolution invented facts unsupported by the experiments.

## Concerns (all LOW — none block; folded into the plan as executor notes)
| # | Severity | Concern | Resolution |
|:--|:--|:--|:--|
| L1 | low | Throwaway shim (1.6 vs 2.1) — 1.6 writes an adapter 2.1 deletes | Issue 1.6 note added: keep it the thinnest adapter that compiles; no abstraction 2.1 would rip out. |
| L2 | low | Intra-epic build redness 1.1↔1.6 (1.1 breaks call sites until 1.6) | Issue 1.6 note added: land 1.1→1.6 as a single green boundary so change-validation FAST isn't spuriously red. |
| L3 | low | 1.4 adds config `--json` before 2.4 defines the envelope | Issue 1.4 note added: treat 1.4's config envelope as provisional; 2.4 finalizes the shape. |
| L4 | low | 5.1 Bedrock rows gated only in prose | Issue 5.1 note added: if 5.1 closes before Bedrock ships, file the Bedrock rows as a `discovered-from` follow-on off 3.3. |

## Missing
- Nothing blocking. The pass-1 evidence gap (no yoshiko-flow recon) is resolved by removing Epic 6;
  recon is deferred into plan-008's own investigation.

## Gate Assessment
- Start Gate: now carries the migration-strategy confirmation (pass-1 #1's chosen resolution). ✓
- bedrock-transport gate: unchanged, exemplary. No superfluous gates.

## Upstream Assessment
- #7 / #8 dispositions unchanged + correct.
- **plan-008 follow-on** must be filed as an actual `bd` bead at intake (not left in prose) —
  land-time note added to the Epic-6-split block.

## Sequencing / Dependency Assessment
- config → registry → Bedrock → MCP → docs spine intact; Epic 2→1, Epic 3→2.1 sound; Epic 4
  independent/parallelizable; Epic 1→2 build seam addressed by 1.6. No cycles, no dangling refs.

**Ready. Remaining items are low-severity polish an executor handles inline; no further cycle
warranted.**

## Operator Resolutions
| Item | Resolution | Status |
|:--|:--|:--|
| L1 throwaway shim | Executor note added to Issue 1.6 (thinnest adapter). | resolved |
| L2 intra-epic redness | Executor note added to Issue 1.6 (land 1.1→1.6 as one unit). | resolved |
| L3 1.4 envelope pre-2.4 | Executor note added to Issue 1.4 (envelope provisional, finalized by 2.4). | resolved |
| L4 5.1 Bedrock rows | Executor note added to Issue 5.1 (follow-on off 3.3 if 5.1 closes first). | resolved |

**Final status: APPROVE; all low concerns folded into the plan as executor notes (2026-07-19).**
