# Implementation Guide: MCP Server

## 1. Overview

The `naba mcp` subcommand starts a stdio-based Model Context Protocol (MCP) server that exposes all 7 image generation capabilities as MCP tools. This allows AI assistants (Claude Desktop, Cursor, etc.) to invoke naba's generation capabilities directly without CLI flag parsing.

The MCP server reuses the same shared pipeline as CLI commands (DD-001, DD-006): resolve API key -> enrich prompt -> call Gemini -> write output -> return result. MCP tool results return the file path, a `Format: <mimeType>` note, and a `ResourceLink` to the file (no inline base64, to stay under MCP client response-size limits); the client opens the linked file to display it.

## 2. Use Cases

| ID | Name | Actor | Preconditions | Flow | Postconditions |
|----|------|-------|---------------|------|----------------|
| UC-016 | Generate image via MCP | MCP client (AI assistant) | GEMINI_API_KEY set or in config; MCP server running | 1. Client sends `generate_image` tool call with prompt 2. Server resolves API key 3. Prompt enriched 4. Gemini API called 5. Image written to disk 6. Result returned with file path + base64 image | Image file on disk; MCP result with text + image content |
| UC-017 | Edit image via MCP | MCP client | API key set; input file exists; MCP server running | 1. Client sends `edit_image` with prompt + file path 2. File existence validated 3. Edit prompt enriched 4. Gemini API called with image 5. Result written and returned | Edited image on disk; MCP result returned |
| UC-018 | Restore image via MCP | MCP client | API key set; input file exists; MCP server running | 1. Client sends `restore_image` with file path and optional prompt 2. Default restoration prompt used if none provided 3. Gemini API called 4. Result returned | Restored image on disk |
| UC-019 | Generate multi-size icons via MCP | MCP client | API key set; MCP server running | 1. Client sends `generate_icon` with prompt and sizes array 2. One API call per size 3. Multiple results returned with all paths and images | One icon file per size; multi-content MCP result |
| UC-020 | Generate story via MCP | MCP client | API key set; MCP server running; steps 2-8 | 1. Client sends `generate_story` with prompt and steps 2. Steps validated 3. One API call per frame 4. All frames returned | N image files; multi-content MCP result |
| UC-021 | Handle MCP auth errors | MCP client | No API key configured | 1. Client sends any tool call 2. resolveClient() finds no key 3. Error returned as tool result (isError: true) 4. Client displays error message | No image generated; error message in tool result |
| UC-022 | Handle MCP arg validation | MCP client | Invalid arguments (missing prompt, bad range) | 1. Client sends tool call with invalid args 2. Handler validates and returns error result | Error in tool result with specific message |

## 3. Implementation Notes

### MCP Handler Pattern

All 7 MCP tool handlers follow this pattern (parallel to CLI's shared pipeline):

```go
func handleToolName(_ context.Context, req mcpsdk.CallToolRequest) (*mcpsdk.CallToolResult, error) {
    // 1. Parse and validate args
    prompt, err := req.RequireString("prompt")
    if err != nil {
        return mcpsdk.NewToolResultError("missing required parameter: prompt"), nil
    }
    optionalParam := req.GetString("param", "default")

    // 2. Resolve model + imageConfig from the shared params (validated client-side)
    model, imgCfg, err := resolveImageParams(req)   // aspect/resolution -> imageConfig; quality -> model
    if err != nil {
        return mcpsdk.NewToolResultError(err.Error()), nil
    }

    // 3. Resolve client with the model override (quality), falling back to config/default
    client, err := resolveClient(model)
    if err != nil {
        return mcpsdk.NewToolResultError(err.Error()), nil
    }

    // 4. Enrich prompt and call the API with the imageConfig
    enriched := gemini.EnrichXxxPrompt(prompt, ...params)
    images, err := client.GenerateWithConfig(enriched, imgCfg)

    // 5. Write to disk; the output path's extension is reconciled to the response mimeType
    outPath := output.OutputPath(outDir, "command", images[0].MIMEType)
    path, err := output.WriteImage(images[0].Data, images[0].MIMEType, outPath, "command", 0)

    // 6. Return text path + a "Format: <mimeType>" note + resource link
    return imageResult(path, images[0].MIMEType), nil
}
```

`resolveClient` takes a model override and `generateAndReturn`/`generateWithImageAndReturn`
take `(model, *gemini.ImageConfig)` — threading the per-call model and imageConfig through
the shared helpers. `icon` uses `resolveQualityModel` (quality/model only, no imageConfig).

### Error Handling Differences from CLI

- CLI returns `exitCodeError` with semantic exit codes (DD-004)
- MCP returns errors as tool results via `mcpsdk.NewToolResultError()` — the MCP protocol has no exit code concept
- Errors are always returned as `(*CallToolResult, nil)` not `(nil, error)` so the MCP client sees them as tool failures, not protocol errors

### Multi-Output Tools

Three tools produce multiple outputs:
- `generate_image` with `count > 1`: N API calls, N images
- `generate_icon` with multiple `sizes`: one API call per size
- `generate_story` with `steps`: one API call per frame

Multi-output results interleave text paths and image content in the `Content` array.

### Tool Parameter Mapping

MCP tools use the same parameter names and defaults as CLI flags. Required params use `req.RequireString()`, optional params use `req.GetString("key", "default")` / `req.GetInt()` / etc.

The six generative tools (`generate_image`, `edit_image`, `restore_image`,
`generate_pattern`, `generate_story`, `generate_diagram`) expose `aspect`, `resolution`,
and `quality` params; `generate_icon` exposes `quality` only (its `sizes` are canvas
pixels, not imageConfig). When a tool emits more than one image (`count`/`steps`/`sizes` >
1), the same `imageConfig` applies to every image. Invalid `aspect`/`resolution`/`quality`
values are rejected as a tool error before any API call.

### Testing

MCP tests follow project conventions (NFR-001, NFR-002):
- Go stdlib only (no testify)
- `httptest.NewServer` for API mocking via `GEMINI_BASE_URL`
- `t.TempDir()` for filesystem isolation
- `t.Setenv()` for env var isolation
- Tests validate arg parsing, auth errors, success paths, and multi-output behavior
