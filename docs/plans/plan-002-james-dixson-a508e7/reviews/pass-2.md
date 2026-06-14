# Review pass-2 — plan-002-james-dixson-a508e7

**Date:** 2026-06-13
**Conformance:** PASS (all 6 checklist items; revisions internally consistent — 2.4→2.5 edge resolves, SC6/SC7 map to real issues, graph acyclic).
**Adversarial verdict:** APPROVE

## Pass-1 resolution verification (each checked against the repo, not just plan text)

| Concern | Status | Evidence |
| :-----: | :----- | :------- |
| C1 arg/router unverified | resolved | Epic 0 / Issue 0.1 spike placed before Epic 1 (1.1 depends-on 0.1) with "negative = design pivot" escape; SC6 asserts it. |
| C2 subagent repo-path | resolved | "Context delivery" rejects repo-path read, names deployed path, routes (a)/(b) to 0.1. Grounded in `install.py` rsync into `resolve_dests`. |
| C3 DRIFT-CHECK underspec | resolved | 2.4 gives edge-by-edge disposition; each verified against live manifest (delete `e-depends-on-skill`, retarget `e-index-table`/`e-cli-subcommand`); `approved: no` step present. |
| C4 trigger precision | resolved | 1.1 requires preserving external SKIP boundaries + phrase list; `naba-diagram`→`diagram-authoring`/`mermaid` boundary confirmed real. |
| C5 nesting/allowed-tools | resolved | 3.1 mandates end-to-end composite call, not review-only. |
| C6 `story` tier | resolved | `naba story` confirmed single CLI invocation emitting N frames; inline tier correct. |
| M1 orphaned `/naba-*` | resolved | Traced `install.py --uninstall` derives removal set from repo scan → real orphan bug; 2.2 mandates uninstall-before-update. |
| M2 rollback | resolved | Rollback row added to Risks. |
| M3 SC don't assert validation | resolved | SC6 + SC7 added. |

## Strengths

- Findings independently confirmed: 7 inline verbs are real cobra commands; 3 composites have none; the three duplicated blocks are verbatim.
- `allowed-tools` union accurate (`naba-batch` carries `Glob, Write`).
- Dependency graph clean; 2.4 depends-on Epic 1 + 2.5 correctly orders spec before manifest.

## Concerns (residual, non-blocking)

| #  | Severity | Concern | Recommendation | Status |
| :- | :------: | :------ | :------------- | :----- |
| N1 | medium | `allowed-tools` reasoning conflates parent-spawn vs child-execution grants; composite Bash/Write/Glob lives in the child, not the parent skill frontmatter. | 0.1 confirms where the grant lives; 3.1 exercises a composite that writes a file. | resolved — folded into 0.1 step 4 + 3.1 |
| N2 | low | 2.4 silent on the four README/installer edges (`e-readme-prereqs/usage/desc`, `e-installer-frontmatter`) that currently exist per-skill. | Explicitly retarget each to the single SKILL.md + commands/*.md. | resolved — added to 2.4 |

## Missing

None blocking. M1–M3 closed; N1/N2 were execution-time refinements, now folded in.

## Gate Assessment

Single human Start Gate appropriate; Epic 0 functions as the de-facto pre-authoring capability gate. Reconcile gate correctly omitted.

## Upstream Assessment

Clean — no open issues. File Epic 0 spike, pre-merge uninstall step, and N1/N2 as beads at execution.
