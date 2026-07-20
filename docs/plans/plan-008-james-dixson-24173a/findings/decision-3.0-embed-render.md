# Decision 3.0 — embed-root restructure for the two-tree render

**Issue:** plan-008 Epic 3, Issue 3.0 (decision, do-first). Pins the embed-root
restructure before any render code, resolving red-team concern A (embed-root restructure
breaks `skill_names`/hash pin).

## (a) Render target — **`$OUT_DIR`** (recommended option chosen)

`build.rs` renders the single `skills/naba` source into two trees under Cargo's
`$OUT_DIR`: `$OUT_DIR/cli/naba` and `$OUT_DIR/mcp/naba`. The CLI embed re-points
`include_dir!` from `$CARGO_MANIFEST_DIR/skills` to `$OUT_DIR/cli`.

- **No committed render, no gitignore churn.** The rendered trees live in the build
  output dir, never in the source tree, so there is nothing to gitignore and no risk of a
  stale committed render drifting from the source.
- `skills/naba` stays the single committed source of truth (templatized in 3.2).

## (b) Skill-root computation — unchanged

`SKILLS = include_dir!("$OUT_DIR/cli")` embeds `cli/` as the root, whose immediate
subdirectory is `naba` (the render writes `cli/naba/…`). So `skill_names()` (immediate
subdirs of the embed root) still returns `["naba"]`, and `skill_files`/`read_skill_file`
keep enumerating the `naba` skill unchanged — no code change to the enumeration logic. The
MCP resource surface reads the parallel `mcp/` tree (3.4).

## (c) Byte-identical `cli/` render — **pursue; documented re-baseline fallback**

The CLI variant must reproduce today's `skills/naba` **byte-for-byte** so the pinned
`embed.rs` `NABA_TREE_HASH` (`d5b2fdfe…368c8`) — the `deployed==embedded` invariant — does
not change. Approach:

- MCP is **subtractive only** (drop the router / `skills preflight` / composite / `Bash`
  shell-out mechanics per SPEC-MCP-014/015). So the template wraps the CLI-only sections in
  `{% if cli %}…{% endif %}` gates; there are **no** `{% if mcp %}` additions.
- Render with minijinja using **`trim_blocks` + `lstrip_blocks`** (and `{%- -%}` where
  needed) so the CLI render strips the gate tag lines without leaving stray blank lines —
  reproducing the original bytes. Files with no gates (`commands/*.md`, `README.md`) pass
  through verbatim.

**Fallback (if byte-identical proves unachievable):** re-baseline `NABA_TREE_HASH` to the
new CLI-render hash and accept a **one-time forced re-upgrade** across every recorded
receipt target (Epic 2). This is now cheap and safe: `naba skills upgrade` (unqualified)
already re-hits every receipt target continue-on-error, so a single post-upgrade run brings
all installs current. Re-baseline procedure:

1. Build; read the emitted `embedded_tree_hash("naba")` (the `cli/` render hash).
2. Replace `NABA_TREE_HASH` in `embed.rs` with that value; note the re-baseline in the
   commit message and SPEC-EMBED (5.5).
3. Existing installs report `outdated` until `naba skills upgrade` runs (surfaced by the
   `skills preflight` skills axis) — the one-time forced re-upgrade.

## Consequence for 3.0b (DRIFT-CHECK)

Under the `$OUT_DIR` choice the source `skills/naba` **does not move**, so the DRIFT-CHECK
`skill-md`/`commands` node globs need **no re-glob** — only the `e-installer-skillset`
contract text update + §0 re-approval (the unconditional part of 3.0b).
