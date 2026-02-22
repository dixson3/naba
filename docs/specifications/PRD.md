# Product Requirements Document (PRD)

## 1. Purpose & Goals

naba is a standalone command-line interface for AI image generation powered by the Google Gemini API. It provides the same capabilities as the nanobanana MCP server -- text-to-image generation, image editing, restoration, icon generation, pattern creation, visual story sequences, and technical diagram generation -- without requiring an MCP host.

The CLI follows established conventions from tools like `gh` and `kubectl`: structured subcommands, `--json` output for machine consumption, LLM-friendly piped output, and environment-variable-based authentication.

### Goals

- Provide a fast, cross-platform binary for AI image generation from the terminal
- Support 7 distinct image generation modes via subcommands
- Enable scriptable automation with structured JSON output and semantic exit codes
- Deliver actionable error messages that guide users toward resolution
- Maintain minimal dependencies (stdlib HTTP client, cobra for CLI, yaml.v3 for config)

## 2. Technical Constraints

- **Language:** Go (fast builds, trivial cross-compilation)
- **CLI framework:** github.com/spf13/cobra v1.10.2
- **Config format:** YAML via gopkg.in/yaml.v3
- **HTTP client:** net/http (stdlib only, no external HTTP dependencies)
- **API:** Google Gemini generateContent endpoint (v1beta)
- **Testing:** Go standard library only -- no testify, no gomock
- **Build:** goreleaser for cross-platform releases (darwin/linux, amd64/arm64)
- **Distribution:** Homebrew tap (dixson3/tap/naba), `go install`, source build
- **License:** MIT, Copyright 2026 James Dixson, Yoshiko Studios LLC

## 3. Requirement Traceability Matrix

| ID | Requirement | Priority | Source | Status | Code Reference |
|----|-------------|----------|--------|--------|----------------|
| REQ-001 | All 7 generation subcommands implemented (generate, edit, restore, icon, pattern, story, diagram) | P0 | Plan-01 Completion Criteria | Complete | `internal/cli/generate.go`, `edit.go`, `restore.go`, `icon.go`, `pattern.go`, `story.go`, `diagram.go` |
| REQ-002 | Config get/set works with ~/.config/naba/config.yaml | P0 | Plan-01 Completion Criteria | Complete | `internal/cli/config.go`, `internal/config/config.go` |
| REQ-003 | GEMINI_API_KEY authentication resolves from env var or config file | P0 | Plan-01 Completion Criteria | Complete | `internal/config/auth.go` |
| REQ-004 | --json output on all commands with absolute paths | P0 | Plan-01 Completion Criteria | Complete | `internal/output/json.go`, `internal/cli/root.go` (auto-detect piped stdout) |
| REQ-005 | --output / -o works for file targeting on all commands | P0 | Plan-01 Completion Criteria | Complete | `internal/output/writer.go` |
| REQ-006 | Image input works for edit and restore commands | P0 | Plan-01 Completion Criteria | Complete | `internal/gemini/client.go` (GenerateWithImage) |
| REQ-007 | Semantic exit codes (0, 1, 2, 3, 4, 5, 10) | P0 | Plan-01 Completion Criteria | Complete | `internal/gemini/client.go` (constants), `internal/cli/generate.go` (exitCodeError type) |
| REQ-008 | Actionable error messages with fix suggestions | P1 | Plan-01 Completion Criteria | Complete | `internal/gemini/client.go` (parseAPIError) |
| REQ-009 | Unit tests for client, prompt, output, config, CLI modules | P0 | Plan-02 Completion Criteria | Complete | All 4 testable packages have test files; 81 tests pass |
| REQ-010 | go build produces working binary with version ldflags | P0 | Plan-01 Completion Criteria | Complete | `Makefile`, `.goreleaser.yaml` |
| REQ-011 | MIT LICENSE with Yoshiko Studios LLC attribution | P0 | Plan-01 Completion Criteria | Complete | `LICENSE` |
| REQ-012 | GEMINI_BASE_URL env var override for testability | P0 | Plan-02 Phase 1 | Complete | `internal/gemini/client.go` (NewClient) |
| REQ-013 | All 5 packages have test files | P0 | Plan-02 Completion Criteria | Partial | `cmd/naba` has no test file (noted as acceptable -- thin main.go) |
| REQ-014 | Homebrew distribution via dixson3/tap | P1 | README | Complete | `.goreleaser.yaml` (brews section) |
| REQ-015 | Auto-detect piped stdout and enable JSON mode | P1 | Plan-01 Output Behavior | Complete | `internal/cli/root.go` (PersistentPreRun) |
| REQ-016 | MCP server exposes all 7 generation capabilities as tools via stdio | P1 | Plan-gxu5t | In Progress | `internal/mcp/server.go`, `internal/cli/mcp.go` |
| REQ-017 | MCP tool results include both file path (text) and base64 image content | P1 | Plan-gxu5t | In Progress | `internal/mcp/server.go` (imageResult, multiImageResult) |
| REQ-018 | MCP server reuses existing prompt enrichment and Gemini client (no duplication) | P1 | Plan-gxu5t, DD-001, DD-006 | In Progress | `internal/mcp/server.go` |

## 4. Functional Specifications

### FS-001: Text-to-Image Generation (generate command)

Accepts a text prompt and optional style/variation flags. Calls Gemini API with enriched prompt. Supports --count for multiple variations (1-8), --seed for reproducibility, --format for grid/separate output, --preview for system viewer launch. Writes PNG/JPEG to disk with auto-generated or user-specified filename.

Code: `internal/cli/generate.go`

### FS-002: Image Editing (edit command)

Accepts an image file path and edit instruction. Reads image as base64, sends with prompt to Gemini API. Validates input file existence before API call.

Code: `internal/cli/edit.go`

### FS-003: Image Restoration (restore command)

Accepts an image file path and optional enhancement prompt. Uses default restoration prompt when none provided. Input file validated before API call.

Code: `internal/cli/restore.go`

### FS-004: Icon Generation (icon command)

Generates app icons at specified pixel sizes (repeatable --size flag). Supports style (flat/skeuomorphic/minimal/modern), background color, corner style, output format (png/jpeg). Makes one API call per requested size.

Code: `internal/cli/icon.go`

### FS-005: Pattern Generation (pattern command)

Generates seamless patterns/textures. Supports style (geometric/organic/abstract/floral/tech), color scheme, density, tile size, repeat mode (tile/mirror).

Code: `internal/cli/pattern.go`

### FS-006: Story Sequence Generation (story command)

Generates 2-8 sequential frames telling a visual story. Makes one API call per frame with step-aware prompts. Supports visual consistency (consistent/evolving), transition style, layout format. Validates steps range (2-8) with ExitUsage code.

Code: `internal/cli/story.go`

### FS-007: Diagram Generation (diagram command)

Generates technical diagrams. Supports 7 diagram types (flowchart, architecture, network, database, wireframe, mindmap, sequence), 4 visual styles, 4 layouts, 3 complexity levels, 3 color schemes.

Code: `internal/cli/diagram.go`

### FS-008: Configuration Management (config command)

Subcommands: `config get <key>`, `config set <key> <value>`. Persists to `~/.config/naba/config.yaml`. Valid keys: api_key, model, default_output_dir. Creates config directory on first write.

Code: `internal/cli/config.go`, `internal/config/config.go`

### FS-009: Version Display (version command)

Displays version, git commit, and build date. Values injected via ldflags at build time.

Code: `internal/cli/version.go`

### FS-010: MCP Server (mcp command)

Starts a stdio-based Model Context Protocol server exposing all 7 generation capabilities as MCP tools. No flags -- MCP servers are configured by the client. Uses `github.com/mark3labs/mcp-go` SDK. Each tool handler follows the same pattern: parse args, resolve API client, enrich prompt, call Gemini, write output file, return text path + base64 image content. Supports multi-output for generate (count), icon (sizes), and story (steps).

Code: `internal/cli/mcp.go`, `internal/mcp/server.go`
