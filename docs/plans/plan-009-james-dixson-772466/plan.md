# Plan: Author a tool-agnostic agent-tools SPEC (skills self-management lifecycle, MCP-over-CLI interface, --json agent output with envelopes + pipe-auto-enable) with naba as reference implementation; yoshiko-flow recon + cross-reference; resolves GH #12

**ID:** plan-009-james-dixson-772466
**Author:** james-dixson
**Created:** 2026-07-21
**Status:** complete
**Epic:** naba-mol-d10
**Fingerprint:** 0f63fbbe7d3aa0abe4d3d63af37988f741d19adc379dd33184e8fb915495c17a
**Phase log:**
- 2026-07-21 scoping: initial scope captured
- 2026-07-21 investigating: 3 experiments identified (E1 yf-recon, E2 naba-inventory, E3 spec-structure)
- 2026-07-21 drafting: synthesizing plan (findings sufficient; E3 folded as design decision)
- 2026-07-21 review: red-team pass-1 — APPROVE (5 concerns folded: 2 medium, 3 low)
- 2026-07-21 ready-for-approval: ready-check green — pass-1 APPROVE + audit pass
- 2026-07-21 approved: operator approved
- 2026-07-21 intake: epic naba-mol-d10 poured
- 2026-07-21 executing: start gate resolved
- 2026-07-21 reconciling: post-execution reconciliation
- 2026-07-21 complete: plan complete

## Objective
Author a tool-agnostic agent-tools SPEC (skills self-management lifecycle, MCP-over-CLI interface, --json agent output with envelopes + pipe-auto-enable) with naba as reference implementation; yoshiko-flow recon + cross-reference; resolves GH #12

## Motivation

naba has, across plans 004–008, grown a **coherent, well-specified pattern** for making a
CLI first-class to AI agents: (1) a **skills self-management lifecycle** (binary-embedded skill
tree; `skills install/upgrade/status/remove/preflight`; integrity markers; per-harness idiomatic
layouts + install receipt); (2) an **MCP-over-CLI interface** (every CLI capability exposed as
an MCP tool + skills served as lazy-loaded `skill://` resources, from a dedicated `mcp/` render);
and (3) **`--json` agent-friendly output** (a universal envelope + pipe-auto-enable). Today this
pattern is only documented as **naba-specific** clauses scattered across
`docs/specifications/{skills,mcp,json-output,commands}.md`. There is no **tool-agnostic** statement
of the pattern that another CLI (or the yoshiko-flow toolchain) could adopt as a contract.

GH #12 (split from plan-007) asks for exactly that: a portable "agent-tools" SPEC with naba as the
**reference implementation**, preceded by a **yoshiko-flow reconnaissance** so the SPEC reconciles
with the existing `yf` kernel / `yf-*` skills / `yf-skill-authoring` conventions rather than
duplicating or contradicting them. plan-008 advanced the naba *implementation* of axes 1 & 2 but
explicitly left the portable-SPEC authoring and the yoshiko-flow reconciliation open. This plan
closes #12 by authoring that SPEC and wiring the cross-references.

## Upstream Issues

| Issue | Title | Disposition | Notes | Resolved By |
|:------|:------|:------------|:------|:------------|
| [#12](https://github.com/dixson3/naba/issues/12) | plan-008 seed: portable agent-tools SPEC (skills self-mgmt + MCP-over-CLI + `--json`), reconcile with yoshiko-flow | include | This plan authors the tool-agnostic agent-tools SPEC (all three axes), does the yoshiko-flow reconnaissance, and cross-references from yoshiko-flow — the remaining #12 scope after plan-008 shipped the naba implementation of axes 1 & 2. Closes #12. | (this plan) |

## Scope Decisions (operator-confirmed, 2026-07-21)

- **Plan root:** vault-default `docs/plans/` (naba-repo-rooted; no incubator).
- **SPEC home / yoshiko-flow depth:** author the agent-tools SPEC **in naba**
  (`docs/specifications/`), and **cross-reference** it from yoshiko-flow (a pointer/link). Do
  **not** modify yoshiko-flow's own specs/skills/kernel — the reconciliation is a *reconnaissance
  finding + a reference link*, not source changes in the yoshiko-flow repo.
- **Stretch template deferred:** the portable skill/scaffolding template (a naba-derived scaffold
  any harness-tool could adopt) is **out of scope** for this plan — file it as a follow-on bead.

## Out of Scope

- Any source change inside the `yoshiko-flow` repo (specs, skills, kernel) — reconciliation is
  recon + cross-reference only.
- The portable skill/scaffolding template (deferred follow-on).
- Any change to naba's *implementation* of the three axes — they shipped in plans 004–008; this
  plan documents/abstracts, it does not re-implement.

## Open Questions → Investigation

- **E1 — yoshiko-flow reconnaissance:** survey the `yf` kernel, `yf-*` skills, `yf-skill-authoring`,
  and yoshiko-flow's own `SPEC.md`/specs. What conventions already exist for skill packaging,
  CLI-surface contracts, and agent-facing output? Where does naba's three-axis pattern **agree**,
  **extend**, or **conflict** with them? What is the right cross-reference seam (where in
  yoshiko-flow does a pointer to the agent-tools SPEC belong)?
- **E2 — naba reference-contract inventory:** enumerate the concrete naba clauses that constitute
  each axis — skills lifecycle (`SPEC-EMBED-*`, `SPEC-INSTALL-*`, `SPEC-HARNESS-*`,
  `SPEC-PREFLIGHT-*`), MCP (`SPEC-MCP-*`, incl. skills-as-resources 014/015), `--json`
  (`SPEC-JSON-*`, `SPEC-GLOBAL-003` pipe-auto-enable) — and identify which parts are **naba-specific**
  vs **generalizable** into a tool-agnostic requirement.
- **E3 — SPEC structure + home format (investigate → decide):** propose the agent-tools SPEC's
  document structure (tool-agnostic requirements + a "reference implementation: naba" mapping
  table), its file home under `docs/specifications/`, clause-ID scheme (a distinct `AT-*` /
  `AGENT-TOOLS-*` namespace vs reusing `SPEC-*`), and how it plugs into the traceability +
  DRIFT-CHECK machinery without over-coupling.

## Investigation Findings

Full findings in `findings/`. Summary:

- **E1 — yoshiko-flow recon** (`findings/exp-001-yf-recon.md`): **naba is the documented behavioral
  ancestor of `yf skills`** — yf was reverse-engineered from naba (plan-010 recon). So the SPEC
  reconciles with a *descendant*. Axis 1 (skills): strong AGREE, yf already implements the pattern
  (`REQ-YF-MARK-001..004`, `REQ-YF-CLI-001/002`); naba EXTENDS with a skills-receipt + per-harness
  layouts; **live naming CONFLICT** yf "surface" vs naba "harness"; guardrails `GR-001` (not a
  general skill manager) / `GR-003` (not a runtime) constrain wording. Axis 3 (`--json`): strong
  AGREE — yf has `REQ-YF-CLI-003` + `docs/yf/preflight-contract.md` (status-field authoritative);
  **pipe-auto-enable ABSENT in yf** (naba extension). Axis 2 (MCP-over-CLI): **ABSENT in yf**
  (GR-003), additive/novel. No pre-existing agent-tools doctrine. Cross-ref seam best fit:
  `skills/yf-skill-authoring/reference/SURFACE_CONVENTION.md` "See also" (prose link, **not** a
  drift edge — cross-repo). yf is SPEC-first but a pure pointer is doc-only.
- **E2 — naba contract inventory** (`findings/exp-002-naba-inventory.md`): ~25 **generalizable**
  clauses (a minority of clauses, the majority of the value) across the three axes; the rest
  (MCP-002–010 image-tool schemas, JSON-001 field names, EMBED-004 Go-port migration, all Rust
  mechanics) are naba-specific residue. Three flagship portable contracts: (a) receipt-driven,
  harness-descriptor, integrity-marked skills self-management + fast preflight; (b)
  skills-as-lazy-MCP-resources with every CLI verb mirrored as a tool; (c) documented,
  test-enforced universal `--json` envelope that auto-enables when piped. **Namespace decision:**
  give the agent-tools SPEC its **own ID namespace** (do NOT reuse `SPEC-*` — collision breaks
  naba's traceability/drift wiring) and cross-reference naba `SPEC-*` as the reference impl.

### E3 design decision (SPEC structure / home / namespace — synthesized from E1+E2)

- **Home:** `docs/specifications/agent-tools.md` in the **naba** repo (operator scope decision).
- **Namespace:** new `AGENT-TOOLS-<AXIS>-NNN` clause IDs (`AXIS` ∈ `SKILLS` | `MCP` | `JSON`).
  Distinct from `SPEC-*` so naba's `check_traceability.py` (which scans `**SPEC-…**`) does **not**
  require parity coverage for portable requirements — their "coverage" is the reference-impl
  mapping (below), not a naba parity case.
- **House style:** RFC-2119 **"shall"** + `*(testable)*` markers (yoshiko-flow `SPEC-TEMPLATE.md`
  style) so the doc cross-references cleanly with yf; a **Guardrails** section mirroring GR-001/003
  (self-manages only its *own* embedded skills; MCP axis is optional; not a general manager/runtime).
- **Structure:** intro + three axis sections (SKILLS / MCP / JSON), each a set of `AGENT-TOOLS-*`
  requirements; a **"Reference implementations" mapping table** (each `AGENT-TOOLS-*` → the naba
  `SPEC-*` clause **and** the yoshiko-flow `REQ-YF-*` clause that realize it); a **"Deltas across
  implementations"** subsection (surface↔harness, skills-receipt vs binary-receipt, MCP present in
  naba / absent in yf, pipe-auto-enable naba-only).
- **Traceability/DRIFT-CHECK:** add a lightweight naba drift **node** for `agent-tools.md` with a
  **`cross-ref`** edge to the `skill-spec` node (agreement, deliberately NOT `field-set-equal`);
  do **not** wire it into `check_traceability.py`'s required set. Keep coupling minimal.
- **yoshiko-flow cross-reference:** a doc-only "See also" pointer in
  `SURFACE_CONVENTION.md` — landed via the reconcile step against the yoshiko-flow repo
  (recon + reference only; no `REQ-*` change).

## Approach

1. **Author `docs/specifications/agent-tools.md`** (naba) — a tool-agnostic, RFC-2119 SPEC of the
   three-axis agent-tools pattern using the `AGENT-TOOLS-<AXIS>-NNN` namespace, built from the ~25
   generalizable clauses (E2), with a Guardrails section compatible with yoshiko-flow GR-001/003.
2. **Reference-implementation mapping** — a table binding every `AGENT-TOOLS-*` requirement to the
   naba `SPEC-*` clause and the yoshiko-flow `REQ-YF-*` clause that realize it, plus a
   "Deltas across implementations" subsection naming the divergences (E1).
3. **Wire naba's local machinery minimally** — index `agent-tools.md` from
   `docs/specifications/README.md`; add a DRIFT-CHECK `cross-ref` node/edge (not `field-set-equal`,
   not a traceability-required clause).
4. **yoshiko-flow cross-reference (recon + reference only)** — add a doc-only "See also" pointer to
   the naba agent-tools SPEC in `SURFACE_CONVENTION.md` (and optionally `SPEC.md` §1/§2 +
   `preflight-contract.md` §0); **no `REQ-*`/behavior change** in the yoshiko-flow repo.
5. **Reconcile #12** — update/close #12 noting the SPEC is authored and cross-referenced; file the
   deferred portable-template stretch as a follow-on bead.

## Epics

### Epic 1: Author the agent-tools SPEC (naba)
The core deliverable. A tool-agnostic, RFC-2119 SPEC of the three-axis pattern.
- Issue 1.1: Author `docs/specifications/agent-tools.md` — intro + Guardrails (GR-001/003-compatible:
  a tool manages **only its own embedded** skills; the MCP axis is **optional**; not a general
  manager/runtime) + three axis sections (`AGENT-TOOLS-SKILLS-*`, `AGENT-TOOLS-MCP-*`,
  `AGENT-TOOLS-JSON-*`), one requirement per generalizable clause from E2 (RFC-2119 "shall",
  `*(testable)*` markers). Enumerate the three flagship contracts. The MCP-axis section states the
  interface is **stdio-based, not an HTTP server** (GR-011-style insurance for the tool-agnostic
  claim — red-team M5).
- Issue 1.2: **Reference-implementation mapping table** — every `AGENT-TOOLS-*` requirement → the
  naba `SPEC-*` clause AND the yoshiko-flow `REQ-YF-*` clause that realize it; plus a **"Deltas
  across implementations"** subsection (surface↔harness; skills-receipt vs binary-receipt; MCP
  present in naba / absent in yf by GR-003; pipe-auto-enable naba-only). **Authoring constraint
  (red-team C3):** the mapping cites **bare** clause IDs (`SPEC-EMBED-003`, `REQ-YF-MARK-001`) — it
  **never** uses the `**SPEC-…** [MARKER]` bold+bracket form, so `check_traceability.py`'s
  first-marker-wins scan never treats `agent-tools.md` (which sorts before `skills.md`) as a
  clause's canonical definition site.
  - depends-on: 1.1

### Epic 2: Wire naba local machinery (minimal coupling)
- depends-on: Epic 1
- Issue 2.1: Index `agent-tools.md` from `docs/specifications/README.md` (TOC row + one-line
  description positioning it as the portable pattern the per-domain specs realize).
  - depends-on: 1.1
- Issue 2.2: Add a DRIFT-CHECK **`cross-ref`** node for `agent-tools.md` with agreement edges to the
  spec nodes each axis maps onto — `skill-spec` (skills.md) for the SKILLS axis, plus **new nodes for
  `mcp.md` and `json-output.md`** for the MCP and JSON axes (red-team C1: without these two the drift
  net covers only 1 of 3 axes). All edges are **`cross-ref`** (agreement), deliberately **not**
  `field-set-equal`. Perform the §0 re-approval the manifest convention requires. Do **not** wire
  `agent-tools.md` into `check_traceability.py`'s required set. Where an axis has no cheap node to
  cross-ref, the SPEC + plan state plainly that its mapping is **agreement-by-review** (a named,
  accepted limitation), so "agreement with `skill-spec`" is never read as covering the whole SPEC.
  Verify `check_traceability.py` stays GREEN (its scan matches `**SPEC-…**`, so `AGENT-TOOLS-*`
  clauses add no required coverage) and the markdown lints clean.
  - depends-on: 1.1

### Epic 3: yoshiko-flow cross-reference (recon + reference only)
Cross-repo, **doc-only**. Lands a "See also" pointer in the yoshiko-flow repo — no `REQ-*`/behavior
change (scope decision + E1: a pure pointer is doc-only even under yf's SPEC-first rule).
- depends-on: Epic 1 (the SPEC must exist + have a stable path/URL to point at)
- Issue 3.1: Add a doc-only "See also: naba agent-tools SPEC" pointer in
  `yoshiko-flow/skills/yf-skill-authoring/reference/SURFACE_CONVENTION.md` (best-fit seam), and
  optionally `yoshiko-flow/SPEC.md` §1/§2 + `docs/yf/preflight-contract.md` §0. **Gated by the
  cross-repo landing gate** (below) — the naba SPEC must be pushed first so the pointer targets a
  real committed URL, and the yoshiko-flow commit/push is a separate operator-authorized action in
  that repo. **Close condition (red-team C2):** this issue is not closed until the yoshiko-flow
  pointer commit is **pushed to its remote** (verified per the Reconcile Gate's cross-repo landing
  verification) — a local-only or unmade commit does not satisfy it.
- **Sequencing (red-team C4):** naba fully **lands + pushes** its changes (Phase 6 push of the
  merged tree, incl. `agent-tools.md`) → **then** Epic 3 runs against the stable pushed URL and
  commits/pushes the pointer in the yoshiko-flow repo → **then** #12 closes. The naba push is
  therefore **not** the final action; do not batch Epic 3 into naba's pre-push tree.

## Gates

### Start Gate (mandatory)
- Type: human
- Approvers: operator

### Capability Gate: Cross-repo landing (yoshiko-flow)
- Type: human
- Approvers: operator
- Condition: the naba `agent-tools.md` SPEC is committed and pushed to naba `origin/main` (so the
  yoshiko-flow pointer targets a real, stable URL), AND the operator authorizes a **doc-only**
  commit in the **separate** `~/workspace/dixson3/yoshiko-flow` repo (its own git/push authority,
  SPEC-first rule respected because the change is a pointer, not a `REQ-*`).
- Test: `git -C ~/workspace/dixson3/yoshiko-flow status` is clean before the edit; naba
  `agent-tools.md` is present at the pushed commit (`git ls-remote`/`gh` can confirm the naba URL
  resolves).
- Blocks: Issue 3.1
- Instructions: land + push the naba SPEC (Phase 6 push), then make the yoshiko-flow pointer edit
  in that repo and commit/push it there on explicit operator authorization; it does not go through
  naba's merge-back.

### Reconcile Gate (upstream #12 include)
- Type: auto (all execution beads closed)
- Approvers: automated (all execution beads closed)
- Condition: comment on and **close** GH #12 — the tool-agnostic agent-tools SPEC is authored
  (all three axes), naba + yf mapped as reference/conforming implementations, and the yoshiko-flow
  cross-reference landed; the deferred portable-template stretch filed as a follow-on bead.
- **Cross-repo landing verification (red-team C2):** before the reconcile step closes #12, verify the
  yoshiko-flow pointer commit is **pushed to its remote** — `git -C ~/workspace/dixson3/yoshiko-flow
  log --oneline origin/main | grep <pointer-commit>` (or `git ls-remote`) — since Epic 3's artifact
  lands outside beads authority and a bead can be closed while the commit is unpushed. Do not close
  #12 asserting "the cross-reference landed" without this evidence.
- Blocks: reconcile step.

## Risks & Mitigations

| Risk | Mitigation |
|:-----|:-----------|
| **Cross-repo edit** (Epic 3 touches yoshiko-flow) escapes naba's merge-back/push flow | Isolated to **one doc-only pointer**; gated behind the **Cross-repo landing gate**; committed/pushed **separately** in the yoshiko-flow repo on explicit operator authorization; naba's Phase-6 merge/push covers only the naba changes. |
| SPEC **overclaims** and trips yoshiko-flow `GR-001` (not a general manager) / `GR-003` (not a runtime) | A **Guardrails** section mirrors GR-001/003; the MCP axis is stated **optional** (yf is a by-design non-adopter); red-team checks the wording. |
| **Naming conflict** surface↔harness silently universalized | The **Deltas** subsection (1.2) names it explicitly; the SPEC uses neutral phrasing ("harness/surface selector") and does not pick a winner. |
| **Namespace collision** with naba `SPEC-*` breaks traceability/drift | New `AGENT-TOOLS-*` namespace (E2); 2.2 verifies `check_traceability.py` stays green and the drift edge is `cross-ref`, not `field-set-equal`. |
| **Over-coupling** the portable SPEC to naba's local machinery | Minimal: one README index row + one `cross-ref` drift edge; explicitly NOT a traceability-required clause set. |
| **Reference-impl mapping rots** as naba/yf specs evolve | The mapping is `cross-ref` (agreement, not byte-equality); cite clause IDs (stable/append-only) not line numbers; the drift edge surfaces divergence for a human. |
| yoshiko-flow **SPEC-first** rule seems to require a `REQ-*` change | E1 confirms a pure "See also" pointer is **doc-only** and needs no `REQ-*`; only a *behavior* change would — and none is in scope. |

## Success Criteria

- `docs/specifications/agent-tools.md` exists in naba: RFC-2119, three axis sections
  (`AGENT-TOOLS-SKILLS-*` / `-MCP-*` / `-JSON-*`) built from the ~25 generalizable clauses, with a
  Guardrails section compatible with yoshiko-flow GR-001/003 and the MCP axis marked optional.
- A **reference-implementation mapping table** binds every `AGENT-TOOLS-*` requirement to its naba
  `SPEC-*` and yoshiko-flow `REQ-YF-*` realizations, and a **"Deltas across implementations"**
  subsection names the divergences (surface↔harness, receipt scope, MCP presence, pipe-auto-enable).
- `agent-tools.md` is indexed from `docs/specifications/README.md`; a DRIFT-CHECK `cross-ref`
  node/edge is added (with §0 re-approval); `check_traceability.py` is GREEN; markdown lints clean.
- The yoshiko-flow repo carries a **doc-only "See also"** pointer to the naba agent-tools SPEC in
  `SURFACE_CONVENTION.md` (landed as a separate operator-authorized commit; no `REQ-*` change).
- **GH #12 is closed**, referencing the authored SPEC + cross-reference; the portable-template
  stretch is filed as a follow-on bead.
