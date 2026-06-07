# Review Pass 1 — plan-001-james-dixson-a9545f

**Date:** 2026-06-07
**Reviewers:** conformance (PASS) → red-team (adversarial)
**Verdict:** REVISE
**Status:** resolved

## Conformance

PASS. All 5 epics have ≥1 issue; dependency graph acyclic with no dangling refs; 6 verifiable success criteria; upstream wiring honest (empty, no reconcile gate); both gates fully specified; all portability sections present.

## Red-team verdict: REVISE

Plan is well-structured and accurately grounded, but leans on two wrong claims about the reference/upstream behavior and one ordering hazard that contradicts the local-only decision. None fatal; all fixable with small edits.

## Strengths

- Findings (F1, F2, F4, F6) verified accurate against the live repo; F1→Epic 3 ordering argument correct (`load_skills()` keys off frontmatter).
- Dependency sequencing sound (1→2→3, 4 parallel, 5 last); shared project-root surface correctly deferred to one reconciling issue.
- Upstream triage honest (empty); capability-loss guard before agent deletion (3.1 blocks 3.4) is the right defensive ordering.
- drift-check manifest design mirrors reference node/edge structure; enforcing run gated behind human `approved: yes`.

## Concerns

| # | Severity | Concern | Recommendation |
|---|----------|---------|----------------|
| C1 | medium | Reference installer has NO `--uninstall` path (verified absent in beads-skills install.py/sh/README). Issues 3.2/3.6 treat it as "adaptation" when it is net-new design; uninstall is fiddly (which dirs; whether to remove keep-existing companion rules). | Either drop `--uninstall` from scope, OR call it out as new design with its own test + explicit companion-rule policy (leave rules unless `--force`). |
| C2 | medium | `beads-upstream init` does NOT write a `## Upstream Tracking` section — the skill says init only runs `bd config set ...`; the trigger contract ships as the companion rule via install.sh. Issue 4.3, F4, Criterion 4 inherit the wrong assumption. | Rewrite 4.3 to "configure via `bd config set` (owner/repo/enabled/backend) + `bd config set dolt.local-only true`"; drop the "persist ## Upstream Tracking section" language from 4.3/F4/Criterion 4. |
| C3 | medium | Local-only decision collides with bd's `Remote Consistency: No remotes configured` warning, whose remediation is to add a dolt remote. "0 errors, warnings resolved" (Epic 4 / Criterion 4) is unsatisfiable without violating Decision 4. | In 4.4 + Criterion 4, enumerate Remote Consistency as a permanently-accepted warning (intentional local-only); reword to "warnings resolved or explicitly accepted." Use `bd config set dolt.local-only true` to assert intent. |
| C4 | low | F3 mis-describes `install.sh` as a "1-line `exec uv run`"; it is ~15 lines with a `uv`-on-PATH guard. (Issue 3.3 is already correct.) | Correct F3 to "thin wrapper with a `uv`-on-PATH guard, then `exec uv run install.py "$@"`." |
| C5 | low | `depends-on-tool: [naba]` makes installed skills warn/inert on machines without the naba CLI; plan never states how end-users get the binary. | Issue 3.5 README must document installing the naba CLI binary as a prerequisite + note the expected warning when absent. |
| C6 | low | `bd setup claude && bd setup codex` run *after* the optimal-instructions slim-down (5.1) may re-inject managed content into the just-thinned CLAUDE.md, partially undoing the K2 move. | Reorder 5.1: run/anchor `bd setup` managed block first, restructure around it, then verify no divergence. |

## Missing

| # | Item | Resolution direction |
|---|------|----------------------|
| M1 | No issue installs/verifies companion-rule delivery end-to-end; naba skills have no `protocols/` dirs and no issue creates them. | State explicitly that naba ships **zero** companion rules (so 3.6 does not expect a `rules/` dir), OR add an issue to author `skills/<x>/protocols/*.md`. |
| M2 | No `LICENSE` issue. Reference ships top-level LICENSE; operator standing requirement = MIT / James Dixson / Yoshiko Studios LLC / current year. (Note: a 1.1 KB `LICENSE` already exists in the repo.) | Add a verify step (LICENSE exists, correct attribution/year). |
| M3 | No step verifying the *old* `/generate` etc. commands are fully gone (a prior plugin install could leave stale commands). | Add to 3.6: verify stale/old commands absent from installed locations. |

## Gate Assessment

Acceptable as written. Start Gate (human) + DRIFT-CHECK manifest approval (human) both justified and minimally scoped. Gate test `grep -q '^approved: yes' DRIFT-CHECK.md` valid and matches reference manifest format. "No reconcile gate" justification sound (verified empty upstream). No over-gating.

## Upstream Assessment

Disposition correct and verified (empty issue list, active `dixson3` token). Forward-only config is the right call; no supersedes. Sole defect = the init-writes-a-CLAUDE.md-section misconception (C2); fix 4.3/F4/Criterion-4 wording and the upstream story is clean.

## Operator Resolutions

| Concern | Resolution | Status |
|---------|-----------|--------|
| C1 (uninstall) | Keep `--uninstall` but reframed as net-new design: dedicated test, companion-rule policy (rules left in place unless `--force`). Reflected in 3.2/3.6. | resolved |
| C2 (upstream init section) | Rewrote 4.3 to `bd config set` (owner/repo/enabled/backend) + `dolt.local-only true`; removed "## Upstream Tracking section" claim from 4.3/F4/Criterion 4. | resolved |
| C3 (Remote Consistency warning) | Criterion 4 + 4.4 now accept Remote Consistency as an intentional, permanently-accepted warning; added `bd config set dolt.local-only true`. | resolved |
| C4 (F3 wording) | Corrected F3 install.sh description. | resolved |
| C5 (naba binary prereq) | 3.5 README must document naba CLI binary install prerequisite + expected warning. | resolved |
| C6 (bd setup ordering) | 5.1 reordered: anchor `bd setup` managed block first, restructure around it, verify no divergence. | resolved |
| M1 (companion rules) | Stated naba ships zero companion rules; 3.6 does not expect a `rules/` dir (installer still supports it for future). | resolved |
| M2 (LICENSE) | Added LICENSE verification to Epic 3 (3.5). | resolved |
| M3 (stale commands) | 3.6 now verifies old `/<name>` commands absent. | resolved |
