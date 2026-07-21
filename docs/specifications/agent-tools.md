# Agent-Tools Specification — a tool-agnostic contract for making a CLI first-class to AI agents

**Status:** portable pattern specification. Unlike the other files in `docs/specifications/`
(which pin naba's own implementation via `SPEC-*` clauses and a parity suite), this document
is **tool-agnostic**: it states the pattern any command-line tool may adopt to become
first-class to AI agents, and names naba and the yoshiko-flow `yf` kernel as **reference
implementations**.

**Clause IDs:** `AGENT-TOOLS-<AXIS>-NNN`, where `<AXIS>` ∈ `SKILLS | MCP | JSON`. This
namespace is **deliberately distinct** from naba's `SPEC-*` namespace so that naba's
traceability harness (`tests/parity/check_traceability.py`, which scans for `**SPEC-…**`
clause markers) never treats an `AGENT-TOOLS-*` requirement as a naba clause requiring parity
coverage. A conforming implementation demonstrates coverage through the reference-implementation
mapping (see [Reference implementations across tools](#reference-implementations-across-tools)),
not through a naba parity case.

**Conformance language:** RFC 2119. "shall" denotes a mandatory requirement; "should" a
recommendation; "may" an option. Each requirement that a conformance suite can mechanically
check is marked `*(testable)*`.

## Motivation

A CLI becomes first-class to AI agents along three orthogonal axes:

1. A **skills self-management lifecycle** — the tool embeds its own agent-facing skill
   instructions, installs them into whatever harness the agent runs in, and keeps them
   integrity-checked and up to date, with no external package manager.
2. An **MCP-over-CLI interface** — every CLI capability is mirrored as an MCP tool over stdio,
   and the embedded skills are served as lazily-loaded MCP resources, so an MCP client can both
   *invoke* the tool and *discover its instructions* through one server.
3. **`--json` agent-friendly output** — a universal, documented, test-enforced output envelope
   that auto-enables when the tool's stdout is piped, so an agent parsing the tool's output
   never has to scrape human-formatted text.

naba grew this pattern across its plans 004–008; the yoshiko-flow `yf` kernel independently
implements the skills and `--json` axes (it was reverse-engineered from naba's behavior). This
SPEC abstracts the ~two dozen **generalizable** clauses out of naba's implementation-specific
detail so another tool can adopt the contract, and maps each requirement to its naba and
yoshiko-flow realizations.

### The three flagship portable contracts

The value of this pattern concentrates in three contracts a conforming tool should treat as the
headline deliverables:

- **(a)** a receipt-driven, harness-descriptor-based, integrity-marked **skills self-management**
  lifecycle fronted by a fast preflight gate (the SKILLS axis);
- **(b)** **skills-as-lazily-loaded-MCP-resources** with every CLI verb mirrored as an MCP tool
  (the MCP axis);
- **(c)** a documented, **test-enforced universal `--json` envelope** that auto-enables when the
  tool's output is piped (the JSON axis).

## Guardrails

These guardrails bound the pattern so it stays compatible with the yoshiko-flow `yf` kernel's
own guardrails (`GR-001`, `GR-003`, `GR-011`) and does not overclaim.

- **AGENT-TOOLS-GR-001 — Self-management is scoped to the tool's *own* embedded skills.** A
  conforming tool manages **only** the skill tree embedded in its own binary. It is **not** a
  general package manager, skill manager, or registry for arbitrary third-party skills.
- **AGENT-TOOLS-GR-002 — The tool is not a skill *runtime*.** A conforming tool installs and
  serves its skills; the **agent harness** (Claude Code, Codex, etc.) runs them. The tool does
  not execute skill instructions itself.
- **AGENT-TOOLS-GR-003 — The MCP axis is optional.** A tool may conform on the SKILLS and JSON
  axes alone. The MCP-over-CLI interface is an **optional** axis; a by-design non-adopter (such
  as `yf`, which forbids an async HTTP stack) remains a conforming implementation of the other
  two axes.
- **AGENT-TOOLS-GR-004 — The MCP interface is stdio-based, not an HTTP server.** Where the MCP
  axis is adopted, the interface **shall** be an MCP server over stdio. A conforming tool does
  **not** stand up a long-running HTTP service, so the pattern imposes no async-network runtime
  on adopters.

## Axis 1 — Skills self-management lifecycle (`AGENT-TOOLS-SKILLS-*`)

A conforming tool ships its agent-facing skill instructions inside its own binary and manages
their installation, integrity, and freshness itself.

- **AGENT-TOOLS-SKILLS-001 — Embedded, integrity-marked skill tree.** A conforming tool **shall**
  embed its skill tree in the binary and, on install, mark each skill's manifest with an
  idempotent integrity marker of the form `<!-- <tool>-skills: v=<version> tree=<hash> -->`
  injected after the manifest's frontmatter (else prepended). Re-injection **shall** be
  idempotent. *(testable)*
- **AGENT-TOOLS-SKILLS-002 — Deterministic, marker-invariant tree hash.** The integrity hash
  **shall** be computed deterministically over the skill tree's files (a stable per-file sort by
  skill-relative path, hashing both relative path and file bytes), with the manifest's marker
  line stripped before hashing so the embedded (marker-free) and installed (marked) trees hash
  identically. *(testable)*
- **AGENT-TOOLS-SKILLS-003 — Four-state status vocabulary.** `status`/`doctor`-style inspection
  **shall** report a skill against at least four orthogonal states: **installed** (manifest
  present), **complete** (every embedded file present on disk), **unmodified** (recomputed
  on-disk tree hash equals the embedded tree hash), and **up-to-date** (the installed marker's
  hash equals the current embedded tree hash). *(testable)*
- **AGENT-TOOLS-SKILLS-004 — Data-driven harness descriptor.** Install destinations **shall** be
  resolved from a **data-driven harness descriptor** — a static row per supported agent harness
  carrying at least `{ id, user_subpath, project_subpath }` — rather than from hard-coded
  per-harness branches. All harnesses share the same anchors (a user anchor such as `$HOME` and
  a project anchor such as the git root); only the subpath differs per harness and scope.
  *(testable)*
- **AGENT-TOOLS-SKILLS-005 — Scope resolution with explicit-target override.** The installer
  **shall** support at least a `user` scope (resolved against the user anchor) and a `project`
  scope (resolved against the git root if inside a repository, else the working directory), with
  an explicit target-directory override that wins over both and bypasses subpath resolution.
  *(testable)*
- **AGENT-TOOLS-SKILLS-006 — Resolved-path dedupe.** When several requested harnesses resolve to
  the **same absolute path**, the installer **shall** deploy and record that directory exactly
  once (dedupe by resolved absolute path; the first-requested harness wins the recorded entry),
  preventing a double write or double record. *(testable)*
- **AGENT-TOOLS-SKILLS-007 — Extensibility by data, not code.** Adding support for a new harness
  **should** be a single new descriptor row, with no structural code change to the install/upgrade
  machinery. *(testable)*
- **AGENT-TOOLS-SKILLS-008 — Uniform install unit.** The install unit (the skill manifest plus
  its companion files) and the required frontmatter keys **shall** be identical across all
  harnesses; only the destination path data differs. Any per-harness `name_transform` constrains
  the on-disk skill **name**, not the manifest body. *(testable)*
- **AGENT-TOOLS-SKILLS-009 — Install receipt / target registry.** Every install **shall** be
  recorded in a persistent **receipt** (a target registry keyed by `(harness, scope,
  resolved-path)`) so that later upgrade/remove/preflight operations need not re-supply the
  original harness/scope/target. Install upserts a target idempotently; remove drops the matching
  entry. *(testable)*
- **AGENT-TOOLS-SKILLS-010 — Receipt-driven, continue-on-error, path-deduped upgrade.** An
  **unqualified** upgrade (no harness/scope/target given) **shall** enumerate every recorded
  target and refresh each in turn, **continuing on error** (a failing target is reported but does
  not abort the rest) and **deduping by resolved absolute path**. *(testable)*
- **AGENT-TOOLS-SKILLS-011 — Receipt self-heal / legacy migration.** When the receipt is absent
  or empty, the tool **shall** synthesize it from an idempotent disk scan of the known install
  locations (any location already holding an installed skill becomes a recorded target) and
  persist the result, so an already-migrated environment is a no-op on re-run. *(testable)*
- **AGENT-TOOLS-SKILLS-012 — Fast preflight gate.** A conforming tool **shall** provide a fast,
  non-networked **preflight** command emitting a machine-readable multi-axis envelope (at least a
  **skills** axis and a **binary/self-update** axis) that the embedded skill can invoke at trigger
  time and branch on. *(testable)*
- **AGENT-TOOLS-SKILLS-013 — Preflight skills axis.** The preflight's **skills** axis **shall**
  assert that every embedded skill is installed, complete, unmodified, and up-to-date against the
  resolved destination, and the first failing state **shall** drive the remediation detail.
  *(testable)*
- **AGENT-TOOLS-SKILLS-014 — Non-blocking self-update axis.** The preflight's **binary /
  self-update** axis **shall** be **non-blocking**: an absent or stale update-check cache yields
  an `unknown` (or equivalent) tri-state that never fails the overall preflight, so a fresh
  install with no cache still passes. *(testable)*

## Axis 2 — MCP-over-CLI interface (`AGENT-TOOLS-MCP-*`)

This axis is **optional** (`AGENT-TOOLS-GR-003`). Where adopted, the tool exposes its CLI surface
and its embedded skills through a single MCP server over stdio (`AGENT-TOOLS-GR-004`).

- **AGENT-TOOLS-MCP-001 — Every CLI verb mirrored as an MCP tool over stdio.** A conforming MCP
  interface **shall** run as an MCP server over **stdio** (not HTTP) and register each CLI
  capability as an MCP tool corresponding 1:1 to a CLI verb, alongside a resource capability.
  *(testable)*
- **AGENT-TOOLS-MCP-002 — Tool-level results, resource links, own output resolution.** MCP tool
  invocations **shall** report failures as **tool-level error results** (not process exit codes),
  **shall** resolve their own output-directory independent of the CLI's, and **shall** return
  artifact **resource links** (e.g. `file://` URIs) for produced files, one entry per artifact.
  *(testable)*
- **AGENT-TOOLS-MCP-003 — Skills as lazily-loaded MCP resources (metadata-only listing).** The
  MCP server **shall** enumerate the embedded skill tree through `resources/list` as concrete
  resources — a compact per-skill index resource (`skill://<name>`) followed by one resource per
  file (`skill://<name>/<rel>`) — carrying **URIs and metadata only, never file bodies**, so a
  client discovers skills cheaply and fetches instruction content on demand. *(testable)*
- **AGENT-TOOLS-MCP-004 — Lazy `skill://` read.** `resources/read` **shall** resolve the
  `skill://` scheme: `skill://<name>/<rel>` returns the embedded file's content with a MIME type
  by extension, and `skill://<name>` returns a generated index of the skill's file URIs; an
  unknown skill or file returns a resource-not-found error. *(testable)*

> **Agreement-by-review limitation.** The yoshiko-flow `yf` kernel does **not** implement this
> axis (by `GR-003`/`GR-011`), so an implementation conforms on the MCP axis only if it actually
> serves the interface above. Where no cheap cross-reference node exists for an axis, that axis's
> agreement is verified **by review**, not by an automated drift edge — a named, accepted
> limitation of the reference-implementation mapping.

## Axis 3 — `--json` agent-friendly output (`AGENT-TOOLS-JSON-*`)

A conforming tool emits structured output an agent can parse directly, and switches to it
automatically when its output is not a terminal.

- **AGENT-TOOLS-JSON-001 — Universal, test-enforced output envelope.** Every subcommand **shall**
  emit a **documented** JSON structure under `--json`. Discrete-result commands **shall** use a
  **common envelope** `{ "status": <string>, "data": <payload>?, "error": <string>? }`; a
  conformance/traceability test **shall** enumerate every subcommand and assert each emits its
  documented envelope, so "universal" is mechanically enforced rather than aspirational.
  Pre-existing documented JSON shapes may be **grandfathered** rather than rewritten, provided
  each is documented and tested. *(testable)*
- **AGENT-TOOLS-JSON-002 — Pipe auto-enable.** At startup the tool **shall** auto-detect whether
  stdout is a terminal; when stdout is **not** a character device, it **shall** force `--json`
  on, so a piped or captured invocation emits machine-readable output without an explicit flag.
  *(testable)*
- **AGENT-TOOLS-JSON-003 — Scalar-when-one, array-when-many, with declared exceptions.** Commands
  that may produce one or many results **shall** emit a single object for one result and a JSON
  array for more than one, **except** for a **declared** set of always-array commands that emit an
  array even for a single result. The always-array exceptions **shall** be documented. *(testable)*
- **AGENT-TOOLS-JSON-004 — Multi-target output convention.** A command that may act on one or
  several targets **shall** use a consistent flat-vs-array convention: a single resolved target
  emits the flat per-target payload; multiple resolved targets emit a `targets[]` array of the
  same per-target objects, in a deterministic order after dedupe. The single-target shape **shall**
  be exactly one element's worth of the multi-target shape, so a consumer can branch on the
  presence of the array key. *(testable)*
- **AGENT-TOOLS-JSON-005 — Declared normalized nondeterministic fields.** The tool **shall**
  declare the set of nondeterministic output fields (elapsed times, timestamps, version/commit/
  build strings, auto-generated names/paths) that a conformance suite normalizes before
  comparison, so the documented JSON is deterministically testable. *(testable)*

## Reference implementations across tools

Two tools realize this pattern: **naba** (the reference implementation of all three axes) and the
yoshiko-flow **`yf`** kernel (a conforming implementation of the SKILLS and JSON axes;
`yf` is a by-design non-adopter of the MCP axis per `AGENT-TOOLS-GR-003`). `yf`'s skills machinery
was reverse-engineered from naba's behavior, so this is reconciliation with a descendant.

The table below binds every requirement to the naba clause and the yoshiko-flow clause (or
contract doc) that realize it. Clause IDs are cited as **bare identifiers** — for example
`SPEC-EMBED-003`, `REQ-YF-MARK-001` — and never in the `**SPEC-…** [MARKER]` bold-and-bracket form
that naba's `check_traceability.py` treats as a clause's canonical definition site; this keeps
`agent-tools.md` (which sorts before `skills.md`) from ever being mistaken for the defining file
of a `SPEC-*` clause.

| Requirement | naba (`SPEC-*`) | yoshiko-flow `yf` (`REQ-YF-*` / contract doc) |
|:--|:--|:--|
| AGENT-TOOLS-SKILLS-001 embedded integrity marker | SPEC-EMBED-001 | REQ-YF-MARK-001, REQ-YF-MARK-003 |
| AGENT-TOOLS-SKILLS-002 deterministic marker-invariant hash | SPEC-EMBED-002 | REQ-YF-MARK-002 |
| AGENT-TOOLS-SKILLS-003 four-state status vocabulary | SPEC-EMBED-003 | REQ-YF-MARK-004 |
| AGENT-TOOLS-SKILLS-004 data-driven harness descriptor | SPEC-HARNESS-001, SPEC-HARNESS-002 | REQ-YF-CLI-002, REQ-YF-INSTALL-003 (surface, not a full descriptor — see Deltas) |
| AGENT-TOOLS-SKILLS-005 scope resolution + target override | SPEC-HARNESS-003 | REQ-YF-CLI-002 (`--scope`/`--surface`/`--target`) |
| AGENT-TOOLS-SKILLS-006 resolved-path dedupe | SPEC-HARNESS-005 | *(naba extension — no `yf` counterpart)* |
| AGENT-TOOLS-SKILLS-007 extensibility by data | SPEC-HARNESS-006 | *(partial — `yf` surface set is fixed at two)* |
| AGENT-TOOLS-SKILLS-008 uniform install unit | SPEC-HARNESS-007 | REQ-YF-INSTALL-003 (uniform frontmatter parse) |
| AGENT-TOOLS-SKILLS-009 install receipt / target registry | SPEC-INSTALL-001 | *(naba extension — `yf` has only a binary receipt, REQ-YF-SELF-001; see Deltas)* |
| AGENT-TOOLS-SKILLS-010 receipt-driven, continue-on-error upgrade | SPEC-INSTALL-002 | *(naba extension — `yf` upgrade is per-invocation)* |
| AGENT-TOOLS-SKILLS-011 receipt self-heal / legacy migration | SPEC-INSTALL-003 | *(naba extension)* |
| AGENT-TOOLS-SKILLS-012 fast preflight gate | SPEC-PREFLIGHT-001 | REQ-YF-CLI-001 (`preflight` verb), `docs/yf/preflight-contract.md` |
| AGENT-TOOLS-SKILLS-013 preflight skills axis | SPEC-PREFLIGHT-003 | `docs/yf/preflight-contract.md` (skills axis) |
| AGENT-TOOLS-SKILLS-014 non-blocking self-update axis | SPEC-PREFLIGHT-004 | REQ-YF-SELF-001, `docs/yf/preflight-contract.md` |
| AGENT-TOOLS-MCP-001 CLI verbs as MCP tools over stdio | SPEC-MCP-001 | *(absent by `GR-003`/`GR-011` — see Deltas)* |
| AGENT-TOOLS-MCP-002 tool-level results + resource links | SPEC-MCP-013 | *(absent)* |
| AGENT-TOOLS-MCP-003 skills as lazily-loaded MCP resources | SPEC-MCP-014 | *(absent)* |
| AGENT-TOOLS-MCP-004 lazy `skill://` read | SPEC-MCP-015 | *(absent)* |
| AGENT-TOOLS-JSON-001 universal, test-enforced envelope | SPEC-JSON-006 | REQ-YF-CLI-003, `docs/yf/preflight-contract.md` (status field authoritative) |
| AGENT-TOOLS-JSON-002 pipe auto-enable | SPEC-GLOBAL-003 | *(naba extension — absent in `yf`; see Deltas)* |
| AGENT-TOOLS-JSON-003 scalar-when-one / array-when-many | SPEC-JSON-002, SPEC-JSON-003 | *(n/a — `yf` has no image-style multi-result domain)* |
| AGENT-TOOLS-JSON-004 multi-target `targets[]` convention | SPEC-JSON-007 | *(naba extension)* |
| AGENT-TOOLS-JSON-005 declared normalized nondeterministic fields | SPEC-JSON-005 | *(agreement-by-review — normalization is convention, not a single `yf` clause)* |

### Deltas across implementations

Where naba and `yf` diverge, the SPEC uses neutral phrasing and does not pick a winner; the
divergences are:

- **Surface ↔ harness naming.** `yf` names the harness selector `--surface` (REQ-YF-CLI-002); naba
  renamed it `--harness`, keeping `--surface` as a deprecated hidden alias (SPEC-HARNESS-004). This
  SPEC uses the neutral term **"harness/surface selector"** and does not universalize either name.
- **Descriptor breadth.** naba's harness descriptor is a five-row data table over distinct harness
  layouts (SPEC-HARNESS-002); `yf` supports two surfaces (`claude`, `agents`, REQ-YF-CLI-002).
  Both are data-driven; naba's is broader.
- **Skills receipt vs binary receipt.** naba records a **skills-install** receipt and drives
  unqualified upgrade/preflight from it (SPEC-INSTALL-001/002); `yf` records only a **binary**
  self-install receipt (REQ-YF-SELF-001) and has no skills-install target registry.
- **MCP present in naba, absent in `yf`.** naba serves the full MCP-over-CLI interface
  (SPEC-MCP-001/013/014/015); `yf` is a **by-design non-adopter** (`GR-003` not-a-runtime,
  `GR-011` no async HTTP stack) and conforms on the SKILLS and JSON axes only
  (`AGENT-TOOLS-GR-003`).
- **Pipe auto-enable naba-only.** naba forces `--json` when stdout is not a TTY (SPEC-GLOBAL-003);
  `yf` has no pipe-auto-enable — a consumer must pass `--json` explicitly (REQ-YF-CLI-003).
- **Preflight auth axis.** naba's preflight adds an **auth** axis that checks the effective
  provider's API key (SPEC-PREFLIGHT-002); `yf`'s preflight deliberately validates no API keys.
  This is a naba-specific axis, not a portable requirement, and is omitted from the SKILLS axis
  above.
