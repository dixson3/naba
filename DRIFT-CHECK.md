# DRIFT-CHECK.md — naba manifest

The `drift-check` engine's per-repo configuration for **this** repository. It declares the
artifact graph the engine verifies: nodes, source-of-truth edges, per-edge contracts, the
changed-path globs that scope an on-edit check, and the fixed-authority policy. The reusable
mechanism (cascade principle, isolated evidence-based sub-agent, the four check categories,
spec-bootstrap/conflict handling) lives in the `drift-check` skill — not here.

naba's Claude-facing asset is the single `skills/naba` skill, invoked as `/naba <subcommand>`
and deployed by the frontmatter-driven `install.py`. One `SKILL.md` carries the router and the
shared guidance; per-subcommand detail lives in `skills/naba/commands/*.md`; one `README.md`
lists the subcommands. The skill ships no per-skill `spec/`, `agents/`, `scripts/`,
`formulas/`, or `templates/` directories and no companion rules — so the graph below is
deliberately small. It adds two naba-specific edges the reference repo lacks: every CLI verb a
`commands/*.md` invokes must correspond to a real cobra command in the Go CLI source, and the
skill's subcommand set must agree with its implementation guide.

## 0. Status

`approved: yes` — operator-approved (plan-002, Issue 2.4, 2026-06-14) for the single-skill
`/naba <subcommand>` layout, superseding the plan-001 10-skill manifest. The drift-verifier
full sweep passed all 7 edges before approval. The engine enforces this manifest; it is a
silent no-op only while `approved: no`.

## 1. Artifact Nodes

`Kind` ∈ {source, doc, spec}. `Authority` ∈ {fixed, derived}. `Reachability` ∈ {required, optional}.

| Node ID | Glob | Kind | Authority | Reachability |
|---------|------|------|-----------|--------------|
| `skill-md` | `skills/naba/SKILL.md` | source | derived | required |
| `frontmatter-contract` | `skills/naba/SKILL.md` (frontmatter `name` / `skill-group` / `depends-on-tool`) | source | derived | required |
| `commands` | `skills/naba/commands/*.md` | source | derived | required |
| `skill-readme` | `skills/naba/README.md` | doc | derived | required |
| `installer` | `install.py` | source | derived | required |
| `project-readme` | `README.md` | doc | derived | required |
| `skill-spec` | `docs/specifications/IG/skills.md` | spec | derived | required |
| `cli-source` | `internal/cli/*.go` | source | fixed | required |

## 2. Source-of-Truth Edges

`Check Category` ∈ {cross-ref, contract, behavioral, required-section}.

| Edge ID | Source Node | Derived Node | Check Category |
|---------|-------------|--------------|----------------|
| `e-readme-prereqs` | `frontmatter-contract` | `skill-readme` | contract |
| `e-readme-usage` | `commands` | `skill-readme` | required-section |
| `e-readme-desc` | `skill-md` | `skill-readme` | contract |
| `e-installer-frontmatter` | `frontmatter-contract` | `installer` | contract |
| `e-index-table` | `skill-readme` | `project-readme` | contract |
| `e-cli-subcommand` | `cli-source` | `commands` | cross-ref |
| `e-skill-spec` | `skill-md` | `skill-spec` | cross-ref |

`e-depends-on-skill` (plan-001) is **deleted**: the composites no longer have sibling skills;
their dependency is now intra-skill router logic, not a `depends-on-skill` frontmatter edge.

## 3. Per-Edge Contracts

`Contract` ∈ {path-resolves, identifier-matches, value-equal, field-set-subset, field-set-equal, section-present}.

| Edge ID | Contract | Verification |
|---------|----------|--------------|
| `e-readme-prereqs` | `field-set-subset` | the `skills/naba/README.md` Prerequisites match the SKILL.md frontmatter `depends-on-tool` (`[naba]`). Source is frontmatter `depends-on-tool`, not a prereq script. |
| `e-readme-usage` | `section-present` | the `skills/naba/README.md` Subcommands table lists every `skills/naba/commands/<sub>.md` (one `/naba <sub>` row per command file). |
| `e-readme-desc` | `value-equal` | the `skills/naba/README.md` intro/description matches the SKILL.md `description` intent. |
| `e-installer-frontmatter` | `field-set-subset` | the frontmatter keys `install.py` reads (`skill-group`, `depends-on-tool`, optional `depends-on-skill`, plus `name`) are present/parseable in `skills/naba/SKILL.md`; the installer's discovery (a `SKILL.md` per `skills/*/`) matches the on-disk skill set (one skill: `naba`). |
| `e-index-table` | `field-set-equal` | the project README "Subcommands" table lists exactly the subcommands in the SKILL.md dispatch table / `skills/naba/commands/` dir (same set, no extras or omissions). |
| `e-cli-subcommand` | `identifier-matches` | every `naba <verb>` an inline `commands/*.md` invokes (generate/edit/restore/icon/pattern/diagram/story) corresponds to a real cobra command in `internal/cli/*.go`. Composite `commands/*.md` (storyboard/batch/brand-kit) invoke only those same verbs — no new cobra command. CLI source is the fixed authority. |
| `e-skill-spec` | `field-set-equal` | the subcommand set + tier (inline/composite) in the SKILL.md dispatch table equals the subcommand→CLI-verb map in `docs/specifications/IG/skills.md` §4. Keeps the IG guide in sync with the skill. |

## 4. Referencers (orphan check)

| Required Node | Valid Referencers |
|---------------|-------------------|
| `skill-md` | the single `skills/naba/` dir must contain one `SKILL.md` |
| `commands` | `skills/naba/commands/` must contain one `<sub>.md` per dispatch-table subcommand |
| `skill-readme` | the `skills/naba/` dir must contain one `README.md` |
| `installer` | referenced by the project README "Claude Code Skills" section |
| `skill-spec` | referenced by AGENTS.md "Claude Code Skills" section and the SKILL.md drift note |

## 5. Required-Section Contracts

| Required Section | Source Node | Source detail |
|------------------|-------------|---------------|
| One-line description | `skill-readme` | SKILL.md `description` |
| Prerequisites | `skill-readme` | SKILL.md frontmatter `depends-on-tool` (`[naba]`) |
| Subcommands table | `skill-readme` | one `/naba <sub>` row per `commands/<sub>.md` |
| Install | `skill-readme` | repo-level `install.sh` / `install.py` reference |
| Subcommands table | `project-readme` | one `/naba <sub>` row per subcommand in the dispatch table |
| Skill install instructions | `project-readme` | `install.sh` actual flags |

## 6. Trigger Scope

A source-node edit fans out to every derived edge it feeds.

| Changed-Path Glob | Scopes To |
|-------------------|-----------|
| `skills/naba/SKILL.md` | `e-readme-prereqs`, `e-readme-desc`, `e-installer-frontmatter`, `e-index-table`, `e-skill-spec` |
| `skills/naba/commands/*.md` | `e-readme-usage`, `e-index-table`, `e-cli-subcommand`, `e-skill-spec` |
| `skills/naba/README.md` | `e-readme-prereqs`, `e-readme-usage`, `e-readme-desc`, `e-index-table` |
| `install.py` | `e-installer-frontmatter` |
| `README.md` | `e-index-table` |
| `docs/specifications/IG/skills.md` | `e-skill-spec` |
| `internal/cli/*.go` | `e-cli-subcommand` |

## 7. Fixed-Authority Conflict Policy

`cli-source` (`internal/cli/*.go`) is the sole `fixed` authority. On an `e-cli-subcommand`
conflict, the Go CLI wins: report the `commands/*.md` as drifted; never edit the CLI to match a
skill. **Exception:** if the evidence shows a `commands/*.md` names a verb that genuinely should
exist but does not (a stale reference on either side), emit a **CONFLICT**, report it to the
operator, and halt; never silently rewrite either side.

For `e-skill-spec`, neither side is `fixed`: `skill-md` and `skill-spec` must agree, but a
mismatch is reported as drift (update whichever is stale) rather than a hard CONFLICT, unless
the operator has designated the IG guide authoritative per AGENTS.md "Specifications".
