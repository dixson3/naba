# Red-Team Review — Pass 2

**Plan:** plan-008-james-dixson-24173a
**Date:** 2026-07-20
**Final status:** RESOLVED (blocker + 2 low refinements addressed in plan v3; frozen — re-reviewed in pass-3)

## Verdict: REVISE

The revision genuinely resolves 4 of 5 pass-1 concerns (A, C, D, E) and every Missing item with
substance. Concern **B** is resolved in *content* (the two DRIFT-CHECK-update issues exist) but its
**sequencing is prose-only, not wired into the bead dependency edges** — re-introducing the exact
on-edit-engine failure B was raised to prevent. That is the sole blocker; the fix is two edges.

## Strengths (verified against code)

- **A fully resolved.** Issue 3.0 pins `$OUT_DIR` render, keeps skill root = `naba`, byte-identical
  `cli/` render against the real pin (`embed.rs:348`), forced-upgrade fallback documented.
- **C resolved, over-delivers.** All four pages carry `--surface` (skills.md: 8 occurrences); 1.5
  updates all four even though only two `e-web-*` edges mechanically fire — config/mcp aren't
  cli-source-derived, so updating them anyway is correct.
- **D resolved cleanly.** Gate test pinned to Issue 1.3 (no SPEC dep); Epic 2 dropped from Blocks;
  4.2 non-gating. No circular gate dependency.
- **E resolved.** 2.3 dedupes by resolved absolute path; 4.1 documents the codex↔agents overlap.
  Migration ambiguity moot (legacy only ever wrote `agents`; codex is new).
- **M2 well-integrated.** 4.3 live smoke-test (`command -v`-gated) + two-tier gate keeps CI on the
  portable baseline.

## Concerns

| # | Severity | Concern | Recommendation |
|:--|:---------|:--------|:---------------|
| B-residual | **medium-high** (blocker) | "Sequence before" is prose-only, not edge-enforced. (1) 3.0b must precede 3.2's templatizing edit, but 3.0b `depends-on: 3.0` and 3.2 `depends-on: 3.1` — parallel branches off 3.0, nothing forces 3.0b first → on-edit engine can FAIL. (2) 5.3 must precede 5.2's deletions, but both only `depends-on: 5.1` — 5.2 can delete `IG/*`/`EDD/CORE.md` before 5.3 re-points the nodes → dangling-node FAIL. | Add `depends-on: 3.0b` to Issue 3.2 (or 3.1) and `depends-on: 5.3` to Issue 5.2. Ordering must live in the edges, not prose. |
| B-refine-1 | low | 3.0b says "re-glob skill-md/commands," but under 3.0's recommended `$OUT_DIR` option the source does **not** move — only the `e-installer-skillset` contract text + §0 re-approval are needed; the re-glob is real only under committed `skills/{cli,mcp}/`. | Make the re-glob conditional on 3.0's outcome; keep contract-text update + §0 re-approval unconditional. |
| B-refine-2 | low | After 3.2 the `skill-md` node contains Jinja and the installer deploys the *rendered* `cli/` tree, not source — so `e-installer-skillset`'s "deploys exactly the on-disk skill set" now compares against a template. | Add one sentence in 3.0b's contract update noting the source-is-now-a-template semantic shift, so §0 re-approval captures it. |

## Missing

Nothing new of substance — all pass-1 Missing items are present and concrete. The only gap is the
edge-wiring under Concern B.

## Gate Assessment

- Start Gate: appropriate.
- Harness path validation (auto): correctly two-tier; test pinned to 1.3, blocks 4.2 only, not Epic
  2. Minor harmless wording tension (condition says "matching the SPEC descriptor" while 1.3 is pure
  `resolve_dest`). No cycle from 4.3 `depends-on 2.2`.
- Embed parity preserved (auto): well-formed; pass hinges on 3.0's byte-identical decision. Soft
  documented risk if forced-upgrade needs Epic 2's receipt (Epic 3 "may overlap Epic 2").
- Reconcile Gate (auto, #12 comment): appropriate.

## Upstream Assessment

#12 partial well-dispositioned; 3.5 now a standalone follow-on bead. Ensure that bead + the
descoped `--json` axis are filed/left visible on #12 at land-the-plane.

**Bottom line:** one narrow wiring fix (two `depends-on` edges) upgrades this to APPROVE; everything
else is APPROVE-grade.

## Operator Resolutions

| # | Concern | Resolution | Status |
|:--|:--------|:-----------|:-------|
| B-residual | Prose-only sequencing | Added `depends-on: 3.0b` to Issue 3.2 and `depends-on: 5.3` to Issue 5.2 — ordering now edge-enforced. | resolved |
| B-refine-1 | 3.0b re-glob over-specified | 3.0b re-glob made **conditional on 3.0's outcome**; contract-text update + §0 re-approval kept unconditional. | resolved |
| B-refine-2 | Template-node semantics | 3.0b now states the source-is-now-a-template shift explicitly for the §0 re-approval. | resolved |
