# Finding E2 — naba reference-contract inventory (generalizable vs naba-specific)

**Experiment:** E2 (naba reference-contract inventory)
**Confidence:** HIGH (full read of the split spec set + traceability harness + DRIFT-CHECK graph)
**Sources:** `docs/specifications/{skills,mcp,json-output,commands,README,distribution}.md`,
`tests/parity/check_traceability.py`, `tests/parity/traceability_exemptions.yaml`, `DRIFT-CHECK.md`

## Axis 1 — Skills self-management lifecycle

**Portable contract:** embed a versioned+hashed skill tree with an idempotent integrity marker
(EMBED-001/002); expose a four-state status vocabulary **installed / complete / unmodified /
up-to-date** (EMBED-003); resolve install destinations from a **data-driven harness descriptor**
with user/project/override scope (HARNESS-001/003/006/007); record every install in a **receipt**
so unqualified upgrade/remove/preflight are receipt-driven, continue-on-error, path-deduped
(INSTALL-001/002, HARNESS-005); self-heal a missing receipt by scanning known locations
(INSTALL-003); gate all of it behind a **fast multi-axis JSON preflight** with a non-blocking
self-update axis (PREFLIGHT-001/003/004).

**Generalizable clauses:** EMBED-001/002 (partial — marker concept portable, literal prefix +
byte recipe specific), EMBED-003 (yes), HARNESS-001/003/005/006/007 (yes), HARNESS-002/004
(partial), INSTALL-001/002 (yes), INSTALL-003 (partial), PREFLIGHT-001/003/004 (yes),
PREFLIGHT-002 (partial).

**naba-specific residue:** marker prefix `naba-skills:`, exact sha256 byte recipe, the single
skill name `naba`, the concrete 5-row harness table + `.claude`/`.agents`/`.pi`/… paths, the
`--surface`→`--harness` migration, all Rust mechanics (`include_dir!`, `$OUT_DIR`, `build.rs`,
minijinja, `src/harness.rs`). EMBED-004 (Go-port hash migration) is pure history.

## Axis 2 — MCP-over-CLI interface

**Portable contract (three-fold):** (1) every CLI capability mirrored as an MCP tool over stdio
(MCP-001), MCP returning **tool-level error results + per-artifact resource links + its own
output-dir resolution** rather than exit codes (MCP-013); (2) **skills as lazily-loaded MCP
resources** — metadata-only `resources/list` of a `skill://` index + per-file URIs, bodies
fetched on demand via `resources/read` (MCP-014/015); (3) the MCP surface serves an
**MCP-flavored render** of skill content distinct from the CLI-deployed tree (the `mcp/` vs
`cli/` split, ties to EMBED-005).

**Generalizable clauses:** MCP-001 (partial), MCP-013/014/015 (yes). **MCP-002–012 are the bulk
of the MCP spec but the LEAST generalizable** — the specific 8-tool roster + per-tool image-domain
schemas + `list_images` + `file://`/`image/*` blob specifics. A tool-agnostic SPEC keeps only
001/013/014/015 and states "each tool corresponds 1:1 to a CLI verb."

## Axis 3 — `--json` agent output

**Portable contract:** a documented **universal envelope `{status, data?, error?}`** every
subcommand emits and a traceability test enforces (JSON-006); **auto-enable JSON when stdout is
not a TTY** (GLOBAL-003); scalar-when-one/array-when-many with declared always-array exceptions
(JSON-002/003); the flat-vs-`targets[]` **multi-target** convention where single is one element of
many (JSON-007); a declared set of **normalized nondeterministic fields** so output is
deterministic-testable (JSON-005).

**Generalizable clauses:** JSON-002/005/006/007 (yes), GLOBAL-003 (yes), JSON-001/003/004
(partial). **JSON-006 (universal envelope + every subcommand's JSON documented and test-enforced)
is the single most generalizable clause in the whole inventory.**

**naba-specific residue:** exact `Result` field names (JSON-001), grandfathered image-`Result` +
`doctor` envelopes, concrete command names.

## Traceability + DRIFT-CHECK wiring → namespace decision

- **Traceability:** every `**SPEC-…**` `[PINNED]`/`[NEW]` clause MUST map to a parity case, a
  citing pytest module, or a justified exemption; `check_traceability.py` regex-scans all
  `docs/specifications/*.md` and fails on any uncovered required clause (first-marker-wins per ID).
- **DRIFT-CHECK:** spec files are drift nodes tied by edges (e.g. `e-skill-spec` = `field-set-equal`
  between the SKILL.md dispatch table and `skills.md`'s subcommand map).
- **Namespace recommendation:** `SPEC-*` IDs are stable, append-only, load-bearing. **Do NOT reuse
  them** in a tool-agnostic SPEC — collision breaks the "one clause defined once" invariant and
  would force the parity suite to cover portable clauses it can't. Give the agent-tools SPEC its
  **own namespace** (e.g. `AGENT-TOOLS-<AXIS>-NNN` / `ATSPEC-*`) and **reference** naba `SPEC-*`
  IDs as the reference implementation ("naba realizes AGENT-TOOLS-SKILLS-003 via SPEC-EMBED-003").
  If drift-checked, model it as a new node with a **`cross-ref`** edge to naba's `skill-spec`
  node — deliberately not `field-set-equal` (the two are not byte-identical).

## Implications for plan-009

1. Author the SPEC from the **~25 generalizable clauses** (a minority of clauses, the majority of
   the value), structured on the same three axes.
2. Anchor on **three flagship portable contracts**: (a) receipt-driven, harness-descriptor,
   integrity-marked skills self-management + fast preflight; (b) skills-as-lazy-MCP-resources with
   every CLI verb mirrored as a tool; (c) documented, test-enforced universal `--json` envelope
   that auto-enables when piped.
3. **Own ID namespace**, cross-reference naba `SPEC-*` — do not reuse.
4. Mark naba domain schemas (MCP-002–010, JSON-001) + Go-port migration (EMBED-004) as
   out-of-scope / illustrative-only — the cleanest "implementation detail" boundary.
