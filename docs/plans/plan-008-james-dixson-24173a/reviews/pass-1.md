# Red-Team Review — Pass 1

**Plan:** plan-008-james-dixson-24173a
**Date:** 2026-07-20
**Final status:** RESOLVED (all concerns addressed in plan v2; frozen — re-reviewed in pass-2)

## Verdict: REVISE

The approach is sound and well-grounded in the findings. Four concerns (one high, three medium)
plus one low and several "missing" items should be resolved into explicit issues/decisions before
execution. None invalidates the chosen design — they are underspecified seams that the
DRIFT-CHECK on-edit engine and the embed hash-pin will trip on during execution if left implicit.

## Strengths

- Findings are decision-grade (E5/E6/E7 cite exact modules, clause IDs, hash-pin mechanics); the
  three operator decisions are each backed by a finding.
- Harness-as-data is the right abstraction and satisfies the extensibility objective.
- Render-at-build (not install) correctly respects the `deployed==embedded` invariant.
- Receipt design (decouple logical harness from physical path; continue-on-error) improves on
  today's fail-fast `present_surfaces`.
- Upstream #12 partial is specific about in/out scope.

## Concerns

| # | Severity | Concern | Recommendation |
|:--|:---------|:--------|:---------------|
| A | **high** | The two-tree render restructures the **embed root**, not just `mcp.rs` pointers. `include_dir!("$CARGO_MANIFEST_DIR/skills")` + `skill_names()` enumerate `skills/`'s immediate subdirs; rendering into `skills/cli/naba`+`skills/mcp/naba` makes `skill_names()` return `cli`/`mcp` and breaks install/status/marker. Rendering into `$OUT_DIR` forces re-pointing `include_dir!` + a commit/gitignore decision. The embedded-tree hash is a **pinned regression constant** (`embed.rs:16-22`); the `cli/` render must be byte-identical or every install reads "outdated" and the pin must be re-baselined. | Add **Issue 3.0** pinning: (a) render target (`$OUT_DIR` vs committed); (b) how `skill_names`/`skill_files`/`SKILLS` root change; (c) byte-identical render vs accepted forced-upgrade (the latter interacts with receipt migration — a first-run re-upgrade across all recorded harnesses). Add a matching risk row. |
| B | medium-high | **DRIFT-CHECK.md churn spans Epic 3 and Epic 5, but only Epic 5 budgets it.** Epic 3 templatizes `skills/naba/SKILL.md` and adds `cli/`+`mcp/` subdirs — mutating the `skill-md`/`commands` nodes + the `e-installer-skillset` contract — yet no Epic-3 issue touches DRIFT-CHECK. Epic 5 retires `skill-spec`/`ig-configuration`/`edd-core` nodes. The on-edit engine (`approved: yes`) FAILs unless the manifest is rewritten in the same pass, and node changes require operator **§0 re-approval**. | Add a DRIFT-CHECK-update sub-issue under Epic 3 (skill node/glob restructure + `e-installer-skillset`); make 5.3 explicit about node re-pointing + a §0 re-approval step; sequence both before their triggering edits. |
| C | medium | **Web docs reconciliation for the flag rename is unscoped.** Four `web/content/pages/*` pages document `--surface` verbatim; DRIFT-CHECK edges `e-web-skills-lifecycle`/`e-web-install-methods` (cli-source → web, fixed authority) FAIL when `cli.rs` gains `--harness`. Issue 1.4's "help text/prose" reads CLI-only. | Add an issue to update the four web pages for `--harness`/multi-harness; add a success criterion; confirm web docs in-scope. |
| D | medium | **Harness-path gate is ambiguously sourced and over-serializes Epic 2.** Its Test `cargo test harness_paths` "Blocks: Issue 4.2" — but 4.2 authors the SPEC-validation tests (gate blocks the issue creating its own test). Both 1.3 and 4.2 describe path tests. 4.2→4.1→5.1, so gating Epic 2 transitively serializes it behind the SPEC split/authoring, which Epic 2 doesn't need. | Pin the gate test to **Issue 1.3** (pure `resolve_dest` assertions, no SPEC dep); drop 4.2 from "Blocks"; treat descriptor↔SPEC agreement (4.2) as separate verification not gating Epic 2. |
| E | low | **Overlapping physical paths in the receipt.** Portable `agents` and `codex` both resolve to `.agents/skills`; `install --harness codex --harness agents` records two targets with identical path → unqualified upgrade deploys/prunes the dir twice. Migration disk-scan of `.agents/skills` is harness-ambiguous. | Dedupe upgrade enumeration by **resolved absolute path** before deploy/prune; note the codex↔agents overlap in the harness SPEC. |

## Missing

- Web-docs success criterion/issue (Concern C).
- Explicit DRIFT-CHECK **§0 re-approval** step after node changes.
- Rollback/decision for the case where the two-tree render can't preserve the hash pin
  (forced-upgrade path).
- Risk-table rows: embed-root restructure; web-docs drift; receipt path-overlap.
- `doctor` output changes (it surfaces installed surfaces; display prose not called out).
- Extensibility success criterion/test ("adding a harness = one data row + SPEC row").
- The pre-existing skill-md ↔ `mcp.rs` param drift (Issue 3.5 stretch) filed as a tracked bead so
  it survives if 3.5 is descoped.

## Gate Assessment

- Start Gate: appropriate (human/operator).
- Harness path validation (auto): correct strategy (path-assertions substitute for un-runnable
  harnesses), but Test-source ambiguity + Epic-2 serialization need fixing (Concern D).
- Embed parity preserved (auto): well-formed; pass hinges on the byte-identical-render decision
  (Concern A). "`skills status` clean after install" holds even under forced upgrade.
- Reconcile Gate (auto, comment on #12): appropriate and correctly typed.

## Upstream Assessment

#12 partial is well-dispositioned. Ensure the descoped `--json` axis and the skill-md↔mcp.rs drift
follow-on are filed as beads / left visibly on #12 so they aren't lost at land-the-plane.

## Operator Resolutions

| # | Concern | Resolution | Status |
|:--|:--------|:-----------|:-------|
| A | Embed-root restructure + hash pin | Added **Issue 3.0** (decision-first): render into `$OUT_DIR`, keep skill root = `naba`, pursue byte-identical `cli/` render; fallback = re-baselined pin + documented forced re-upgrade. Risk row added. | resolved |
| B | DRIFT-CHECK churn across Epic 3 + 5 | Added **Issue 3.0b** (skill-node re-glob + `e-installer-skillset` + §0 re-approval, before 3.2); **5.3** made explicit (node re-point + §0 re-approval, before 5.2 deletions). Risk row added. | resolved |
| C | Web-docs `--harness` reconciliation | Added **Issue 1.5** (update 4 `web/content/pages/*` for `--harness`, satisfy `e-web-*` edges) + success criterion + risk row. Web docs now explicitly in-scope. | resolved |
| D | Harness-path gate source + Epic-2 serialization | Gate Test pinned to **Issue 1.3** (`resolve_dest` assertions, no SPEC dep); **dropped Epic 2 from Blocks**; 4.2 recast as descriptor↔SPEC verification (non-gating). | resolved |
| E | Receipt path-overlap (agents↔codex) | **Issue 2.3** dedupes upgrade enumeration by resolved absolute path; **4.1** documents the overlap in the SPEC. Risk row added. | resolved |
| M1 | Missing risk rows / criteria / doctor / extensibility / drift bead | 3 risk rows added; **1.4** now covers `doctor` output; extensibility + web-docs + live-smoke success criteria added; **3.5** notes the skill-md↔mcp.rs drift is filed as a standalone follow-on bead; 3.0 records the forced-upgrade rollback. | resolved |
| M2 | Live harness smoke-test (opencode/pi/codex runnable locally) | Verified all three run headlessly with env creds (opencode→Bedrock, pi→OpenRouter, codex→OpenRouter). Added **Issue 4.3** live discovery smoke-test (local tier, `command -v`-gated) + upgraded the harness-path gate to two tiers + success criterion. | resolved |
