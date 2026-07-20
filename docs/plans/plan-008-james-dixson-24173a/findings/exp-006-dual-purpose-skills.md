# Finding E6: Dual-purpose (CLI + MCP) skills architecture

**Date:** 2026-07-20
**Experiment:** E6
**Confidence:** HIGH (full read of the skill, `src/mcp.rs`, `src/embed.rs`, SPEC §11/§12).

## Recommendation (decisive)

**Single templatized source, rendered at BUILD time into two embedded trees (`cli/` + `mcp/`),
with the template engine as a `[build-dependencies]`.** Prefer **minijinja** (readability) or
**tinytemplate** (lightest); a **no-crate `build.rs` section-filter** is an acceptable zero-new-dep
fallback since the MCP transform is mostly subtractive. **Do NOT** maintain two hand-authored
skill sets; **do NOT** render at install time.

## Why

### Content stratifies into shared core + transport shell
- **Transport-agnostic core (byte-identical, high value):** prompt-engineering order
  (`SKILL.md:114`), per-command narrowing (`SKILL.md:123-128`), anti-patterns (`SKILL.md:133`),
  and the style/variation/type **enum vocabularies** — which already appear a *third* time as MCP
  schema enums in `mcp.rs:771-772`.
- **CLI-invocation shell (differs / meaningless to MCP), ~40-50% of bytes, low unique value:**
  invocation lines + bash `Examples`, flag tables with short-flags, and whole
  **Claude-Code-mechanic sections wrong for MCP hosts** — the `## Preflight` gate
  (`SKILL.md:30-52`), the `## Router` (`$ARGUMENTS`, `${CLAUDE_SKILL_DIR}`, `SKILL.md:54-105`),
  output `-o`/CWD rules, provider rules. The **3 composite subcommands** (storyboard/batch/brand-kit)
  have **no MCP counterpart** (MCP exposes 8 flat tools, `mcp.rs:707-716`, SPEC-MCP-002) → subtract, don't translate.
- **Delta verdict:** shared core is large and identical; the differing part is a moderate-to-large
  but **mostly subtractive** shell. Not "a few tweaks," not two different bodies of knowledge.

### The embed/hash/marker invariant dictates render timing
- `embed.rs` embeds the raw `skills/` tree at compile time (`include_dir!`, `embed.rs:34`) and
  enforces **deployed == embedded**: install writes raw bytes + injects a marker, and
  `deployed_tree_hash` (marker stripped) must equal `embedded_tree_hash` (SPEC-EMBED-002/003;
  test `deployed_marked_tree_hashes_as_embedded` `embed.rs:448-488`). `skills status`/`doctor`
  `up_to_date`/`unmodified` depend on it.
- **Render-at-install BREAKS this** — deployed rendered content ≠ embedded template → every install
  reads "modified/outdated." Rejected.
- **Render-at-MCP-serve is fine** (ephemeral `skill://` reads, `mcp.rs:601-630`, never touch the hash).
- **∴ Only clean design: render-at-BUILD into two embedded trees.** `build.rs` renders the single
  source into `cli/` (embedded; marker/hash machinery unchanged) and `mcp/` (embedded; served raw
  over `skill://`). Each tree has a stable hash; `embed.rs` semantics untouched.
  `skill_resources()`/`read_skill_resource` (`mcp.rs:577,601`) point at the `mcp/` tree — small, localized change.

### Fixes a real defect
SPEC-MCP-014/015 (`mcp.rs:577-630`) currently serves the **raw CLI-flavored** `SKILL.md` over
`skill://naba/SKILL.md` — so an MCP host is told to parse `$ARGUMENTS` and shell out `naba generate`
via `Bash`, when its correct action is to call the `generate_image` tool. The `mcp/` tree fixes this.

### Cost
- **Embed cost negligible either way** (~27KB md; second tree ~+40-56K, immaterial in a multi-MB binary).
- **Build-time engine ⇒ zero runtime/binary cost, zero new runtime dep** — honors minimal-deps exactly;
  the constraint that would favor two-set evaporates. Only compile time is affected (tinytemplate/minijinja light there).
- **Two-set's real penalty is drift:** the param inventory is *already* duplicated between skill flag
  tables and `mcp.rs` golden schemas (`mcp.rs:701-702`) with a `DRIFT-CHECK.md` edge `e-skill-spec`
  (`SKILL.md:183-186`); a second full set makes the shared core a **third** hand-maintained copy.

## Template-crate ranking (if a crate is used)

| Crate | Dep tree | Verdict |
|:------|:---------|:--------|
| tinytemplate | serde only | lightest-adequate; `{{if}}`/`{{for}}` cover cli/mcp gates |
| minijinja | serde + modest defaults | readable Jinja, self-contained |
| handlebars | pest/pest_derive proc-macro | heavier than needed |
| tera | regex, pest, chrono, globwalk, unic-* | overkill, rejected |

As `[build-dependencies]` the crate adds **zero** to the shipped binary. The template needs only
coarse `{{#if cli}}…{{/if}}` / `{{#if mcp}}…{{/if}}` gates around the shell, shared core ungated —
so even a trivial engine or a `build.rs` filter suffices.

## Follow-on to file
Fold the parameter/enum inventory so the template core and `mcp.rs` golden schemas derive from
**one** list — closes the pre-existing skill-md ↔ `mcp.rs` drift (an epic candidate, or at least a
tracked follow-on). **Go-ism spotted for E8:** `mcp.rs:701-702` comment "match the Go-captured
golden verbatim".

## Files
`skills/naba/SKILL.md`, `skills/naba/commands/*.md`; `src/mcp.rs` (skill_resources 577,
read_skill_resource 601, tools 707, enums 771); `src/embed.rs` (hash_tree 118, skill_status 235,
parity test 448); `SPEC.md` §11.1 (703-718), §12 (722-740).
