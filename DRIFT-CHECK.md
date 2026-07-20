# DRIFT-CHECK.md — naba manifest

The `drift-check` engine's per-repo configuration for **this** repository. It declares the
artifact graph the engine verifies: nodes, source-of-truth edges, per-edge contracts, the
changed-path globs that scope an on-edit check, and the fixed-authority policy. The reusable
mechanism (cascade principle, isolated evidence-based sub-agent, the four check categories,
spec-bootstrap/conflict handling) lives in the `drift-check` skill — not here.

naba's Claude-facing asset is the single `skills/naba` skill, invoked as `/naba <subcommand>`
and deployed by `naba skills install` (the skill tree is embedded in the binary at compile time
via `include_dir` in `src/embed.rs`; the legacy shell/python installer scripts were removed in
plan-003). One `SKILL.md` carries the router and the shared guidance; per-subcommand detail
lives in `skills/naba/commands/*.md`; one `README.md` lists the subcommands. The skill ships no
per-skill `spec/`, `agents/`, `scripts/`, `formulas/`, or `templates/` directories and no
companion rules — so the graph below is deliberately small. It adds naba-specific edges the
reference repo lacks: every CLI verb a `commands/*.md` or the `SKILL.md` preflight step invokes
must correspond to a real clap command in the Rust CLI source; the skill's subcommand set must
agree with its implementation guide; and the default-model id in the Rust provider client must
agree with the model id stated in the docs.

## 0. Status

`approved: yes` — operator-approved for the single-skill `/naba <subcommand>` layout
(plan-002, 2026-06-14), re-approved for the plan-003 changes (2026-06-14), **and re-approved
for the plan-004 Go→Rust port + plan-005 self-update/preflight additions** (2026-07-12): the
Go source nodes (`internal/cli/*.go`, `internal/cli/skills.go`, `internal/gemini/client.go`)
were retargeted to their Rust equivalents (`src/cli.rs`, `src/skills.rs`,
`src/provider/gemini.rs`), and the plan-005 `self`/`skills preflight` surfaces were added
(`self-source` node, `e-skill-preflight` / `e-readme-self` edges). **Re-approved for the
plan-006 website (2026-07-19):** the `web/` documentation pages (`web-usage`, `web-config`,
`web-install`) were added as `derived` nodes that must follow the fixed Rust capability
sources, so a feature/capability change in the CLI, self-management, or provider client fans
out to the site docs (`e-web-usage-commands`, `e-web-config-self`, `e-web-config-model`,
`e-web-install-methods`). **Re-approved for the plan-007 multi-provider config + provider
ecosystem (2026-07-19):** two new fixed sources were added — `mcp-source` (`src/mcp.rs`, the 8
MCP tools + the `skill://` lazy-loading resources) and `provider-source`
(`src/provider/registry.rs`, `src/provider/bedrock.rs`, `src/config.rs`, the provider registry +
Bedrock + the nested per-provider config schema) — feeding two new `derived` web pages
(`web-skills`, `web-mcp`) plus the rewritten config page, via `e-web-config-providers`,
`e-web-skills-lifecycle`, `e-web-skills-subcommands`, and `e-web-mcp-tools`; and a
README ↔ web install/config sync edge (`e-readme-web-install`) was added so the project README
and the site's install/config pages stay in agreement. **Re-approved for the plan-008 dual-purpose
skills render (2026-07-20):** `skills/naba/SKILL.md` is now a **minijinja template** gated by
`{% if cli %}`; `build.rs` renders it into two embedded trees under `$OUT_DIR` — `cli/` (embedded
via `include_dir!("$OUT_DIR/cli")`, byte-identical to the source, the tree `skills install`
deploys) and `mcp/` (the subtractive MCP-flavored render the `skill://` resource surface serves).
The `e-installer-skillset` contract text was updated accordingly; the source `skills/naba/` dir
does **not** move (the render targets `$OUT_DIR`), so no `skill-md`/`commands` node glob change is
needed. The engine enforces this manifest; it is a silent no-op only while `approved: no`.

## 1. Artifact Nodes

`Kind` ∈ {source, doc, spec}. `Authority` ∈ {fixed, derived}. `Reachability` ∈ {required, optional}.

| Node ID | Glob | Kind | Authority | Reachability |
|:---------|:------|:------|:-----------|:--------------|
| `skill-md` | `skills/naba/SKILL.md` | source | derived | required |
| `frontmatter-contract` | `skills/naba/SKILL.md` (frontmatter `name` / `skill-group` / `depends-on-tool`) | source | derived | required |
| `commands` | `skills/naba/commands/*.md` | source | derived | required |
| `skill-readme` | `skills/naba/README.md` | doc | derived | required |
| `installer` | `src/skills.rs` | source | derived | required |
| `project-readme` | `README.md` | doc | derived | required |
| `skill-spec` | `docs/specifications/IG/skills.md` | spec | derived | required |
| `cli-source` | `src/cli.rs` | source | fixed | required |
| `self-source` | `src/self_cmd/*.rs`, `src/preflight.rs`, `src/dirs.rs` | source | fixed | required |
| `gemini-source` | `src/provider/gemini.rs` (`DEFAULT_MODEL` constant) | source | fixed | required |
| `mcp-source` | `src/mcp.rs` | source | fixed | required |
| `provider-source` | `src/provider/registry.rs`, `src/provider/bedrock.rs`, `src/config.rs` | source | fixed | required |
| `ig-configuration` | `docs/specifications/IG/configuration.md` | spec | derived | required |
| `edd-core` | `docs/specifications/EDD/CORE.md` | spec | derived | required |
| `web-usage` | `web/content/pages/usage.md` | doc | derived | required |
| `web-config` | `web/content/pages/config.md` | doc | derived | required |
| `web-install` | `web/content/pages/install.md` | doc | derived | required |
| `web-skills` | `web/content/pages/skills.md` | doc | derived | required |
| `web-mcp` | `web/content/pages/mcp.md` | doc | derived | required |

## 2. Source-of-Truth Edges

`Check Category` ∈ {cross-ref, contract, behavioral, required-section}.

| Edge ID | Source Node | Derived Node | Check Category |
|:---------|:-------------|:--------------|:----------------|
| `e-readme-prereqs` | `frontmatter-contract` | `skill-readme` | contract |
| `e-readme-usage` | `commands` | `skill-readme` | required-section |
| `e-readme-desc` | `skill-md` | `skill-readme` | contract |
| `e-installer-skillset` | `skill-md` | `installer` | cross-ref |
| `e-index-table` | `skill-readme` | `project-readme` | contract |
| `e-cli-subcommand` | `cli-source` | `commands` | cross-ref |
| `e-skill-preflight` | `cli-source` | `skill-md` | cross-ref |
| `e-readme-self` | `self-source` | `project-readme` | cross-ref |
| `e-skill-spec` | `skill-md` | `skill-spec` | cross-ref |
| `e-model-ig-config` | `gemini-source` | `ig-configuration` | contract |
| `e-model-edd-core` | `gemini-source` | `edd-core` | contract |
| `e-web-usage-commands` | `cli-source` | `web-usage` | cross-ref |
| `e-web-config-self` | `self-source` | `web-config` | cross-ref |
| `e-web-config-model` | `gemini-source` | `web-config` | contract |
| `e-web-config-providers` | `provider-source` | `web-config` | cross-ref |
| `e-web-install-methods` | `self-source` | `web-install` | cross-ref |
| `e-web-skills-lifecycle` | `cli-source` | `web-skills` | cross-ref |
| `e-web-skills-subcommands` | `skill-md` | `web-skills` | cross-ref |
| `e-web-mcp-tools` | `mcp-source` | `web-mcp` | cross-ref |
| `e-readme-web-install` | `web-install` | `project-readme` | contract |

`e-depends-on-skill` (plan-001) is **deleted**: the composites no longer have sibling skills;
their dependency is now intra-skill router logic, not a `depends-on-skill` frontmatter edge.

## 3. Per-Edge Contracts

`Contract` ∈ {path-resolves, identifier-matches, value-equal, field-set-subset, field-set-equal, section-present}.

| Edge ID | Contract | Verification |
|:---------|:----------|:--------------|
| `e-readme-prereqs` | `field-set-subset` | the `skills/naba/README.md` Prerequisites match the SKILL.md frontmatter `depends-on-tool` (`[naba]`). Source is frontmatter `depends-on-tool`, not a prereq script. |
| `e-readme-usage` | `section-present` | the `skills/naba/README.md` Subcommands table lists every `skills/naba/commands/<sub>.md` (one `/naba <sub>` row per command file). |
| `e-readme-desc` | `value-equal` | the `skills/naba/README.md` intro/description matches the SKILL.md `description` intent. |
| `e-installer-skillset` | `field-set-equal` | the `naba skills` installer (`src/skills.rs`, operating over the binary-embedded skill tree in `src/embed.rs`) deploys exactly the skill set — one dir per skill (one skill: `naba`) — and on `install`/`upgrade` injects the integrity marker into the deployed `SKILL.md`. `skills/naba/SKILL.md` is a **minijinja template**; `build.rs` renders it into `$OUT_DIR/cli` + `$OUT_DIR/mcp`, and the embed is `include_dir!("$OUT_DIR/cli")` — the installer deploys the **rendered `cli/` tree** (byte-identical to the source, so the deployed file set still equals the `skills/naba/` source set by construction). The MCP `skill://` surface serves the parallel `mcp/` render. |
| `e-index-table` | `field-set-equal` | the project README "Subcommands" table lists exactly the subcommands in the SKILL.md dispatch table / `skills/naba/commands/` dir (same set, no extras or omissions). |
| `e-cli-subcommand` | `identifier-matches` | every `naba <verb>` an inline `commands/*.md` invokes (generate/edit/restore/icon/pattern/diagram/story) corresponds to a real clap command in `src/cli.rs`. Composite `commands/*.md` (storyboard/batch/brand-kit) invoke only those same verbs — no new command. CLI source is the fixed authority. |
| `e-skill-preflight` | `identifier-matches` | the `naba skills preflight` invocation in the SKILL.md `## Preflight` section, and any `naba self …` verb the docs reference, correspond to real clap subcommands in `src/cli.rs` (`SkillsCommand::Preflight`, `Commands::SelfCmd` → `Update`/`Install`/`Uninstall`). CLI source is the fixed authority. |
| `e-readme-self` | `field-set-equal` | the project README "Self-update" section documents exactly the `naba self` verbs that exist (`update`/`install --from-build`/`uninstall`) and the "Skill-gate preflight" subsection documents `naba skills preflight`; no README-documented `self`/`preflight` verb lacks a real command, and none is omitted. `self-source` is the fixed authority. |
| `e-skill-spec` | `field-set-equal` | the subcommand set + tier (inline/composite) in the SKILL.md dispatch table equals the subcommand→CLI-verb map in `docs/specifications/IG/skills.md` §4. Keeps the IG guide in sync with the skill. |
| `e-model-ig-config` | `value-equal` | the `DEFAULT_MODEL` constant in `src/provider/gemini.rs` equals the default model id stated in `docs/specifications/IG/configuration.md` (the Model Resolution Order built-in default). `gemini-source` is the fixed authority. |
| `e-model-edd-core` | `value-equal` | the `DEFAULT_MODEL` constant in `src/provider/gemini.rs` equals the **Default model** id stated in `docs/specifications/EDD/CORE.md` §5. `gemini-source` is the fixed authority. |
| `e-web-usage-commands` | `identifier-matches` | every `naba <verb>` shown on the usage page (`web/content/pages/usage.md` — generate/edit/restore/icon/pattern/diagram/story) corresponds to a real clap command in `src/cli.rs`; a new user-facing command/verb must be reflected on the usage page. `cli-source` is the fixed authority. |
| `e-web-config-self` | `field-set-subset` | the `naba self …`, `naba doctor`, and `naba skills …` verbs documented on the config page (`web/content/pages/config.md`) correspond to real clap subcommands in `src/cli.rs` / `src/self_cmd/*.rs`; no config-page verb lacks a real command. `self-source` is the fixed authority. |
| `e-web-config-model` | `value-equal` | the default Gemini model id and `--quality` tier ids (`fast`→flash, `high`→pro) stated on the config page match the `DEFAULT_MODEL` constant and the quality→model mapping in `src/provider/gemini.rs`. `gemini-source` is the fixed authority. |
| `e-web-install-methods` | `field-set-subset` | the install methods and the `naba skills install`/`upgrade` + `naba self install --from-build` lifecycle documented on the install page (`web/content/pages/install.md`) correspond to real commands in `src/cli.rs` / `src/self_cmd/*.rs`. `self-source`/`cli-source` are the fixed authority. |
| `e-web-config-providers` | `field-set-subset` | the provider list, per-provider default models, the uniform api-key resolution precedence, the Bedrock auth modes + region default, the `naba provider`/`naba models` commands, and the nested config-key set (`default-provider`, `<provider>.model`/`.api-key`/`.api-key-envvar`) stated on the config page (`web/content/pages/config.md`) match the provider registry (`src/provider/registry.rs`), the Bedrock provider (`src/provider/bedrock.rs` — `DEFAULT_REGION`, `CURATED_MODELS`, auth modes), and the config schema/env-var constants + `valid_keys` in `src/config.rs`. `provider-source` is the fixed authority. |
| `e-web-skills-lifecycle` | `field-set-subset` | the `naba skills` verbs (`install`/`upgrade`/`remove`/`status`/`preflight`) and their `--scope`/`--surface`/`--target`/`--dry-run`/`--json` flags documented on the skills page (`web/content/pages/skills.md`) correspond to real clap subcommands + args in `src/cli.rs` (`SkillsCommand`, `SkillsArgs`). `cli-source` is the fixed authority. |
| `e-web-skills-subcommands` | `field-set-equal` | the `/naba <subcommand>` set on the skills page (`web/content/pages/skills.md`) equals the SKILL.md dispatch table (inline: generate/edit/restore/icon/pattern/diagram/story; composite: storyboard/batch/brand-kit) — no extras or omissions. `skill-md` is the source. |
| `e-web-mcp-tools` | `field-set-equal` | the 8 MCP tools + their required/optional params, the `skill://<name>` / `skill://<name>/<rel>` / `file://` resource surface, and `NABA_OUTPUT_DIR` resolution documented on the MCP page (`web/content/pages/mcp.md`) equal the tools + schemas + resource handlers in `src/mcp.rs` (`tools()`, `skill_resources`, `read_skill_resource`, `resolve_output_dir`). `mcp-source` is the fixed authority. |
| `e-readme-web-install` | `field-set-subset` | the project README Install / Setup / Providers / Configuration / MCP sections agree with the site install + config pages (`web/content/pages/install.md`, `config.md`) on the install methods, the nested per-provider config surface, the providers (incl. Bedrock) + api-key resolution, and the MCP pointer. Neither side is `fixed` (both derive from the Rust sources); a mismatch is drift on whichever is stale. |

## 4. Referencers (orphan check)

| Required Node | Valid Referencers |
|:---------------|:-------------------|
| `skill-md` | the single `skills/naba/` dir must contain one `SKILL.md` |
| `commands` | `skills/naba/commands/` must contain one `<sub>.md` per dispatch-table subcommand |
| `skill-readme` | the `skills/naba/` dir must contain one `README.md` |
| `installer` | the `naba skills` command (`src/skills.rs`), referenced by the project README "Claude Code Skills" / "Install the skill" sections and AGENTS.md "Claude Code Skills" |
| `self-source` | `naba self` / `naba skills preflight` (`src/self_cmd/`, `src/preflight.rs`, `src/dirs.rs`), referenced by the project README "Self-update" / "Skill-gate preflight" sections, the SKILL.md `## Preflight` section, AGENTS.md "Architecture"/"Distribution", and SPEC §15–§18 |
| `skill-spec` | referenced by AGENTS.md "Claude Code Skills" section and the SKILL.md drift note |
| `gemini-source` | the `DEFAULT_MODEL` constant is consumed by `src/commands.rs`, `src/doctor.rs`, and `src/mcp.rs` (client construction) |
| `ig-configuration` | referenced by AGENTS.md "Specifications" (docs/specifications/IG) and EDD/CORE |
| `edd-core` | referenced by AGENTS.md "Specifications" (docs/specifications/EDD) |
| `mcp-source` | `src/mcp.rs` (the 8 MCP tools + `skill://` resource handlers), consumed by the `naba mcp` command (`src/cli.rs`) and referenced by the website MCP page and the project README "MCP Server" section |
| `provider-source` | `src/provider/registry.rs` / `src/provider/bedrock.rs` / `src/config.rs`, consumed by `src/commands.rs` (`naba provider`/`naba models`), the selector (`src/provider/select.rs`), and referenced by the website config page + README "Providers"/"Configuration" sections and AGENTS.md |
| `web-usage` | the naba website usage page (`web/content/pages/usage.md`), linked from the site nav and `web/README.md` |
| `web-config` | the naba website config page (`web/content/pages/config.md`), linked from the site nav and `web/README.md` |
| `web-install` | the naba website install page (`web/content/pages/install.md`), linked from the site nav and `web/README.md` |
| `web-skills` | the naba website skills page (`web/content/pages/skills.md`), linked from the site nav (`MENUITEMS`), the home cards, and the install/config pages |
| `web-mcp` | the naba website MCP page (`web/content/pages/mcp.md`), linked from the site nav (`MENUITEMS`), the home cards, and the config page |

## 5. Required-Section Contracts

| Required Section | Source Node | Source detail |
|:------------------|:-------------|:---------------|
| One-line description | `skill-readme` | SKILL.md `description` |
| Prerequisites | `skill-readme` | SKILL.md frontmatter `depends-on-tool` (`[naba]`) |
| Subcommands table | `skill-readme` | one `/naba <sub>` row per `commands/<sub>.md` |
| Install | `skill-readme` | `naba skills install` reference (binary-embedded skill) |
| Subcommands table | `project-readme` | one `/naba <sub>` row per subcommand in the dispatch table |
| Skill install instructions | `project-readme` | `naba skills install`/`upgrade`/`remove`/`status` verbs + flags |
| Self-update | `project-readme` | `naba self update`/`install --from-build`/`uninstall` + the vendor `curl\|sh` install; Homebrew remains the documented default |

## 6. Trigger Scope

A source-node edit fans out to every derived edge it feeds.

| Changed-Path Glob | Scopes To |
|:-------------------|:-----------|
| `skills/naba/SKILL.md` | `e-readme-prereqs`, `e-readme-desc`, `e-installer-skillset`, `e-index-table`, `e-skill-preflight`, `e-skill-spec`, `e-web-skills-subcommands` |
| `skills/naba/commands/*.md` | `e-readme-usage`, `e-index-table`, `e-cli-subcommand`, `e-skill-spec` |
| `skills/naba/README.md` | `e-readme-prereqs`, `e-readme-usage`, `e-readme-desc`, `e-index-table` |
| `src/skills.rs` | `e-installer-skillset` |
| `README.md` | `e-index-table`, `e-readme-self`, `e-readme-web-install` |
| `docs/specifications/IG/skills.md` | `e-skill-spec` |
| `src/cli.rs` | `e-cli-subcommand`, `e-skill-preflight`, `e-web-usage-commands`, `e-web-install-methods`, `e-web-skills-lifecycle` |
| `src/self_cmd/*.rs`, `src/preflight.rs`, `src/dirs.rs` | `e-readme-self`, `e-skill-preflight`, `e-web-config-self`, `e-web-install-methods` |
| `src/provider/gemini.rs` | `e-model-ig-config`, `e-model-edd-core`, `e-web-config-model` |
| `src/provider/registry.rs`, `src/provider/bedrock.rs`, `src/config.rs` | `e-web-config-providers` |
| `src/mcp.rs` | `e-web-mcp-tools` |
| `docs/specifications/IG/configuration.md` | `e-model-ig-config` |
| `docs/specifications/EDD/CORE.md` | `e-model-edd-core` |
| `web/content/pages/usage.md` | `e-web-usage-commands` |
| `web/content/pages/config.md` | `e-web-config-self`, `e-web-config-model`, `e-web-config-providers` |
| `web/content/pages/install.md` | `e-web-install-methods`, `e-readme-web-install` |
| `web/content/pages/skills.md` | `e-web-skills-lifecycle`, `e-web-skills-subcommands` |
| `web/content/pages/mcp.md` | `e-web-mcp-tools` |

## 7. Fixed-Authority Conflict Policy

Five `fixed` authorities: `cli-source` (`src/cli.rs`), `self-source` (`src/self_cmd/*.rs`,
`src/preflight.rs`, `src/dirs.rs`), `gemini-source` (`src/provider/gemini.rs`), `mcp-source`
(`src/mcp.rs`), and `provider-source` (`src/provider/registry.rs`, `src/provider/bedrock.rs`,
`src/config.rs`).

On an `e-cli-subcommand`, `e-skill-preflight`, `e-readme-self`, `e-web-usage-commands`,
`e-web-config-self`, `e-web-install-methods`, `e-web-skills-lifecycle`, `e-web-mcp-tools`, or
`e-web-config-providers` conflict, the Rust source wins: report the `commands/*.md` / `SKILL.md`
/ README / `web/content/pages/*.md` as drifted; never edit the CLI / MCP / provider source to
match a doc. **Exception:** if the evidence shows a doc names a verb/tool/provider that genuinely
should exist but does not (a stale reference on either side), emit a **CONFLICT**, report it to
the operator, and halt; never silently rewrite either side.

For `e-web-skills-subcommands` and `e-readme-web-install`, neither side is `fixed` (both derive
from the fixed Rust sources): a mismatch is reported as drift (update whichever is stale) rather
than a hard CONFLICT.

On an `e-model-ig-config`, `e-model-edd-core`, or `e-web-config-model` conflict, the Rust
`DEFAULT_MODEL` constant wins: report the doc (`ig-configuration` / `edd-core` /
`web/content/pages/config.md`) as drifted and update the doc to match the
constant; never edit the constant to match a doc. (The constant is also guarded by a
compile-time test assertion, so a model change is a deliberate, test-gated edit that the docs
must follow.)

For `e-skill-spec`, neither side is `fixed`: `skill-md` and `skill-spec` must agree, but a
mismatch is reported as drift (update whichever is stale) rather than a hard CONFLICT, unless
the operator has designated the IG guide authoritative per AGENTS.md "Specifications".
