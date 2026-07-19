Title: skills
Slug: skills
Subtitle: the /naba Claude Code skill and its lifecycle

naba ships a single [Claude Code](https://claude.com/claude-code) skill that wraps the whole
CLI as one slash command with subcommands: `/naba <subcommand>`. The skill files are
**embedded in the binary** at compile time, so installing them is offline and always
version-matched — there is no marketplace plugin and no separate installer script.

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

## Surfaces: claude vs agents

The same embedded skill tree can install to two surfaces:

| Surface | Flag | Destination |
|:--------|:-----|:------------|
| **claude** (default) | `--surface claude` | `<root>/.claude/skills` — the Claude Code skill directory |
| **agents** | `--surface agents` | `<root>/.agents/skills` — the generic agents surface |

The `claude` surface is the default; use `--surface agents` for harnesses that read the generic
`.agents/skills` location.

## Scopes: user vs project

Scope chooses the `<root>` the surface installs under:

| Scope | Flag | Root |
|:------|:-----|:-----|
| **user** (default) | `--scope user` | `$HOME` (e.g. `~/.claude/skills`) |
| **project** | `--scope project` | the git root (else the current directory) |

For a fully explicit destination, `--target DIR` overrides scope/surface entirely.

## Lifecycle

```bash
naba skills install                  # default: user scope -> ~/.claude/skills
naba skills install --dry-run        # show what would be written, change nothing
naba skills install --scope project  # install into <git-root>/.claude/skills
naba skills install --surface agents # install into ~/.agents/skills (agents surface)
naba skills install --target DIR     # install into an explicit directory
naba skills upgrade                  # rewrite from the embedded tree, pruning stale files
naba skills remove                   # remove the naba skill again
naba skills status                   # report up-to-date / complete / unmodified
naba skills preflight --json         # fast, offline skill-gate (see below)
```

Every verb accepts `--json` for a machine-readable `{status, data}` envelope, plus the shared
`--scope`, `--surface`, `--target`, and `--dry-run` flags.

Run **`naba skills upgrade`** after any `naba` upgrade (a `self update` already does this unless
you pass `--binary-only`) so the installed skill always matches the binary.

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
