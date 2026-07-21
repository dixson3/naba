# Upstream #12: plan-008 seed: portable agent-tools SPEC (skills self-mgmt + MCP-over-CLI + --json agent output), reconcile with ~/workspace/dixson3/yoshiko-flow

- **Number:** 12
- **Title:** plan-008 seed: portable agent-tools SPEC (skills self-mgmt + MCP-over-CLI + --json agent output), reconcile with ~/workspace/dixson3/yoshiko-flow
- **URL:** 
- **State:** OPEN
- **Labels:** type::task, priority::medium

## Body

Split from plan-007 (red-team pass-1 #4). Author a tool-agnostic 'agent-tools' SPEC capturing naba's portable pattern: (1) skills self-management lifecycle (embedded tree + skills install/upgrade/status/remove/preflight, integrity marker, claude default + generic .agents surfaces, user+project scopes); (2) MCP interface (mcp exposes all CLI interactions as tools + lazily-loaded skills-as-resources); (3) --json agent-friendly output (documented envelopes + pipe-auto-enable). naba is the reference implementation. Start with a yoshiko-flow reconnaissance finding (yf kernel, yf-* skills, yf-skill-authoring, SPEC.md) then author the SPEC; decide its home (naba docs/specifications + whether to land/reference in yoshiko-flow); stretch: a portable skill/scaffolding template. Seed a new /yf-plan for this.
