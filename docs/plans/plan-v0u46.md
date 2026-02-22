# Plan: MCP Claude Desktop Image Handling

## Context

The naba MCP server currently returns full base64-encoded image data inline in tool results. Claude Desktop enforces a ~1MB limit on MCP tool responses, so any non-trivial image (or multi-image response) gets rejected. Images are already being written to disk via `NABA_OUTPUT_DIR`, but the response still includes the heavy base64 payload alongside the path.

The goal is to make naba's MCP mode work well in Claude Desktop: return file paths (and optionally resource links) so Claude can reference images without hitting the size limit, while keeping the inline base64 path available for small single images.

## Changes

### 1. Replace inline base64 with file path + resource link in `internal/mcp/server.go`

**Replace `imageResult()`** — instead of returning `[TextContent(path), ImageContent(base64)]`, return `[TextContent(path), ResourceLink(file://path)]`:

```go
func imageResult(path string, data []byte, mimeType string) *mcpsdk.CallToolResult {
    return &mcpsdk.CallToolResult{
        Content: []mcpsdk.Content{
            mcpsdk.NewTextContent(path),
            mcpsdk.NewResourceLink(
                "file://"+path,
                filepath.Base(path),
                "Generated image",
                mimeType,
            ),
        },
    }
}
```

**Replace `multiImageResult()`** — same pattern per image, no base64:

```go
func multiImageResult(paths []string, images []gemini.ImageResult) *mcpsdk.CallToolResult {
    var content []mcpsdk.Content
    for i, img := range images {
        content = append(content,
            mcpsdk.NewTextContent(paths[i]),
            mcpsdk.NewResourceLink(
                "file://"+paths[i],
                filepath.Base(paths[i]),
                fmt.Sprintf("Generated image %d", i+1),
                img.MIMEType,
            ),
        )
    }
    return &mcpsdk.CallToolResult{Content: content}
}
```

This eliminates the `encoding/base64` import from server.go entirely.

### 2. Require `NABA_OUTPUT_DIR` for MCP mode

MCP handlers must write images to a known directory (not CWD which is undefined in Claude Desktop context). Add a check at the top of `resolveClient()` or add a new `resolveOutputDir()` helper that errors when the output dir is empty:

In each top-level handler entry point (`generateAndReturn`, `generateWithImageAndReturn`, `handleGenerateImage`, `handleGenerateIcon`, `handleGenerateStory`), the `outDir` resolution already exists. Add a fallback: if `NABA_OUTPUT_DIR` is unset and config has no `default_output_dir`, default to `~/.local/share/naba/images` (XDG data convention) rather than CWD, so MCP mode always has a predictable output directory.

Add to `internal/config/auth.go`:

```go
func DefaultOutputDir() string {
    home, err := os.UserHomeDir()
    if err != nil {
        return ""
    }
    return filepath.Join(home, ".local", "share", "naba", "images")
}
```

Update `ResolveOutputDir()` to use this as final fallback when called from MCP context — or simply update the MCP handlers to fill in the default when `config.ResolveOutputDir()` returns empty.

### 3. Register a resource template for generated images

Add a `file://` resource template so Claude Desktop can discover and browse generated images via the MCP resources API.

In `internal/mcp/server.go`, after `registerTools(s)`, add:

```go
func registerResources(s *mcpserver.MCPServer) {
    s.AddResourceTemplate(
        mcpsdk.NewResourceTemplate(
            "file:///{path}",
            "Generated image file",
            mcpsdk.WithTemplateDescription("Access a generated image by its file path"),
            mcpsdk.WithTemplateMIMEType("image/*"),
        ),
        handleReadResource,
    )
}
```

With a handler that reads the file from disk:

```go
func handleReadResource(ctx context.Context, req mcpsdk.ReadResourceRequest) ([]mcpsdk.ResourceContents, error) {
    path := strings.TrimPrefix(req.Params.URI, "file://")
    data, err := os.ReadFile(path)
    if err != nil {
        return nil, fmt.Errorf("read image: %w", err)
    }
    mime := mimeFromExt(filepath.Ext(path))
    encoded := base64.StdEncoding.EncodeToString(data)
    return []mcpsdk.ResourceContents{
        mcpsdk.BlobResourceContents{
            URI:      req.Params.URI,
            MIMEType: mime,
            Blob:     encoded,
        },
    }, nil
}
```

### 4. Add `list_images` tool for browsing output directory

Add a lightweight tool that lists recently generated images in the output directory, so Claude can discover and reference them for edit/restore operations:

In `internal/mcp/tools.go`, add:
```go
func listImagesTool() mcpsdk.Tool {
    return mcpsdk.NewTool("list_images",
        mcpsdk.WithDescription("List recently generated images in the output directory"),
        mcpsdk.WithNumber("limit",
            mcpsdk.Description("Maximum number of images to return"),
            mcpsdk.DefaultNumber(20),
        ),
    )
}
```

Handler in `internal/mcp/server.go` — reads the output dir, sorts by modification time (newest first), returns paths:
```go
func handleListImages(_ context.Context, req mcpsdk.CallToolRequest) (*mcpsdk.CallToolResult, error) {
    outDir := resolveOutputDirWithDefault()
    limit := req.GetInt("limit", 20)
    // glob for naba-* image files, sort by mtime desc, return up to limit paths
    ...
}
```

This makes it easy for Claude to say "edit the most recent image" or "restore image #3".

### 5. Update README MCP config example

Update the Claude Desktop config example in `README.md` to include the recommended `NABA_OUTPUT_DIR` setting:

```json
{
  "mcpServers": {
    "naba": {
      "command": "naba",
      "args": ["mcp"],
      "env": {
        "GEMINI_API_KEY": "<your-key>",
        "NABA_OUTPUT_DIR": "~/.local/share/naba/images"
      }
    }
  }
}
```

### 6. Tests

**`internal/mcp/tools_test.go`**:
- Update existing success tests: assert result content contains `ResourceLink` type (not `ImageContent`)
- Assert no `ImageContent` in responses (base64 removed)
- Add `TestListImages_Success` — populate output dir, invoke, verify file list returned
- Add `TestListImages_EmptyDir` — empty dir returns empty list, not error

**`internal/mcp/tools_test.go`** (resource handler):
- `TestReadResource_Success` — write a file, invoke handler with `file://` URI, verify blob returned
- `TestReadResource_NotFound` — invoke with nonexistent path, verify error

**`internal/config/config_test.go`**:
- `TestDefaultOutputDir` — returns `~/.local/share/naba/images`

## Files modified

| File | Change |
|---|---|
| `internal/mcp/server.go` | Replace base64 with ResourceLink in results; add `resolveOutputDirWithDefault()`; add resource template handler; add `handleListImages`; add `registerResources()` |
| `internal/mcp/tools.go` | Add `listImagesTool()` definition |
| `internal/config/auth.go` | Add `DefaultOutputDir()` |
| `internal/mcp/tools_test.go` | Update success tests for ResourceLink; add list_images + resource handler tests |
| `internal/config/config_test.go` | Add DefaultOutputDir test |
| `README.md` | Update MCP config example with recommended `NABA_OUTPUT_DIR` |
| `CLAUDE.md` | Note MCP-mode default output dir behavior |

## Verification

```bash
go build ./...
go test ./... -count=1
```

Manual test with Claude Desktop:
1. Set `NABA_OUTPUT_DIR=/tmp/naba-test` in the MCP config
2. Ask Claude to generate an image — should succeed without 1MB error
3. Ask Claude to list recent images — should see the generated file
4. Ask Claude to edit the generated image — should find it by path
5. Verify images exist in `/tmp/naba-test/`
