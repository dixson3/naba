Title: mcp
Slug: mcp
Subtitle: naba as a Model Context Protocol server

`naba mcp` starts a stdio-based [Model Context Protocol](https://modelcontextprotocol.io)
server that exposes `naba`'s image pipeline as MCP **tools**, and its embedded skill tree as
lazily-loaded MCP **resources**, to assistants like Claude Desktop and Cursor. It drives the
same provider/selector/output pipeline the CLI uses — no generation logic is reimplemented.

## Why run naba as an MCP server?

MCP is the way to give a **desktop** assistant — one that can't run shell commands — first-class
access to `naba`. The assistant calls `naba`'s image tools directly over the protocol and gets
structured results back, no terminal involved.

That makes it the counterpart to the [agent harness skill](/skills/), and it helps to know which
one you want:

- **The skill** ([skills page](/skills/)) is for **coding agents that already have a shell** —
  Claude Code, opencode, and friends. When triggered, the agent *shells out to the `naba` CLI*.
  The skill is essentially instructions plus a command to run.
- **The MCP server** is for **assistants without a shell** — Claude Desktop, Cursor. The
  assistant calls `naba`'s tools *over the MCP protocol*; `naba` runs as a long-lived server process
  it talks to.

Both drive the identical provider/selector/output pipeline, so you get the same images either
way — the difference is purely how the assistant reaches `naba`. You can run both at once (a skill
in your coding agent, an MCP server in your desktop app) from the same binary.

## Claude Desktop configuration

Add `naba` to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "naba": {
      "command": "naba",
      "args": ["mcp"],
      "env": {
        "GEMINI_API_KEY": "<your-key>",
        "OPENROUTER_API_KEY": "<your-key>",
        "AWS_BEARER_TOKEN_BEDROCK": "<your-token>",
        "NABA_OUTPUT_DIR": "/path/to/output"
      }
    }
  }
}
```

Only the key(s) for the provider(s) you use are required. Verify the server responds to the
MCP initialize handshake:

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"0.1.0"}}}' | naba mcp
```

## `NABA_OUTPUT_DIR`

MCP tools do **not** use the CLI's `-o`/CWD path. They write images through the MCP output-dir
resolution, in this order (highest precedence first):

1. the `NABA_OUTPUT_DIR` environment variable
2. the `default_output_dir` config key
3. the XDG default (`~/.local/share/naba/images`)

Setting `NABA_OUTPUT_DIR` is recommended so you know where
generated images land. Errors surface as tool-level error results (`isError: true`), never a
process exit.

## Tools

The server registers exactly **8 tools**:

| Tool | Required params | Optional params |
|:-----|:----------------|:----------------|
| `generate_image` | `prompt` | `style`, `variations[]`, `count` (1–8), `seed`, `aspect`, `resolution`, `quality` |
| `edit_image` | `prompt`, `file` | `aspect`, `resolution`, `quality` |
| `restore_image` | `file` | `prompt`, `aspect`, `resolution`, `quality` |
| `generate_icon` | `prompt` | `sizes[]`, `style`, `background`, `corners`, `format`, `quality` |
| `generate_pattern` | `prompt` | `style`, `colors`, `density`, `size`, `repeat`, `aspect`, `resolution`, `quality` |
| `generate_story` | `prompt` | `steps` (2–8), `style`, `transition`, `layout`, `aspect`, `resolution`, `quality` |
| `generate_diagram` | `prompt` | `type`, `style`, `layout`, `complexity`, `colors`, `aspect`, `resolution`, `quality` |
| `list_images` | — | `limit` (default 20) |

The generative tools accept the shared `aspect`, `resolution`, and `quality` params matching the
CLI; `generate_icon` takes `quality` but no `aspect`/`resolution` (icon `sizes` are canvas
pixels, a separate concept). With `count`/`steps`/`sizes` > 1 the same imageConfig applies to
every image in the call. `list_images` is MCP-only — it lists recently generated `naba-*`
images in the output directory, newest first.

## Lazy-loading skills as resources

Beyond the generated `file://` image links, the server exposes **MCP-authored usage guidance**
for its tools as MCP **resources** under a `skill://` URI scheme — the lazy-loading pattern.

The important part: this guidance is written *for the MCP tools*, not borrowed from the CLI. `naba`
renders its skill source into two variants — the `/naba` slash-command skill the
[skills page](/skills/) deploys, and a separate **MCP render** served here. The MCP render
describes calling `generate_image`, `edit_image`, and friends by **tool name and parameters**,
with `NABA_OUTPUT_DIR` output resolution and `file://` result links — and **no `/naba` slash
commands and no `--flags`**, because none of those exist in an MCP session. So a desktop assistant
fetches guidance it can actually act on, instead of instructions about a shell it doesn't have.

- **`resources/list`** enumerates the MCP render cheaply. Per embedded skill `<name>` it emits a
  compact `skill://<name>` index resource plus one `skill://<name>/<rel>` resource per file — the
  MCP guide (`SKILL.md`) plus any per-tool notes (`mcp/*.md`). The CLI command docs
  (`commands/*.md`) and the skill `README.md` are **not** served here. Listing carries **URIs and
  metadata only — never file bodies** — so discovery is cheap.
- **`resources/read`** serves content on demand:
  - `skill://<name>/<rel>` returns that embedded file as text (MIME by extension —
    `text/markdown` for `.md`).
  - `skill://<name>` returns a generated markdown index listing every file's read URI.
  - `file://<path>` returns a generated image by path (the reserved `file:///{path}` resource
    template).

Each generation tool's `description` also carries a one-line pointer to `skill://naba`, so the
assistant knows the guidance exists and can fetch it on demand. Always-loaded context stays
minimal — the assistant discovers skills up front and pulls the full instruction content only when
it needs it, so tool schemas and the resource listing both stay lean.

## Related

- [skills page](/skills/) — the same skill tree as an in-agent `/naba` slash command (for shell-capable coding agents).
- [config](/config/) — provider keys and `default_output_dir`.
