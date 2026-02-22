# Plan: Add `naba mcp` Subcommand

**Status:** Completed

## Context

naba is a CLI for AI image generation via Gemini API. It already has 7 generation commands (generate, edit, restore, icon, pattern, story, diagram). This change adds an `mcp` subcommand that starts a stdio-based MCP server, exposing all 7 capabilities as MCP tools — allowing AI assistants (Claude Desktop, etc.) to use naba directly.

## SDK Choice

**`github.com/mark3labs/mcp-go`** — builder-style tool definitions, explicit `mcp.NewImageContent()` helper for returning images, lighter dependency footprint than the official SDK. Widely adopted, stable API.

## New Files

### 1. `internal/mcp/server.go` — Server setup + tool registration

- `Serve(version string) error` — creates MCP server, registers 7 tools, calls `ServeStdio`
- `registerTools(s)` — registers all tool definitions + handlers
- Import aliases: `mcpsdk` for `mcp-go/mcp`, `mcpserver` for `mcp-go/server`

### 2. `internal/mcp/tools.go` — Tool definitions + handlers

**7 tool definition functions** (each returns `mcpsdk.Tool` with typed params):

| Tool | Required Params | Optional Params |
|------|----------------|-----------------|
| `generate_image` | prompt | style (enum), variations (array), count (1-8), seed |
| `edit_image` | prompt, file | — |
| `restore_image` | file | prompt |
| `generate_icon` | prompt | sizes (int array), style, background, corners, format |
| `generate_pattern` | prompt | style, colors, density, size, repeat |
| `generate_story` | prompt | steps (2-8), style, transition, layout |
| `generate_diagram` | prompt | type, style, layout, complexity, colors |

**7 handler functions** — each follows the same pattern:
1. Parse args from `req.GetArguments()`
2. `resolveClient()` → calls `config.ResolveAPIKey()` + `config.Load()` + `gemini.NewClient()`
3. Call the matching `gemini.Enrich*Prompt()` function (exact signatures from `internal/gemini/prompt.go`)
4. Call `client.Generate()` or `client.GenerateWithImage()`
5. Write via `output.WriteImage()` → returns absolute path
6. Return `CallToolResult` with text content (path) + image content (base64 data)

**Helper functions**: `resolveClient`, `getStringArg`, `getIntArg`, `getStringSliceArg`, `getIntSliceArg`, `imageResult`, `multiImageResult`

### 3. `internal/cli/mcp.go` — Cobra subcommand (thin)

```go
var mcpCmd = &cobra.Command{
    Use:   "mcp",
    Short: "Start MCP server for AI tool integration",
    Args:  cobra.NoArgs,
    RunE:  func(cmd *cobra.Command, args []string) error {
        return mcpserver.Serve(Version)  // Version from version.go ldflags
    },
}
```

No flags — MCP servers are configured by the client.

### 4. `internal/mcp/tools_test.go` — Tests

Following project conventions (stdlib only, httptest, t.TempDir, t.Setenv):

- **Arg validation**: missing prompt, missing file, nonexistent file, invalid steps range
- **Auth errors**: all 7 handlers with no API key set
- **Success paths**: mock Gemini server, verify result content (text path + image data)
- **Multi-output**: generate with count, story with steps, icon with sizes — verify correct number of API calls and results
- **Helper unit tests**: getStringArg, getIntArg, getStringSliceArg, getIntSliceArg

## Modified Files

- `go.mod` / `go.sum` — add `github.com/mark3labs/mcp-go` dependency
- `CLAUDE.md` — add `mcp` to command list in Architecture section

## Dependencies on Existing Code

- `internal/gemini/prompt.go` — all 7 `Enrich*Prompt` functions (reused directly, not duplicated)
- `internal/gemini/client.go` — `NewClient`, `Generate`, `GenerateWithImage`
- `internal/output/writer.go` — `WriteImage` for file output
- `internal/config/auth.go` — `ResolveAPIKey`
- `internal/config/config.go` — `Load` for model resolution
- `internal/cli/version.go` — `Version` var (passed to MCP server)

## Implementation Order

1. `go get github.com/mark3labs/mcp-go@latest`
2. `internal/mcp/server.go`
3. `internal/mcp/tools.go`
4. `internal/cli/mcp.go`
5. `internal/mcp/tools_test.go`
6. Update `CLAUDE.md`

## Verification

```bash
go build ./...                          # compiles cleanly
go test ./... -count=1                  # all tests pass (existing + new)
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | go run ./cmd/naba mcp
                                        # returns initialize response with 7 tools
```
