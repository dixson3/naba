# naba — Skills Specification

Clause IDs (`SPEC-<AREA>-NNN`) are stable and are never renumbered; append only.

## Skills subcommand → CLI-verb map

naba ships a single Claude Code skill, `skills/naba`, invoked as `/naba <subcommand> [args]`.
The skill is a **packaging layer over the CLI** — it adds no image capability of its own; each
subcommand maps to one or more real clap commands in `src/cli.rs`. Seven subcommands map 1:1 to a
single CLI command (**inline** tier); three are **composite** — they orchestrate multiple existing
verbs via a subagent and have no CLI command of their own. `DRIFT-CHECK.md` edge `e-skill-spec`
keeps this table, `skills/naba/SKILL.md`, and the `commands/*.md` files in agreement.

| Subcommand   | Tier      | CLI verb(s) invoked                              |
| :----------- | :-------- | :---------------------------------------------- |
| `generate`   | inline    | `naba generate`                                 |
| `edit`       | inline    | `naba edit`                                     |
| `restore`    | inline    | `naba restore`                                  |
| `icon`       | inline    | `naba icon`                                      |
| `pattern`    | inline    | `naba pattern`                                   |
| `diagram`    | inline    | `naba diagram`                                   |
| `story`      | inline    | `naba story`                                     |
| `storyboard` | composite | `naba story`, then `naba edit` per frame        |
| `batch`      | composite | sequence of `naba generate`/`icon`/`pattern`/…  |
| `brand-kit`  | composite | `naba icon` + `naba pattern` + `naba generate`  |

`story` is inline despite emitting multiple frames: `naba story` is a **single** CLI invocation
that loops internally. Inline subcommands run the `naba` call directly in the parent context;
composite subcommands spawn a subagent (the `Agent` tool) so the per-image loop output stays out
of the parent context, passing the absolute `${CLAUDE_SKILL_DIR}/commands/<sub>.md` path plus the
shared guidance. When subcommands are added, removed, or retiered, update this table, `SKILL.md`,
the `commands/` directory, and the README subcommand table together.

## §12 Skill-embed (SPEC-EMBED)

- **SPEC-EMBED-001** [PINNED] The binary embeds the `skills/` tree. Marker prefix `<!--
  naba-skills:`; marker format `<!-- naba-skills: v=<version> tree=<hash> -->` injected into
  each `SKILL.md` after its YAML frontmatter (else prepended); injection is idempotent.
- **SPEC-EMBED-002** [PINNED] Tree hash `hashTree`: sha256 over, per file sorted by
  skill-relative slash path, `write(relpath bytes) then write(file bytes)`; **no newline
  normalization**; `SKILL.md`'s marker line is stripped before hashing so embedded
  (marker-free) and deployed (marked) trees hash identically.
- **SPEC-EMBED-003** [PINNED] `status`/`doctor` compare: **UpToDate** = marker's `tree=`
  hash == `EmbeddedTreeHash(name)`; **Complete** = every embedded file present on disk;
  **Unmodified** = recomputed `DeployedTreeHash(destDir)` == `EmbeddedTreeHash(name)`;
  **Installed** = `SKILL.md` present.
- **SPEC-EMBED-004** [DIVERGENCE — Concern 4 / M4] The Rust port may **reproduce** Go's
  tree-hash byte-for-byte (so existing installs keep matching), **or** consciously adopt a
  different hash format and require a one-time post-cutover `naba skills upgrade` (Issue
  5.3). Either is acceptable; the choice is recorded in Issue 4.0. The parity suite pins the
  status **semantics** (up-to-date/complete/unmodified flags behave correctly against a
  freshly-installed tree), not the hash literal.
- **SPEC-EMBED-005** [NEW — plan-008] **Dual-purpose two-tree render.** `skills/naba/SKILL.md`
  is a **minijinja template** gated by `{% if cli %}` / `{% if mcp %}`; `build.rs` renders the
  single `skills/` source into two trees under `$OUT_DIR` — `cli/` (embedded via
  `include_dir!("$OUT_DIR/cli")`, the tree `skills install` deploys) and `mcp/` (the
  **subtractive**, MCP-flavored render served by the `skill://` MCP resource surface,
  SPEC-MCP-014/015). Every non-`SKILL.md` file is copied **verbatim** into both trees. The `cli/`
  render is authored to be **byte-identical to the source**, so the SPEC-EMBED-002 canonical tree
  hash is **preserved** — existing installs keep matching and no forced post-cutover `upgrade` is
  triggered by the render.

---

## §18 Skills preflight (SPEC-PREFLIGHT) [NEW — plan-005, Rust-only]

- **SPEC-PREFLIGHT-001** [PINNED] `naba skills preflight [--json]` is a fast skill-gate emitting an
  envelope `{command:"skills preflight", status, axes:{auth, skills, binary}}` with three axes.
  It shares scope/harness/target resolution with `skills`/`doctor` and provider resolution with
  `doctor` (`resolve_provider`/`provider_api_key`/`provider_key_name`, promoted to `pub(crate)`).
- **SPEC-PREFLIGHT-002** [PINNED] **auth** axis: offline provider **key-present** (no network on
  the hot path) — resolves the effective provider and checks its key
  (`GEMINI_API_KEY`/`OPENROUTER_API_KEY`, env or config). This is naba's deliberate divergence
  from `yf preflight`, which validates no API keys.
- **SPEC-PREFLIGHT-003** [PINNED] **skills** axis: every embedded skill is installed + up-to-date
  + complete + unmodified (`embed::skill_status` against the resolved dest); the first failing
  flag drives the `detail` remediation.
- **SPEC-PREFLIGHT-004** [PINNED] **binary** axis is a **tri-state**
  (`up_to_date | update_available | unknown`) read from the update-check cache. An **absent or
  stale cache yields `unknown`, which is NON-BLOCKING** — a fresh install has no cache yet, so the
  overall status stays `ok`. Overall `status` is `auth_missing` (auth fails), else
  `skills_outdated` (skills fails), else `ok`; the binary axis never blocks. Exit code is non-zero
  on any non-`ok` status. `skills/naba/SKILL.md` invokes the gate at trigger time and branches on
  the status.

---

## §19 Harness layout (SPEC-HARNESS) [NEW — plan-008]

Defines each supported agent harness's idiomatic skills-install layout. The descriptor table in
**SPEC-HARNESS-002** is the pinned source of truth; the shipped Rust descriptor
(`src/harness.rs`) and this table are asserted equal by a parity test (Issue 4.2), so the data
here is byte-authoritative.

- **SPEC-HARNESS-001** [NEW] **Descriptor model.** A harness is a static data row
  `{ id, user_subpath, project_subpath, manifest_file = "SKILL.md", frontmatter_required,
  name_transform }`. The **user** anchor is `$HOME`; the **project** anchor is the git root (else
  cwd). `resolve_dest(scope, harness, target)` joins the anchor with the scope-appropriate
  subpath (`user_subpath` for `user` scope, `project_subpath` for `project` scope); an explicit
  `--target` overrides the computed destination entirely. All harnesses share the same `$HOME` /
  git-root anchors — **only the subpath differs** per harness and scope.
- **SPEC-HARNESS-002** [PINNED] **The descriptor table.** Exactly five rows; this is the data
  Issue 4.2 asserts against.

  | id (`--harness`)    | user_subpath                | project_subpath    | frontmatter_required  | name_transform           |
  |:--------------------|:----------------------------|:-------------------|:----------------------|:-------------------------|
  | `claude-code`       | `.claude/skills`            | `.claude/skills`   | `name`, `description` | (none)                   |
  | `opencode`          | `.config/opencode/skills`   | `.opencode/skills` | `name`, `description` | (none)                   |
  | `pi`                | `.pi/agent/skills`          | `.pi/skills`       | `name`, `description` | `lowercase-hyphen,max64` |
  | `codex`             | `.agents/skills`            | `.agents/skills`   | `name`, `description` | (none)                   |
  | `agents` (portable) | `.agents/skills`            | `.agents/skills`   | `name`, `description` | (none)                   |

- **SPEC-HARNESS-003** [NEW] **Discovery / scope rules.** `user` scope resolves against `$HOME`;
  `project` scope resolves against the git root if inside a repository, else the current working
  directory; an explicit `--target <dir>` override wins over both and bypasses subpath
  resolution.
- **SPEC-HARNESS-004** [NEW] **`--surface` → `--harness` migration/alias.** `--harness`
  (repeatable) replaces the former single `--surface` flag. `--surface` remains as a
  **deprecated, hidden alias** that maps `claude` → `claude-code` and `agents` → `agents`; the
  default harness when neither flag is given is `claude-code`. An unknown/legacy value (not a
  canonical harness id) falls back to the uniform `.<value>/skills` layout for both scopes, so an
  arbitrary historical `.<surface>/skills` install keeps resolving to its original directory.
- **SPEC-HARNESS-005** [NEW] **Codex ↔ agents path overlap + dedupe.** The `codex` harness and
  the portable `agents` harness **both** resolve to `.agents/skills` (identical user and project
  subpaths). When several `--harness` values resolve to the **same absolute path**, the installer
  deploys and records that directory **exactly once** — dedupe is by resolved absolute path, and
  the **first-requested** harness wins the recorded row. This prevents a double write / double
  record when both `codex` and `agents` are requested together.
- **SPEC-HARNESS-006** [NEW] **Extensibility.** Adding a harness is a single new data row in
  `src/harness.rs` plus a new row in the SPEC-HARNESS-002 table — no structural code change.
  Future harnesses (cursor, windsurf, aider, etc.) are additional rows and are **not** in scope
  for plan-008.
- **SPEC-HARNESS-007** [NEW] **Uniform install unit.** The install unit
  (`skills/naba/SKILL.md` plus its `commands/`) and the required frontmatter keys
  (`name`, `description`) are **identical across all harnesses** — only the path data differs.
  There is no per-harness content transform; `name_transform` (where present, e.g. `pi`'s
  `lowercase-hyphen,max64`) constrains the on-disk skill **name**, not the manifest body.

---

## §20 Skills install receipt (SPEC-INSTALL) [NEW — plan-008, Rust-only]

Records every place the skill tree was installed so a later `naba skills upgrade` (and the
post-self-update refresh) can find and refresh **all** of them without the operator re-supplying
`--harness`/`--scope`/`--target`.

- **SPEC-INSTALL-001** [NEW] **Target registry.** A JSON receipt at
  `<config_dir>/skills-install.json` (`crate::config::config_dir()`, the same root as the other
  `self`/`preflight` state — SPEC-DIRS-001) records the set of install **targets**. Each target row
  carries `{ harness, scope, path }` and is **keyed by `(harness, scope, resolved-path)`**.
  `install` (and `upgrade` when it resolves an explicit target) **upserts** the target — an upsert
  on an existing key is idempotent (no duplicate row); a `remove` drops the matching row. The
  receipt version string is normalized on save.
- **SPEC-INSTALL-002** [NEW] **Receipt-driven multi-harness upgrade.** An **unqualified**
  `naba skills upgrade` (no `--harness`/`--scope`/`--target`) enumerates **every recorded target**
  and refreshes each in turn: **continue-on-error** (a failing target is reported but does not abort
  the rest) and **dedupe by resolved absolute path** (a path recorded under two harness ids is
  refreshed once). The post-self-update skills refresh (SPEC-SELF-005) and the `skills preflight`
  gate (SPEC-PREFLIGHT-003) are **receipt-driven** over this same target set.
- **SPEC-INSTALL-003** [NEW] **Legacy migration.** When the receipt is absent or empty,
  `load_or_migrate` **synthesizes** it from a **disk scan** of the legacy install locations
  (`.claude`/`.agents` skill dirs under `$HOME` and the git root) — any location that already holds
  an installed skill becomes a recorded target — then persists the synthesized receipt. The scan is
  **idempotent** (re-running adds nothing new) so an already-migrated repo is a no-op.
