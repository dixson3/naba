Title: skills
Slug: skills
Subtitle: the /naba agent skill and its lifecycle

A **skill** teaches an AI coding agent a new capability: some instructions, plus a command the
agent runs on your behalf. naba's skill turns *"make me an app icon of a rocket ship"* into the
right `naba` invocation — you get naba's images without leaving the conversation or memorizing
CLI flags.

naba ships a single skill that wraps the whole CLI as one slash command with subcommands:
`/naba <subcommand>`. It installs into whichever **agent harness** you use —
[Claude Code](https://claude.com/claude-code) is the default and the running example throughout
this page, but opencode, pi, codex, and a portable `agents` layout work too (see
[Harnesses](#harnesses-one-tree-five-idiomatic-homes) below). The skill files are **embedded in
the binary** at compile time, so installing them is offline and always version-matched — there is
no marketplace plugin and no separate installer script.

> Prefer a desktop assistant that can't run shell commands (Claude Desktop, Cursor)? Use the
> [MCP server](/mcp/) instead — it exposes the same image pipeline over a protocol. The
> [why run naba as an MCP server](/mcp/#why-run-naba-as-an-mcp-server) section explains when to
> reach for which.

## Subcommands

Invoke as `/naba <subcommand> [args]`; run `/naba help` to print the dispatch table.

| Subcommand | Purpose |
|:-----------|:--------|
| `/naba generate` | Generate an image from a text prompt |
| `/naba edit` | Edit an existing image with text instructions |
| `/naba restore` | Restore or enhance an existing image |
| `/naba icon` | Generate app icons (optionally multi-size) |
| `/naba pattern` | Generate seamless patterns and textures |
| `/naba diagram` | Generate technical diagram images |
| `/naba story` | Generate a sequential image series |
| `/naba storyboard` | Composite: story sequence + per-frame edits |
| `/naba batch` | Composite: orchestrate multiple naba calls (icon suites, asset pipelines) |
| `/naba brand-kit` | Composite: icon + pattern + hero image set |

The seven inline subcommands run a single `naba` CLI call directly; the three composites
(`storyboard`, `batch`, `brand-kit`) dispatch a subagent that runs the multi-call loop and
returns a compact summary.

## Implicit triggering

You rarely type `/naba`. The skill's description tells Claude Code to **trigger it
automatically** whenever your request matches an image task — even in plain language. Any of
these fire the right subcommand without a slash command:

- "make me an image of a red apple on white" → `generate`
- "remove the background from logo.png" → `edit`
- "sharpen and denoise this old photo" → `restore`
- "I need an app icon for a rocket ship at 256 and 512" → `icon`
- "give me a seamless circuit-board texture" → `pattern`
- "draw a flowchart of the auth flow" → `diagram`
- "show a 4-frame story of a sailboat's voyage" → `story`

The skill **skips** requests for editable diagram *source* (d2/mermaid text) — `naba diagram`
produces a rendered image, not editable source. Explicit `/naba <subcommand>` invocation always
works too, and is the reliable trigger when you want a specific subcommand.

## Harnesses: one tree, five idiomatic homes

The same embedded skill tree can install to several agent **harnesses**, each at its own
idiomatic skills path. Select a harness with `--harness <name>`; the flag is **repeatable**, so
one `install` can write to several harnesses at once. Supported values and their install paths:

| Harness | Flag | Project-scope path | User-scope path |
|:--------|:-----|:-------------------|:----------------|
| **claude-code** (default) | `--harness claude-code` | `<root>/.claude/skills` | `~/.claude/skills` |
| **opencode** | `--harness opencode` | `<root>/.opencode/skills` | `~/.config/opencode/skills` |
| **pi** | `--harness pi` | `<root>/.pi/skills` | `~/.pi/agent/skills` |
| **codex** | `--harness codex` | `<root>/.agents/skills` | `~/.agents/skills` |
| **agents** (portable) | `--harness agents` | `<root>/.agents/skills` | `~/.agents/skills` |

The `claude-code` harness is the default. The portable **`agents`** harness writes the generic
`.agents/skills` location that opencode, pi, and codex all read, so it is the one-shot way to
cover multiple generic harnesses. Install to several at once by repeating the flag:

```bash
naba skills install --harness claude-code --harness opencode --harness pi
```

> The old `--surface claude|agents` flag still works as a deprecated hidden alias
> (`claude` → `claude-code`, `agents` → `agents`) and prints a deprecation notice. Prefer
> `--harness`.

## Scopes: user vs project

Scope chooses the `<root>` each harness installs under:

| Scope | Flag | Root |
|:------|:-----|:-----|
| **user** (default) | `--scope user` | `$HOME` (e.g. `~/.claude/skills`) |
| **project** | `--scope project` | the git root (else the current directory) |

For a fully explicit destination, `--target DIR` overrides scope/harness entirely.

## Lifecycle

```bash
naba skills install                       # default: claude-code harness, user scope -> ~/.claude/skills
naba skills install --dry-run             # show what would be written, change nothing
naba skills install --scope project       # install into <git-root>/.claude/skills
naba skills install --harness opencode    # install into ~/.config/opencode/skills
naba skills install --harness claude-code --harness pi  # repeatable: several harnesses at once
naba skills install --target DIR          # install into an explicit directory
naba skills upgrade                       # refresh every previously-installed harness target
naba skills remove                        # remove the naba skill again
naba skills status                        # report up-to-date / complete / unmodified
naba skills preflight --json              # fast, offline skill-gate (see below)
```

Every verb accepts `--json` for a machine-readable `{status, data}` envelope, plus the shared
`--scope`, `--harness` (repeatable), `--target`, and `--dry-run` flags.

Run **`naba skills upgrade`** after any `naba` upgrade (a `self update` already does this unless
you pass `--binary-only`) so the installed skill always matches the binary. With no flags,
`upgrade` refreshes **every** harness target you previously installed to — tracked in an install
receipt — and continues on error, so a multi-harness install stays in sync in one call.

### Integrity marker

On `install`/`upgrade` naba writes a hidden integrity marker into the deployed `SKILL.md`
(`<!-- naba-skills: v=<version> tree=<sha256> -->`); `status` and `naba doctor` use it to
confirm the install is current, complete, and unmodified. The repository source
`skills/naba/SKILL.md` stays marker-free.

### Preflight

`naba skills preflight --json` is a fast, **offline** gate the `/naba` skill runs at trigger
time (`naba doctor` is the full, network-touching sweep). It reports three axes:

- **auth** — the effective provider's key is present (no network call)
- **skills** — the installed skill tree matches this binary
- **binary** — a tri-state `up_to_date | update_available | unknown`

Overall `status` is `ok` unless auth or skills fails (`auth_missing` / `skills_outdated`); the
gate exits non-zero on a non-`ok` status, so it gates cleanly on its exit code.

## Prerequisites

The skill shells out to the `naba` CLI, so the **`naba` binary must be installed and on PATH**
(see [install](/install/)) and a provider API key set — `GEMINI_API_KEY`,
`OPENROUTER_API_KEY`, or `AWS_BEARER_TOKEN_BEDROCK` (see [config](/config/)).
`naba skills install` always writes the skill files; the skill is inert until the binary is on
PATH.
