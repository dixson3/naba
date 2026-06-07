# DRIFT-CHECK.md — naba manifest

The `drift-check` engine's per-repo configuration for **this** repository. It declares the
artifact graph the engine verifies: nodes, source-of-truth edges, per-edge contracts, the
changed-path globs that scope an on-edit check, and the fixed-authority policy. The reusable
mechanism (cascade principle, isolated evidence-based sub-agent, the four check categories,
spec-bootstrap/conflict handling) lives in the `drift-check` skill — not here.

naba's Claude-facing assets are the `skills/naba-*` skills, deployed by the frontmatter-driven
`install.py`. There are no per-skill `spec/`, `agents/`, `scripts/`, `formulas/`, or `templates/`
directories, and naba ships no companion rules — so the graph below is deliberately small. It
adds one naba-specific edge the reference repo lacks: every `naba <subcommand>` a skill invokes
must correspond to a real cobra command in the Go CLI source.

## 0. Status

`approved: yes` — operator-approved (plan-001, Issue 5.2 / capability gate, 2026-06-07). The
engine enforces this manifest; it is a silent no-op only while `approved: no`.

## 1. Artifact Nodes

`Kind` ∈ {source, doc, spec}. `Authority` ∈ {fixed, derived}. `Reachability` ∈ {required, optional}.

| Node ID | Glob | Kind | Authority | Reachability |
|---------|------|------|-----------|--------------|
| `skill-md` | `skills/naba-*/SKILL.md` | source | derived | required |
| `frontmatter-contract` | `skills/naba-*/SKILL.md` (frontmatter `name` / `skill-group` / `depends-on-tool` / `depends-on-skill`) | source | derived | required |
| `skill-readme` | `skills/naba-*/README.md` | doc | derived | required |
| `installer` | `install.py` | source | derived | required |
| `project-readme` | `README.md` | doc | derived | required |
| `cli-source` | `internal/cli/*.go` | source | fixed | required |

## 2. Source-of-Truth Edges

`Check Category` ∈ {cross-ref, contract, behavioral, required-section}.

| Edge ID | Source Node | Derived Node | Check Category |
|---------|-------------|--------------|----------------|
| `e-readme-prereqs` | `frontmatter-contract` | `skill-readme` | contract |
| `e-readme-usage` | `skill-md` | `skill-readme` | required-section |
| `e-readme-desc` | `skill-md` | `skill-readme` | contract |
| `e-depends-on-skill` | `frontmatter-contract` | `skill-md` | contract |
| `e-installer-frontmatter` | `frontmatter-contract` | `installer` | contract |
| `e-index-table` | `skill-readme` | `project-readme` | contract |
| `e-cli-subcommand` | `cli-source` | `skill-md` | cross-ref |

## 3. Per-Edge Contracts

`Contract` ∈ {path-resolves, identifier-matches, value-equal, field-set-subset, field-set-equal, section-present}.

| Edge ID | Contract | Verification |
|---------|----------|--------------|
| `e-readme-prereqs` | `field-set-subset` | the skill README Prerequisites match the SKILL.md frontmatter `depends-on-tool` (every naba skill: `[naba]`). Source is frontmatter `depends-on-tool`, not a prereq script. |
| `e-readme-usage` | `section-present` | the skill's own slash command (frontmatter `name`, i.e. `/naba-<x>`) and its usage line in SKILL.md appear in the skill README Usage section. |
| `e-readme-desc` | `value-equal` | the skill README one-line description matches the SKILL.md `description` intent. |
| `e-depends-on-skill` | `path-resolves` | every name in a SKILL.md frontmatter `depends-on-skill` resolves to an in-repo `skills/naba-*` dir (e.g. naba-brand-kit → naba-icon/naba-pattern/naba-generate; naba-storyboard → naba-story/naba-edit). |
| `e-installer-frontmatter` | `field-set-subset` | the frontmatter keys `install.py` reads (`skill-group`, `depends-on-tool`, `depends-on-skill`, plus `name`) are present/parseable in every `skills/naba-*/SKILL.md`; the installer's discovery (a `SKILL.md` per `skills/*/`) matches the on-disk skill set. |
| `e-index-table` | `field-set-equal` | the project README "Available skills" table has exactly one `/naba-*` row per `skills/naba-*/` dir that has a SKILL.md (10 rows). |
| `e-cli-subcommand` | `identifier-matches` | every `naba <subcommand>` a SKILL.md invokes (generate/edit/restore/icon/pattern/story/diagram) corresponds to a real cobra command in `internal/cli/*.go`. CLI source is the fixed authority. |

## 4. Referencers (orphan check)

| Required Node | Valid Referencers |
|---------------|-------------------|
| `skill-md` | every `skills/naba-*/` dir must contain one `SKILL.md` |
| `skill-readme` | every `skills/naba-*/` dir must contain one `README.md` |
| `installer` | referenced by the project README "Claude Code Skills" section |

## 5. Required-Section Contracts

| Required Section | Source Node | Source detail |
|------------------|-------------|---------------|
| One-line description | `skill-readme` | SKILL.md `description` |
| Prerequisites | `skill-readme` | SKILL.md frontmatter `depends-on-tool` (`[naba]`) |
| Usage | `skill-readme` | SKILL.md usage / slash command |
| Install | `skill-readme` | repo-level `install.sh` / `install.py` reference |
| Available-skills table | `project-readme` | one `/naba-*` row per skill |
| Skills install instructions | `project-readme` | `install.sh` actual flags |

## 6. Trigger Scope

A source-node edit fans out to every derived edge it feeds.

| Changed-Path Glob | Scopes To |
|-------------------|-----------|
| `skills/naba-*/SKILL.md` | `e-readme-prereqs`, `e-readme-usage`, `e-readme-desc`, `e-depends-on-skill`, `e-installer-frontmatter`, `e-cli-subcommand` |
| `skills/naba-*/README.md` | `e-readme-prereqs`, `e-readme-usage`, `e-readme-desc`, `e-index-table` |
| `install.py` | `e-installer-frontmatter` |
| `README.md` | `e-index-table` |
| `internal/cli/*.go` | `e-cli-subcommand` |

## 7. Fixed-Authority Conflict Policy

`cli-source` (`internal/cli/*.go`) is the sole `fixed` authority. On an `e-cli-subcommand`
conflict, the Go CLI wins: report the SKILL.md as drifted; never edit the CLI to match a skill.
**Exception:** if the evidence shows a skill names a subcommand that genuinely should exist but
does not (a stale reference on either side), emit a **CONFLICT**, report it to the operator, and
halt; never silently rewrite either side.
