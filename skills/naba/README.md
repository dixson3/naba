# naba

One Claude Code skill for the whole `naba` image toolkit, invoked as
`/naba <subcommand> [args]`. The router and shared guidance live in `SKILL.md`; each
subcommand's detail lives in `commands/<sub>.md`.

## Usage

```
/naba <subcommand> [args]
```

Run `/naba help` (or `/naba` with no subcommand) to print the dispatch table.

## Subcommands

| Subcommand   | Tier      | Purpose |
| :----------- | :-------- | :------ |
| `generate`   | inline    | Image from a text prompt (general-purpose). |
| `edit`       | inline    | Modify an existing image with text instructions. |
| `restore`    | inline    | Restore/enhance an existing image (prompt optional). |
| `icon`       | inline    | App icon / logo mark, optionally multi-size. |
| `pattern`    | inline    | Seamless, tileable pattern or texture. |
| `diagram`    | inline    | Rendered technical diagram image. |
| `story`      | inline    | Sequential image series (one call emits N frames). |
| `storyboard` | composite | `story` then per-frame `edit` refinement. |
| `batch`      | composite | A coordinated set of images over a list/spec. |
| `brand-kit`  | composite | Brand asset trio: `icon` + `pattern` + hero `generate`. |

Inline subcommands run a single `naba` CLI call directly. Composite subcommands run a
multi-call loop in a dispatched subagent and return a compact summary.

## Prerequisites

The `naba` CLI must be on PATH (declared in `SKILL.md` frontmatter as
`depends-on-tool: [naba]`). See the repository README for installing the naba binary; when
it is absent the skill installs but is inert.

## Install

Deployed by the repo-level `install.{sh,py}`, which auto-discovers `skills/naba/SKILL.md`
via its frontmatter and copies the whole skill directory (including `commands/`). See the
repository README for install scopes and flags.
