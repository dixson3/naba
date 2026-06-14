# Review pass-1 — plan-002-james-dixson-a508e7

**Date:** 2026-06-13
**Conformance:** PASS (mechanical checklist clean; epics/issues, acyclic deps, verifiable success criteria, gates, portability sections all satisfied).
**Adversarial verdict:** REVISE

## Strengths

- Findings verified accurate: 7 cobra subcommands real; batch/brand-kit/storyboard have no Go command; duplication (Global Flags / Anti-Patterns / prompt framing) is verbatim across skills.
- Installer "keep, minimal touch" justified — `install.py` discovers any `skills/<name>/SKILL.md` and rsync `-a` preserves `commands/`.
- Dependency graph clean; 1.4 (remove old dirs) correctly depends on 1.2 (extract content).
- Risk table names real hazards, not generic ones.

## Concerns

| #   | Severity | Concern                                  | Recommendation |
| :-: | :------: | :--------------------------------------- | :------------- |
| C1 | high | `/naba <subcommand>` arg-passing + markdown-router pattern is asserted, not verified. If slash args don't reach the skill body parseably, the whole invocation model fails. | Add a verification spike / capability gate before Epic 1 authoring: confirm a user-invocable skill receives `<subcommand> <args>` and can route on it. Negative result = design pivot. |
| C2 | high | Composite subagents told to "read `skills/naba/SKILL.md`" use the repo path; installed skills live at `~/.claude/skills/naba/` (or `.agents`/`--target`), and a fresh subagent does not inherit parent context. | Router passes the resolved absolute path of `commands/<sub>.md` (+ shared guidance) into the subagent prompt, OR inline the shared guidance into the prompt. Decide in 1.1. |
| C3 | medium | Issue 2.4 (DRIFT-CHECK rewrite) underspecified for an enforced, approved manifest. `e-index-table` (10-row contract), `e-depends-on-skill` (sibling dirs), `e-cli-subcommand` (per-SKILL.md) don't survive trivially. A parseable-but-wrong manifest passes while checking nothing. | Enumerate edge-by-edge disposition (delete/retarget). Drop manifest to `approved: no` until operator re-approves the rewrite. |
| C4 | medium | Trigger-precision regression accepted thinly. The 10 descriptions carry external SKIP boundaries (diagram image vs diagram-authoring/mermaid source) that must survive in one block; auto-trigger gets coarser. | Issue 1.1 must preserve every external SKIP boundary; keep a natural-language phrase list to sanity-check the merged description. |
| C5 | low | Two-deep subagent nesting + subagent `allowed-tools` not verified. | 3.1 smoke test should run a composite subagent through an end-to-end `naba` call, not review-only. |
| C6 | low | `story` classified inline may flood parent context if `naba story` emits many images. | Confirm `naba story` is a single CLI invocation; reconsider tier if it streams heavy per-image output. |

## Missing

- M1: No cleanup for already-installed `/naba-*` skills. After repo dirs are deleted, `install.py --uninstall` can no longer discover/remove them. Document a one-time `./install.sh --uninstall` *before* merge (or manual `rm`) in the breaking-change note.
- M2: No rollback note if the new invocation model proves unworkable post-merge.
- M3: Success criteria don't assert the dispatch/arg contract was validated (the highest-risk assumption).

## Gate Assessment

Single human Start Gate appropriate. Add a small verification spike (C1, C2) gated before Epic 1 authoring so 10 files aren't authored against an invalid model. Reconcile gate correctly omitted.

## Upstream Assessment

Clean — no open issues; nothing to wire. File the spike/cleanup follow-ups as beads at execution.

## Operator Resolutions

| #   | Resolution                               | Status     |
| :-: | :--------------------------------------- | :--------- |
| C1 | New Epic 0 verification spike (Issue 0.1) added; SC6 added to assert it. | resolved |
| C2 | Approach revised: router passes resolved absolute paths into subagent prompt; shared guidance re-delivery made explicit. | resolved |
| C3 | Issue 2.4 expanded with edge-by-edge disposition + `approved: no` re-approval step. | resolved |
| C4 | Issue 1.1 updated to require preserving external SKIP boundaries + phrase sanity-check. | resolved |
| C5 | Issue 3.1 updated to require an end-to-end composite subagent `naba` call. | resolved |
| C6 | Issue 0.1 spike also confirms `naba story` invocation shape. | resolved |
| M1 | Issue 2.2 + breaking-change note: document `./install.sh --uninstall` before merge. | resolved |
| M2 | Rollback note added to Risks. | resolved |
| M3 | SC6 asserts the dispatch/arg contract validation. | resolved |
