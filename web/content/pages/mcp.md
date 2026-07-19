Title: mcp
Slug: mcp
Subtitle: naba as a Model Context Protocol server

`naba mcp` starts a stdio-based [Model Context Protocol](https://modelcontextprotocol.io)
server that exposes naba's image pipeline as MCP **tools**, and its embedded skill tree as
lazily-loaded MCP **resources**, to assistants like Claude Desktop and Cursor. It drives the
same provider/selector/output pipeline the CLI uses — no generation logic is reimplemented.

## Claude Desktop configuration

Add naba to your `claude_desktop_config.json`:

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
resolution: `NABA_OUTPUT_DIR` env > config `default_output_dir` > the XDG default
`~/.local/share/naba/images`. Setting `NABA_OUTPUT_DIR` is recommended so you know where
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

Beyond the generated `file://` image links, the server exposes naba's embedded skill tree as
MCP **resources** under a `skill://` URI scheme — the lazy-loading pattern:

- **`resources/list`** enumerates the embedded skill tree cheaply. Per embedded skill `<name>`
  it emits a compact `skill://<name>` index resource plus one `skill://<name>/<rel>` resource
  per file (`SKILL.md`, `commands/*.md`, `README.md`). Listing carries **URIs and metadata
  only — never file bodies** — so discovery is cheap.
- **`resources/read`** serves content on demand:
  - `skill://<name>/<rel>` returns that embedded file as text (MIME by extension —
    `text/markdown` for `.md`).
  - `skill://<name>` returns a generated markdown index listing every file's read URI.
  - `file://<path>` returns a generated image by path (the reserved `file:///{path}` resource
    template).

A client discovers skills up front and pulls full instruction content only when it needs it,
so tool schemas and the resource listing both stay lean.

## Related

- [Skills page](/skills/) — the same skill tree as a Claude Code slash command.
- [config](/config/) — provider keys and `default_output_dir`.
