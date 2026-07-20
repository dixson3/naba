# Upstream Issue #12

- **URL:** https://github.com/dixson3/naba/issues/12
- **State:** OPEN
- **Disposition (this plan):** partial
- **Repo:** dixson3/naba

## Title

plan-008 seed: portable agent-tools SPEC (skills self-mgmt + MCP-over-CLI + --json agent output),
reconcile with ~/workspace/dixson3/yoshiko-flow

## Body (verbatim)

Split from plan-007 (red-team pass-1 #4). Author a tool-agnostic 'agent-tools' SPEC capturing
naba's portable pattern: (1) skills self-management lifecycle (embedded tree + skills
install/upgrade/status/remove/preflight, integrity marker, claude default + generic .agents
surfaces, user+project scopes); (2) MCP interface (mcp exposes all CLI interactions as tools +
lazily-loaded skills-as-resources); (3) --json agent-friendly output (documented envelopes +
pipe-auto-enable). naba is the reference implementation. Start with a yoshiko-flow reconnaissance
finding (yf kernel, yf-* skills, yf-skill-authoring, SPEC.md) then author the SPEC; decide its home
(naba docs/specifications + whether to land/reference in yoshiko-flow); stretch: a portable
skill/scaffolding template. Seed a new /yf-plan for this.

## How this plan (plan-008) relates

`partial`. plan-008 advances two of #12's three axes:

- **(1) skills self-management** → the harness-layout SPEC + per-harness idiomatic install
  (`--harness`, receipt-driven multi-harness upgrade, migration). This *supersedes* #12's narrower
  "claude default + generic .agents surfaces" framing with the full per-harness model.
- **(2) MCP interface** → the dual-purpose skills work (build-time `cli/`+`mcp/` render; MCP serves
  the `mcp/` tree), fixing the SPEC-MCP-014/015 defect where MCP served CLI-flavored guidance.

**Left open on #12 (out of scope here):**

- **(3) `--json` agent-friendly output** — documented envelopes + pipe-auto-enable. Not touched by
  plan-008.
- **yoshiko-flow reconciliation** — the yf-kernel/yf-* reconnaissance and deciding whether to
  land/reference the SPEC in yoshiko-flow.
- **Stretch:** the portable skill/scaffolding template.

At land-the-plane, plan-008 comments on #12 recording this partial progress (Reconcile Gate) and
keeps the descoped axes + the skill-md↔`mcp.rs` param-drift follow-on bead (Issue 3.5) visible on
#12 so they are not lost.
