# naba

A standalone CLI for AI image generation using Google Gemini. Generate, edit, and transform images from the command line.

## Install

### Homebrew (macOS and Linux)

```bash
brew install dixson3/tap/naba
```

### Go

```bash
go install github.com/dixson3/naba/cmd/naba@latest
```

### Build from source

```bash
git clone https://github.com/dixson3/naba.git
cd naba
make build
```

## Setup

Set your Gemini API key:

```bash
export GEMINI_API_KEY=<your-key>
```

Or save it to config:

```bash
naba config set api_key <your-key>
```

## Models & pricing

naba defaults to **`gemini-3.1-flash-image`** (Nano Banana 2) — the current GA image
model, optimized for cost and latency. A higher-quality tier, **`gemini-3-pro-image`**
(Nano Banana Pro), is available for final/hero assets.

> **All Gemini image models require a paid (billing-enabled) tier — none work on the free
> tier.** Pro costs roughly 2–3.5× flash per image, so flash is the default; opt into pro
> only when you need its quality.

Select the model per call or in config:

```bash
naba generate "hero banner" --quality high     # alias: high -> gemini-3-pro-image
naba generate "hero banner" --quality fast      # alias: fast -> gemini-3.1-flash-image (default)
naba generate "hero banner" --model gemini-3-pro-image   # raw model id (highest precedence)
```

Model precedence is `--model` > `--quality` > config `model` > config `quality` > built-in
default. Existing configs (e.g. `model: gemini-2.5-flash-image`) keep working unchanged.

## Usage

### Generate images

```bash
naba generate "a red apple on a white background"
naba generate "mountain landscape" --style watercolor
naba generate "city skyline" -n 4 --style pixel-art
naba generate "abstract art" -v lighting -v color-palette -o art.png
naba generate "wide vista" --aspect 16:9 --resolution 2K
```

**Aspect ratio & resolution.** `--aspect` and `--resolution` set the Gemini
`imageConfig` and are available on all generative commands (`generate`, `edit`, `restore`,
`pattern`, `diagram`, `story`). Valid `--aspect`: `1:1, 1:4, 1:8, 2:3, 3:2, 3:4, 4:1, 4:3,
4:5, 5:4, 8:1, 9:16, 16:9, 21:9`. Valid `--resolution`: `512, 1K, 2K, 4K` (uppercase `K`).
Invalid values are rejected before the API call. `icon --size` is **canvas pixels** — a
separate concept from `imageConfig`'s `imageSize` — and is unchanged.

### Edit images

```bash
naba edit photo.png "make the sky more dramatic"
naba edit portrait.jpg "add a hat" -o portrait-hat.png
```

### Restore/enhance images

```bash
naba restore old-photo.jpg
naba restore blurry.png "sharpen and improve colors"
```

### Generate icons

```bash
naba icon "a music note" --size 64 --size 256 --size 512
naba icon "rocket ship" --style flat --background white --corners sharp
```

### Generate patterns

```bash
naba pattern "tropical leaves" --style floral --colors colorful
naba pattern "circuit board" --style tech --density dense --colors mono
```

### Generate stories

```bash
naba story "a cat's journey through a magical forest" --steps 6
naba story "sunrise to sunset" --steps 4 --transition dramatic
```

### Generate diagrams

```bash
naba diagram "user authentication flow" --type flowchart
naba diagram "microservices architecture" --type architecture --complexity comprehensive
naba diagram "database schema for blog" --type database --style clean
```

### Configuration

```bash
naba config set api_key <key>
naba config set model gemini-3-pro-image    # or use: naba config set quality high
naba config set aspect 16:9                  # default imageConfig aspect ratio
naba config set resolution 2K                # default imageConfig resolution
naba config get model
```

Config keys: `api_key`, `model`, `default_output_dir`, `aspect`, `resolution`, `quality`.
Per-call flags override config; within config, `model` beats `quality`.

### MCP Server

`naba mcp` starts a stdio-based [Model Context Protocol](https://modelcontextprotocol.io) server that exposes all 7 image generation tools to AI assistants like Claude Desktop and Cursor.

**Claude Desktop configuration** — add to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "naba": {
      "command": "naba",
      "args": ["mcp"],
      "env": {
        "GEMINI_API_KEY": "<your-key>",
        "NABA_OUTPUT_DIR": "/path/to/output"
      }
    }
  }
}
```

Setting `NABA_OUTPUT_DIR` is recommended — it tells naba where to write generated images. Without it, images are saved to `~/.local/share/naba/images` by default.

**Available tools:**

| Tool | Description |
|------|-------------|
| `generate_image` | Generate an image from a text prompt |
| `edit_image` | Edit an existing image based on a text prompt |
| `restore_image` | Restore or enhance an existing image |
| `generate_icon` | Generate app icons in multiple sizes |
| `generate_pattern` | Generate seamless patterns and textures |
| `generate_story` | Generate a sequence of images that tell a visual story |
| `generate_diagram` | Generate technical diagrams and flowcharts |
| `list_images` | List recently generated images in the output directory |

The generative tools accept `aspect`, `resolution`, and `quality` params (matching the
CLI); `generate_icon` accepts `quality`. With `count`/`steps`/`sizes` > 1 the same
`imageConfig` applies to every image in the call.

**Manual test** — verify the server responds to the MCP initialize handshake:

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"0.1.0"}}}' | naba mcp
```

## Claude Code Skills

naba ships a single [Claude Code](https://claude.com/claude-code) skill that wraps the CLI
as one slash command with subcommands: `/naba <subcommand>` (e.g. `/naba generate`,
`/naba edit`, `/naba icon`, …). The skill files are **embedded in the `naba` binary** and
installed with `naba skills install` (offline, version-matched) — there is no marketplace
plugin and no separate installer script.

> **Prerequisite:** the skill shells out to the `naba` CLI, so the **`naba` binary must be
> installed and on PATH** (see [Install](#install)) and `GEMINI_API_KEY` set (see
> [Setup](#setup)). `naba skills install` always writes the skill files; the skill is inert
> until the binary is on PATH.

> **Breaking change (plan-002 → plan-003):** the previous 10 separate skills
> (`/naba-generate`, `/naba-edit`, …) were consolidated into one `/naba <subcommand>` skill,
> and the old shell/python installer scripts were replaced by `naba skills`. Those old
> per-command skills are **not** embedded in the binary, so `naba skills` cannot remove
> them. If you installed them under a prior version, delete them manually first:
>
> ```bash
> rm -rf ~/.claude/skills/naba-*   # user/claude scope; adjust path for --surface agents or project scope
> ```
>
> Then install the consolidated skill with `naba skills install`.

### Install the skill

```bash
naba skills install                  # default: user scope -> ~/.claude/skills
naba skills install --dry-run        # show what would be written, change nothing
naba skills install --scope project  # install into <git-root>/.claude/skills instead
naba skills install --surface agents # install into ~/.agents/skills (agents surface)
naba skills install --target DIR     # install into an explicit directory
naba skills upgrade                  # rewrite from the embedded tree, pruning stale files
naba skills remove                   # remove the naba skill again
naba skills status                   # report up-to-date / complete / unmodified
```

The skill tree is embedded via `go:embed`, so `naba skills` works offline and always
matches the binary's version. On `install`/`upgrade` it writes a hidden integrity marker
into the deployed `SKILL.md` (`<!-- naba-skills: v=<version> tree=<sha256> -->`); `status`
and `naba doctor` use that marker to confirm the install is current, complete, and
unmodified. The repository source `skills/naba/SKILL.md` stays marker-free.

### Health check

`naba doctor` validates your environment and exits non-zero if any check fails:

```bash
naba doctor                  # checks skills install, API key, model, config
naba doctor --json           # structured output
naba doctor --surface agents # check the agents-surface install instead
```

It reports: skills installed and matching this binary (integrity marker present,
up-to-date, complete, unmodified); `GEMINI_API_KEY` present; the key live-valid (a cheap
`models.list` call, no image cost); the configured model reachable; config parseable; and
the binary version.

### Subcommands

Invoke as `/naba <subcommand> [args]`; run `/naba help` to print the dispatch table.

| Subcommand | Purpose |
|------------|---------|
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
returns a compact summary. Shared prompt guidance, anti-patterns, and the global-flags table
live once in `skills/naba/SKILL.md`.

## Global Flags

| Flag | Description |
|------|-------------|
| `--json` | Output structured JSON (auto-enabled when piped) |
| `-o, --output` | Output file path |
| `-q, --quiet` | Suppress progress output |
| `-m, --model` | Override Gemini model |
| `--no-input` | Disable interactive prompts |

The generative commands also accept `--aspect`, `--resolution` (imageConfig), and
`--quality` (`fast`/`high` model alias); see [Models & pricing](#models--pricing) and the
[Generate images](#generate-images) section.

## Output format

The Gemini image API returns **JPEG**. naba writes the file with the matching extension:
if your `-o` path uses a different extension (e.g. `-o hero.png`), naba corrects it on disk
(`hero.jpg`), warns on stderr, and reports both formats in JSON (`requested_format` /
`actual_format`) so you can decide whether a post-generation conversion is needed.

## JSON Output

When `--json` is used (or stdout is piped), output includes:

```json
{
  "path": "/absolute/path/to/image.jpg",
  "command": "generate",
  "prompt": "a red apple",
  "elapsed_ms": 3200,
  "params": {
    "style": "watercolor",
    "aspect": "16:9",
    "resolution": "2K"
  },
  "requested_format": "png",
  "actual_format": "jpeg"
}
```

`requested_format` appears only when `-o` implied a format; `actual_format` reflects the
response mimeType. They differ when the extension was corrected.

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Usage error |
| 3 | Authentication error |
| 4 | Rate limit exceeded |
| 5 | API error |
| 10 | File I/O error |

## License

MIT - see [LICENSE](LICENSE)
