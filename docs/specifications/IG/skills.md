# Implementation Guide: Claude Code Skill Layer

## 1. Overview

The naba project ships a single Claude Code skill, `skills/naba`, invoked as
`/naba <subcommand> [args]`. The skill is a **packaging layer over the already-specified
CLI** (see PRD.md FS-001…FS-010 and the other IG guides) — it adds no image-generation
capability of its own. Each subcommand maps to one or more existing cobra commands in
`internal/cli/*.go`; the skill never reaches past the public `naba` binary.

The skill replaced 10 separate `skills/naba-*` skills (one per subcommand) in plan-002.
Consolidation removed verbatim-duplicated guidance (prompt framing, anti-patterns, global
flags) that previously lived in all 10 `SKILL.md` files, and changed invocation from
`/naba-<sub>` to `/naba <sub>`.

## 2. Use Cases

| ID | Name | Actor | Preconditions | Flow | Postconditions |
|----|------|-------|---------------|------|----------------|
| UC-SK-001 | Invoke an inline subcommand | Claude Code user | `naba` on PATH; skill installed | 1. User runs `/naba generate "<prompt>"` 2. Router parses `$ARGUMENTS`, first token = subcommand 3. Router reads `commands/generate.md` 4. Runs the single `naba generate` CLI call 5. Presents output path | Image generated; path reported |
| UC-SK-002 | Invoke a composite subcommand | Claude Code user | `naba` on PATH; skill installed | 1. User runs `/naba brand-kit "<brand>"` 2. Router identifies composite tier 3. Router spawns a subagent (`Agent`) with the absolute `commands/brand-kit.md` path + shared guidance + args 4. Subagent runs the multi-call loop 5. Subagent returns a compact summary | Asset set generated; summary (paths/manifest) returned to parent context |
| UC-SK-003 | Discover subcommands | Claude Code user | Skill installed | 1. User runs `/naba help` (or `/naba` with no token, or an unknown token) 2. Router prints the dispatch table | Subcommand list shown; nothing executed |

## 3. Invocation

```
/naba <subcommand> [args]
```

The skill is `user-invocable: true`. The trailing tokens arrive via the `$ARGUMENTS`
placeholder (Claude Code skill contract; absent the placeholder they are appended as
`ARGUMENTS: <value>`). The SKILL.md body is a **router** that splits `$ARGUMENTS` on
whitespace, treats the first token as the subcommand, and dispatches per the table below.
`help` / empty / unknown → print the dispatch table.

## 4. Subcommand → CLI-verb map

Seven subcommands map 1:1 to a single cobra command (**inline** tier). Three are
**composite** — they orchestrate multiple existing verbs and have no cobra command of their
own (they are a skill-level layer, not added to `internal/cli/`).

| Subcommand   | Tier      | CLI verb(s) invoked | cobra source |
| :----------- | :-------- | :------------------ | :----------- |
| `generate`   | inline    | `naba generate` | `internal/cli/generate.go` |
| `edit`       | inline    | `naba edit` | `internal/cli/edit.go` |
| `restore`    | inline    | `naba restore` | `internal/cli/restore.go` |
| `icon`       | inline    | `naba icon` | `internal/cli/icon.go` |
| `pattern`    | inline    | `naba pattern` | `internal/cli/pattern.go` |
| `diagram`    | inline    | `naba diagram` | `internal/cli/diagram.go` |
| `story`      | inline    | `naba story` | `internal/cli/story.go` |
| `storyboard` | composite | `naba story`, then `naba edit` per frame | (orchestration; no cobra command) |
| `batch`      | composite | sequence of `naba generate`/`icon`/`pattern`/… | (orchestration; no cobra command) |
| `brand-kit`  | composite | `naba icon` + `naba pattern` + `naba generate` | (orchestration; no cobra command) |

`story` is inline despite emitting multiple frames: `naba story` is a **single** CLI
invocation that loops internally (`runStory` in `story.go`).

## 5. Hybrid dispatch model

- **Inline tier** — single-call subcommands. The router reads `commands/<sub>.md` and runs
  the `naba` call directly in the parent context. Cheap; no subagent overhead.
- **Composite tier** — multi-call subcommands. The router spawns a subagent (the `Agent`
  tool) so the per-image loop output stays out of the parent context. The dispatch prompt
  passes the **absolute** path `${CLAUDE_SKILL_DIR}/commands/<sub>.md`, the shared guidance,
  and the user args, and requires a compact summary in return. The subagent's own tool grant
  supplies its file-writing tools (`Bash` for the `naba` calls, `Write` for a manifest); the
  parent skill's `allowed-tools` needs only `[Bash, Read, Agent]`. The composite does not
  depend on a child `Glob` grant (it may be absent depending on the agent type) — each item
  is written to an explicit `-o` path and results are listed with `Bash`.

`${CLAUDE_SKILL_DIR}` resolves the skill's **deployed** base directory across all install
scopes (`~/.claude/skills/`, project `.claude/skills/`, `--target` dir), so the subagent
gets a path it can actually read regardless of where the skill was installed.

## 6. Where shared guidance lives

The prompt-engineering order (subject + composition + style + lighting + details), the
anti-patterns list, and the global-flags table are authored **once** in `SKILL.md` (the
"Shared guidance" section), so they are in context on every `/naba` invocation. Composite
subagents are told to read `SKILL.md` (or are handed the guidance inline) so they apply the
same rules. Each `commands/<sub>.md` carries only per-command specifics — usage, the
command's own flag table, command-specific prompt nuance, and examples — and never repeats
the shared guidance.

## 7. Deployment lifecycle (`naba skills`)

The skill files ship **embedded in the `naba` binary** via `go:embed` (a repo-root package
embeds `skills/`, because a `//go:embed` directive cannot reference a parent directory from
`cmd/naba`). The `naba skills` command group is the canonical installer; it supersedes the
former `install.sh`/`install.py` (removed in plan-003).

| Verb | Behavior |
| :--- | :------- |
| `install` | Write the embedded skill tree to the resolved destination, injecting the integrity marker into the deployed `SKILL.md`. |
| `upgrade` | Rewrite each dest file from the (marker-free) embed, inject a fresh marker (idempotent — strips any existing marker first), and **prune** dest files absent from the embed (`rsync --delete` parity). |
| `remove` | Delete the deployed skill directory. |
| `status` | Read the marker and report **up-to-date** / **complete** / **unmodified**. |

Destination resolution mirrors the former installer: an explicit `--target` wins; otherwise
the anchor is `$HOME` (`--scope user`, default) or the git root / cwd (`--scope project`),
joined with `.<surface>/skills` (`--surface claude` default, or `agents`). `--dry-run`
prints actions and changes nothing.

### Integrity marker

On `install`/`upgrade`, a single hidden HTML-comment marker is injected into the deployed
`SKILL.md`, immediately after the YAML frontmatter (so it never breaks the frontmatter
parse):

```
<!-- naba-skills: v=<naba-version> tree=<sha256> -->
```

- `<sha256>` is the **canonical tree hash**: sha256 over, for each file sorted by relative
  path, the relative-path bytes then the file bytes — raw, with no line-ending or
  trailing-newline normalization. The `SKILL.md` marker line is stripped before hashing, so
  a deployed (marked) tree hashes identically to the marker-free embed.
- The binary hashes its own `embed.FS` at runtime (deterministic, no build step).
- The **repo source `skills/naba/SKILL.md` stays marker-free**; the marker exists only in a
  deployed copy.
- `status` reports **up-to-date** (deployed marker `tree` == this binary's embedded hash),
  **complete** (every embedded file present), and **unmodified** (recomputed deployed hash,
  marker stripped, == embedded hash).

### `naba doctor` skill-match check

`naba doctor` reuses `SkillStatus` against the default user/claude destination (unless
`--scope`/`--surface`/`--target` are given) and reports a **fail** when the skill is not
installed, the marker is missing, the marker `tree` hash ≠ the binary's embedded hash
(outdated), or the install is incomplete/modified. doctor's other checks (API key present,
live `models.list` key validation, configured-model reachability, config parseable,
version) are covered in [configuration.md](configuration.md) and
[image-generation.md](image-generation.md).

## 8. Drift contract

`DRIFT-CHECK.md` enforces agreement between this guide and the skill:

- `e-skill-spec`: the subcommand set / dispatch table here agrees with
  `skills/naba/SKILL.md` and the `commands/*.md` files.
- `e-cli-subcommand`: every CLI verb a `commands/*.md` invokes is a real cobra command in
  `internal/cli/*.go` (the inline-tier rows of §4).

When subcommands are added, removed, or retiered, update this guide, `SKILL.md`, the
`commands/` directory, and the README subcommand table together.
