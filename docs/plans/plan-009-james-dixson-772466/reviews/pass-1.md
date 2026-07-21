# Plan Red-Team — pass-1: plan-009-james-dixson-772466

## Verdict: APPROVE

No high-severity blocker. The two riskiest technical claims — namespace/traceability isolation
(`AGENT-TOOLS-*` cannot match `check_traceability.py`'s `SPEC-` regex) and cross-repo containment
(Epic 3 stays outside naba's merge-back) — were verified against the actual naba machinery. The
concerns below are medium/low refinements folded in during drafting, not gate-stoppers.

## Strengths

- Namespace decision verified: `AGENT-TOOLS-*` adds zero required parity coverage.
- Cross-repo isolation correctly constructed: naba pushes first (stable URL), yoshiko-flow edit is a
  separate operator-authorized commit in its own git authority, explicitly outside naba's Phase-6.
- surface↔harness conflict handled without over-reach (neutral phrasing + Deltas subsection).
- Guardrail compatibility addressed head-on (Guardrails section; MCP axis optional).
- `cross-ref` is a real DRIFT-CHECK edge category; §0 re-approval acknowledged; portable-template
  correctly a follow-on bead.

## Concerns

- **C1 — drift/agreement net covers only the skills axis** — severity: medium
  There is no DRIFT-CHECK node for `mcp.md` or `json-output.md` (only `skill-spec`=skills.md,
  `ig-configuration`/`edd-core`=configuration.md). A single `cross-ref` edge to `skill-spec` leaves
  the MCP and JSON axes' reference-impl mappings as unenforced prose. Since `AGENT-TOOLS-*` is
  outside parity, that one edge is the *entire* automated net and silently misses 2/3 of the SPEC.
  Recommendation: add `cross-ref` edges to nodes for `mcp.md` + `json-output.md`, OR state
  explicitly (plan + SPEC) that drift coverage is skills-axis-only and MCP/JSON mappings are
  agreement-by-review — a named, accepted limitation.

- **C2 — #12 close keys on beads-closed, not on the yoshiko-flow pointer actually landing** —
  severity: medium
  The Reconcile Gate is auto (all beads closed), but Epic 3's artifact lands in a separate repo
  outside beads authority. Issue 3.1 could close with the yoshiko-flow commit made-but-unpushed (or
  not made), yet #12's close asserts "the cross-reference landed."
  Recommendation: add an explicit verification (on 3.1 / the Reconcile Gate) that the yoshiko-flow
  pointer commit is pushed to its remote (`git -C … log`/`ls-remote`) before #12 closes.

- **C3 — mapping-table citation form can perturb traceability first-marker-wins** — severity: low
  `parse_clauses` sorts `*.md` and uses `setdefault`; `agent-tools.md` sorts before `skills.md`.
  A `**SPEC-EMBED-003** [PINNED]` bold+bracket form in the mapping table would make `agent-tools.md`
  the canonical definition site (and a sloppy marker could flip a clause to required-but-uncovered).
  Recommendation: authoring constraint — the mapping cites **bare** `SPEC-*` IDs only, never the
  `**SPEC-…** [MARKER]` form.

- **C4 — Phase-6 push ordering is unusual and under-described** — severity: low
  The naba push must happen *before* Epic 3 (gate condition), so naba's push is not the final action.
  Recommendation: state the naba-lands+pushes → Epic 3 → #12-close order explicitly so an executor
  doesn't batch Epic 3 into the pre-push tree.

## Missing

- Verification the yoshiko-flow pointer commit is pushed before #12 closes (C2).
- Explicit statement of how MCP/JSON-axis agreement is assured outside parity + the single drift
  edge (C1).
- Authoring guardrail on SPEC-ID citation form in the mapping table (C3).
- Minor: the SPEC's MCP axis should note the interface is **stdio-based, not an HTTP server**
  (GR-011-style insurance for the tool-agnostic claim).

## Gate Assessment

Three gates appropriate + non-redundant. Cross-repo landing gate well-specified (concrete,
executable Test; asserts yoshiko-flow's separate git authority). Gap: it verifies the naba side
lands before the edit but nothing verifies the yoshiko-flow side lands before #12 closes (C2) — fix
by adding that check to the Reconcile Gate / Issue 3.1.

## Upstream Assessment

Single issue #12, `include`, closed by this plan — well-justified (residual #12 scope after
plan-008). No supersedes/partials. Deferred portable-template correctly a follow-on bead. Caveat
ties to C2: the close should be gated on evidence the yoshiko-flow cross-reference actually landed.

## Operator Resolutions

| # | Concern | Severity | Resolution | Status |
|:--|:--------|:---------|:-----------|:-------|
| C1 | Drift net covers only skills axis | medium | Plan + SPEC state drift coverage is **skills-axis-only**; MCP/JSON mappings are agreement-by-review (named accepted limitation) AND Issue 2.2 adds `cross-ref` edges for `mcp.md` + `json-output.md` where nodes are cheap to add. | resolved |
| C2 | #12 close not gated on yf pointer pushed | medium | Reconcile Gate + Issue 3.1 close condition now require the yoshiko-flow pointer commit be **pushed to its remote** (verified via `git -C … log`/`ls-remote`) before #12 closes. | resolved |
| C3 | Mapping citation form perturbs traceability | low | Issue 1.2 authoring constraint added: mapping cites **bare** `SPEC-*` IDs only, never `**SPEC-…** [MARKER]`; Issue 2.2 verifies GREEN. | resolved |
| C4 | Push ordering under-described | low | Approach + Epic 3 state the explicit naba-lands+pushes → Epic 3 → #12-close order. | resolved |
| M5 | MCP axis stdio-not-HTTP note | low | Issue 1.1 notes the MCP interface is **stdio-based, not an HTTP server** (GR-011 insurance). | resolved |

**Status: resolved** — all concerns folded into the plan during drafting; verdict stands at APPROVE.
