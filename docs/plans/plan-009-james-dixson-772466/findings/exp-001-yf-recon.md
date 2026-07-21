# Finding E1 — yoshiko-flow reconnaissance

**Experiment:** E1 (yoshiko-flow reconnaissance)
**Confidence:** HIGH (read-only survey of yoshiko-flow @ `5bba4bf`, branch `main`)
**Sources:** `yoshiko-flow/{SPEC.md,GUARDRAILS.md,README.md,AGENTS.md}`,
`docs/yf/preflight-contract.md`, `skills/SPEC-TEMPLATE.md`,
`skills/yf-skill-authoring/{SKILL.md,reference/SURFACE_CONVENTION.md}`, `yf/src/`,
`docs/plans/plan-010-.../plan.md`.

## Pivotal fact: naba is the behavioral ancestor of `yf skills`

plan-010's reference recon (`yoshiko-flow/docs/plans/plan-010-james-dixson-73eebd/plan.md:60-62`)
states naba's *behavior* is the model `yf` was reverse-engineered from (skills
install/upgrade/remove/status, `--scope/--surface/--target/--dry-run`, tree-hash integrity marker
`<!-- yf-skills: v=… tree=… -->`, stale-file prune, `doctor --json`). **So the SPEC describes a
pattern yoshiko-flow already implemented *from naba*** — this is reconciliation with a descendant,
not a stranger.

## Axis 1 — skills self-management: strong AGREE (naba = reference), a few EXTENDs, one live CONFLICT

- **Packaging:** `skills/<name>/` + `SKILL.md` (YAML frontmatter) + optional `scripts/*.py`,
  `protocols/*.md` + `manifest.json`, `spec/`, `agents/`. `REQ-YF-INSTALL-003` parses
  `name, skill-group, depends-on-tool, depends-on-skill, user-invocable`.
- **Verbs:** `REQ-YF-CLI-001` = `skills (install|upgrade|remove|status)` + `self, doctor,
  preflight, migrate, version`. `REQ-YF-CLI-002` = `--scope {user,project}`,
  `--surface {claude,agents}`, `--target`, `--dry-run`.
- **Integrity markers match naba's tree-hash model:** `REQ-YF-MARK-001..004` — per-skill SHA256
  tree hash, `SKILL.md` marker-stripped before hashing, marker `<!-- yf-skills: v=… tree=… -->`,
  `status` = installed/up-to-date/complete/unmodified, upgrade prunes stale files.
- **naba EXTENDS:** a **skills install receipt** (yf has only a *binary* receipt,
  `~/.config/yf/yf-receipt.json`, `REQ-YF-SELF-001`); **per-harness idiomatic layouts** (yf has
  `--surface {claude,agents}` only).
- **Live naming CONFLICT (load-bearing):** yoshiko-flow SPEC/CLI is committed to **"surface"**
  (`REQ-YF-CLI-002`, `REQ-YF-SELF-005`); naba (plan-008) renamed `--surface`→`--harness`. The
  agent-tools SPEC must **name this divergence explicitly**, not silently pick "harness."
- **Guardrails to respect:** `GR-001` "`yf` is not a general package/skill manager … manages
  **only** the skills embedded in this binary"; `GR-003` "`yf` is not a skill *runtime* … the
  **harness** runs them." Axis-1 wording must frame self-management as *"a tool manages its own
  embedded skills"* to stay GR-001/GR-003-compatible.

## Axis 3 — `--json`: strong AGREE (yf has a spec-level contract already)

- `REQ-YF-CLI-003` (verbatim): *"every subcommand shall support `--json` for machine-readable
  output and shall exit non-zero on failure."*
- Dedicated contract doc `docs/yf/preflight-contract.md`: *"the **status field**, not the process
  exit code, is the machine-readable verdict in JSON mode … consumers MUST treat the `status`
  field as authoritative"* — the closest existing analog to naba's envelope; cite as prior art.
- **pipe-auto-enable is ABSENT** in yf (no auto-JSON-when-not-a-TTY) — a naba EXTENSION.
- yf standardized the flag name on `--json` (legacy was `--json-output`).

## Axis 2 — MCP-over-CLI: ABSENT in yoshiko-flow (additive/novel)

`yf` deliberately exposes **no** MCP surface (`GR-003` not-a-runtime; `GR-011` forbids an async
HTTP stack). Every MCP mention in the repo is either a *consumer* (`yf-research` calls
`mcp__exa__*`) or ecosystem prose. No "expose this CLI's verbs over MCP" convention exists — this
axis is genuinely novel to naba; the SPEC **introduces** it (and `yf` is a by-design non-adopter
of it).

## No pre-existing agent-tools doctrine; house style + seam

- Grep for `agent-tools`/`three axes`/`portable pattern`/`mcp-over-cli`/`self-managing` → **zero
  matches**. Nearest doctrine: `skills/yf-skill-authoring/reference/SURFACE_CONVENTION.md` (the
  7-element "Skill Surface Convention"); `SPEC.md` §2 composition model + `skills/SPEC-TEMPLATE.md`.
- **House style** (any new SPEC should mirror): RFC-2119 "shall", `REQ-<KEY>-NNN` ids, `*(testable)*`
  markers, a Guardrails section.
- **Cross-reference seam (priority order):** (1) **best fit** —
  `skills/yf-skill-authoring/reference/SURFACE_CONVENTION.md` "See also" pointer (doctrinal home
  for skill self-management + preflight contract); (2) `SPEC.md` §1/§2 one-line "See also";
  (3) `docs/yf/preflight-contract.md` §0 provenance back-pointer; (4) `GUARDRAILS.md` References
  (optional). Use a **prose "See also" link, NOT a drift-check edge** — the SPEC lives in the naba
  repo, so a hard cross-repo drift edge is unenforceable by yf's local tooling.
- yoshiko-flow is **SPEC-first**: a pure cross-reference pointer is doc-only (no `REQ-*` change);
  only if yf *behavior* were to change would a `REQ-*` edit be needed (out of scope here).

## Implications for plan-009

1. Frame the SPEC as reconciling with a **descendant of naba**: name yf's `yf skills` /
   `REQ-YF-MARK` / `REQ-YF-CLI-003` as an **existing conforming implementation** (naba being the
   other reference impl), and enumerate the deltas (surface↔harness, binary-receipt vs
   skills-receipt, no-MCP, no-pipe-auto-enable).
2. Axis 3 agrees cleanly — cite `REQ-YF-CLI-003` + `preflight-contract.md` (incl. the
   status-authoritative-in-JSON nuance).
3. Axis 1 wording must not trip GR-001/GR-003.
4. Axis 2 is additive — no yf counterpart to contradict.
5. The surface↔harness naming conflict is real; do not universalize "harness."
