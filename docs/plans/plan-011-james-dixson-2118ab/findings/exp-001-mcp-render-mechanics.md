# Finding E1 — MCP skill-render mechanics, invariants, and blast radius

**Experiment:** E1 (implementation review of the mcp/ render + resource surface; SPEC contract;
test/golden/drift-check blast radius)
**Confidence:** HIGH (end-to-end read of build.rs, src/embed.rs, src/mcp.rs, the specifications,
tests/parity, and DRIFT-CHECK.md)

## The gap (why this plan exists)

The MCP resource surface is machinery-complete but **content-empty of MCP-specific guidance**:

- `build.rs` renders the single `skills/` source into two `$OUT_DIR` trees — `cli/` and `mcp/` —
  but **only `SKILL.md` is templated** (minijinja `{% if cli %}` / `{% if mcp %}`). **Every other
  file, including `commands/*.md`, is copied verbatim into both trees** (`build.rs`
  `render_dir`).
- `skills/naba/SKILL.md` has **zero `{% if mcp %}` blocks** — only two `{% if cli %}`. So the mcp
  render is "the CLI skill minus its cli-only sections." It still reads *"Create or transform
  images with the naba **CLI**, invoked as `/naba <subcommand>`"*, *"TRIGGER when: /naba invoked"*,
  references `--style`/`--size` **CLI flags**, and carries CLI frontmatter (`depends-on-tool:
  [naba]`, `allowed-tools: [Bash, Read, Agent]`).
- **No `commands/*.md` mentions the MCP tools** (`generate_image`, `edit_image`, params,
  `NABA_OUTPUT_DIR`).

Net: an MCP client (Claude Desktop — no shell, no `/naba`, no `naba` CLI) is served guidance about
slash commands and CLI flags it cannot use. This matches the **SPEC as written** (SPEC-EMBED-005
defines the mcp render as *"subtractive, MCP-flavored"*; SPEC-MCP-014 repeats "subtractive"), so
the gap is **at the SPEC level too** — the plan must amend the SPEC, not just the code.

## Invariants (what must not break)

- **CLI-tree byte-identity / tree hash (SPEC-EMBED-002/005).** `embedded_tree_hash` →
  `read_skill_file` → `SKILLS` = `$OUT_DIR/cli`. The **hash covers only the cli/ (deployed)
  tree.** The mcp/ tree is **not** hashed and is **not** deployed by `skills install`, so
  authoring arbitrary MCP content into the mcp render **cannot** change install-integrity status
  or force an `upgrade`. This is the key de-risker: MCP content is free to diverge.
- The cli/ render must stay **byte-identical to `skills/naba/` source** — so any new MCP content
  must be authored so it renders to **nothing** in the cli tree (a `{% if mcp %}` block renders
  empty under `cli`; an mcp-only source file must be excluded from the cli render).

## Delivery decisions (operator-confirmed, 2026-07-21)

- **Hybrid delivery.** Thin one-line pointer in each MCP tool `description` (so the assistant
  knows guidance exists) + detailed **lazy** `skill://` resources fetched on demand. Keeps
  always-loaded context minimal.
- **One MCP usage guide + per-tool only where needed.** A single MCP-framed `skill://naba` entry
  (tool catalog, prompt-engineering, `NABA_OUTPUT_DIR`/output-dir resolution, quality semantics,
  the `file://` result links) plus a per-tool file only for tools with real specifics.
- **Replace** the verbatim CLI `commands/*.md` in the **mcp tree** with MCP-authored content. The
  cli tree keeps them unchanged.

## build.rs authoring-layout options (design fork for PLAN)

The render must produce a **different file set** for mcp vs cli while keeping cli byte-identical.
Candidate approaches:

1. **`{% if mcp %}`/`{% if cli %}` in SKILL.md + an mcp-only source subtree.** Author the MCP guide
   as `{% if mcp %}` blocks in the templated `SKILL.md`, and put per-tool MCP files under a new
   source subtree (e.g. `skills/naba/mcp/…`) that `build.rs` maps into the **mcp** render only and
   **excludes from the cli** render; simultaneously **exclude `commands/*.md` from the mcp** render.
   Cli stays byte-identical (the mcp/ subtree and the `{% if mcp %}` blocks vanish under `cli`).
2. **Template every file** with `{% if %}` gates (one physical file carries both variants). Simpler
   directory layout, messier files; large divergent bodies are awkward.

Recommend approach 1 (clean separation, each render self-contained). The planner picks; red-team
should probe the cli byte-identity proof under whichever layout.

## Blast radius (everything that must move together)

- **SPEC:** `docs/specifications/skills.md` SPEC-EMBED-005 (subtractive → additive/authored MCP
  render + the new source layout) and `docs/specifications/mcp.md` SPEC-MCP-014/015 (drop
  "subtractive"; the resource set is the MCP-authored files; add the tool-description pointer
  clause).
- **Impl:** `build.rs` (render routing), `skills/naba/SKILL.md` (`{% if mcp %}` guide) + new
  `skills/naba/mcp/…` content, `src/mcp.rs` (thin pointer appended to each tool `description`; the
  resource enumeration already lists whatever files are in the mcp tree via `embed::skill_files_mcp`
  — confirm it needs no change beyond the new file set).
- **Tests/goldens (tests/parity):** `test_mcp.py` `EXPECTED` per-tool `description` strings change
  (pointer appended) → regen `golden/mcp/tools.json` via `--update-golden`/`UPDATE_GOLDEN=1`; the
  **resources/list** enumeration test and any `skill://` read test change (new mcp file set) — the
  independent oracle must be **hand-updated** to the intended new surface, not blindly regenerated.
- **Docs:** `web/content/pages/mcp.md` — rewrite the "Lazy-loading skills as resources" section to
  describe MCP-authored guidance (not "the same CLI skill via resources"), and add the *why the
  user cares* framing.
- **DRIFT-CHECK.md:** approved manifest with `mcp-source`/`mcp-spec`/`skill-spec`/`web-mcp` nodes
  and the `e-web-mcp-tools` edge. New MCP content files may need a node/glob; the manifest text
  (currently says the mcp render is "subtractive") must be re-approved to the authored-render model.

## Landmines

- The parity **resource enumeration** test encodes the current `commands/*.md`-in-mcp set; naive
  `--update-golden` would bless whatever we emit — the oracle must be reviewed as *intended*, not
  auto-captured (it is the guard).
- `quality` tool-description normalization (`<QUALITY_DESC>`) already exists in the golden harness;
  the new per-tool pointer text must not collide with that normalization.
- Keep the **tool `name`s and `inputSchema`** (the SPEC-pinned surface, incl. the plan-010 enum
  single-source) unchanged — only the `description` gains a pointer.
