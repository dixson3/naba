# Plan: Overlay Claude Plugin on Naba Repo

## Context

The naba repo is a Go CLI for AI image generation. It already has an MCP server exposing 8 tools. The user wants to add a Claude Code plugin **inline at the repo root** so the same repo hosts both the Go source and a full Claude plugin (skills, agents, rules, hooks). The repo will be registered as a single-plugin marketplace (like `dixson3/code-manager`).

Go tooling (`go build`, `go test`, `golangci-lint`) ignores non-`.go` directories, so plugin files coexist without conflict.

## New files to create (20 files, no existing files modified except `.gitignore`)

```
.claude-plugin/
  plugin.json                # Plugin manifest (name, version, author, hooks)
  marketplace.json           # Single-plugin marketplace descriptor
  preflight.json             # Install-time rule symlinks

skills/
  generate_image/SKILL.md    # Image generation workflow
  edit_image/SKILL.md         # Image editing workflow
  restore_image/SKILL.md      # Image restoration workflow
  generate_icon/SKILL.md      # Icon generation workflow
  generate_pattern/SKILL.md   # Pattern generation workflow
  generate_story/SKILL.md     # Story sequence workflow
  generate_diagram/SKILL.md   # Diagram generation workflow
  browse_images/SKILL.md      # Image browsing/management
  brand_kit/SKILL.md          # Composite: icon + pattern + variations
  storyboard/SKILL.md         # Composite: story + iterative editing

agents/
  naba_image_assistant.md     # Routes requests to correct MCP tool
  naba_batch_processor.md     # Batch generation orchestration

rules/
  naba-image-prompts.md       # Prompt engineering guidance
  naba-tool-routing.md        # Tool selection decision tree

hooks/
  session-start.sh            # SessionStart: verify naba + API key
  pre-naba-check.sh           # PreToolUse: validate before naba calls
```

## Implementation order

### 1. `.claude-plugin/` manifest files

**`plugin.json`** — core manifest with:
- name: `naba`, version: `1.0.0`
- author: James Dixson, license: MIT
- hooks: `SessionStart` -> `hooks/session-start.sh`, `PreToolUse` (matcher `Bash(naba *)`) -> `hooks/pre-naba-check.sh`

**`marketplace.json`** — single-plugin marketplace pointing `source: "./"` at the repo root

**`preflight.json`** — symlinks `rules/naba-image-prompts.md` and `rules/naba-tool-routing.md` into `.claude/rules/naba/`

### 2. `rules/` — prompt engineering + tool routing

Two rule files providing Claude guidance:
- **naba-image-prompts.md**: Best practices for crafting prompts per tool (composition, lighting, style references, anti-patterns)
- **naba-tool-routing.md**: Decision tree for selecting the right MCP tool based on user intent

### 3. `hooks/` — session lifecycle scripts

- **session-start.sh**: Checks `naba` on PATH and `GEMINI_API_KEY` availability, emits status JSON
- **pre-naba-check.sh**: Guards `naba` CLI invocations, exits non-zero if env is broken
- Both need `chmod +x`

### 4. `skills/` — 10 skills (8 tool-mapping + 2 composite)

Each skill wraps an MCP tool with workflow guidance (prerequisites check, parameter recommendations, iteration suggestions). Skill parameters match the actual MCP tool definitions in `internal/mcp/tools.go`:

| Skill | MCP Tool | Key Parameters |
|-------|----------|----------------|
| generate_image | generate_image | prompt, style (10 enum), variations (7 enum), count (1-8), seed |
| edit_image | edit_image | prompt, file |
| restore_image | restore_image | file, prompt (optional) |
| generate_icon | generate_icon | prompt, sizes[], style (4 enum), background, corners, format |
| generate_pattern | generate_pattern | prompt, style (5 enum), colors (3 enum), density (3 enum), size, repeat |
| generate_story | generate_story | prompt, steps (2-8), style, transition, layout |
| generate_diagram | generate_diagram | prompt, type (7 enum), style (4 enum), layout (4 enum), complexity (3 enum), colors (3 enum) |
| browse_images | list_images | limit |
| brand_kit | composite | Orchestrates generate_icon + generate_pattern + generate_image |
| storyboard | composite | Orchestrates generate_story + edit_image |

### 5. `agents/` — 2 agents

- **naba_image_assistant.md**: General routing agent — maps user intent to the correct MCP tool, includes prompt engineering guidance
- **naba_batch_processor.md**: Batch orchestration — icon suites, asset collections, documentation diagrams

### 6. Update `.gitignore`

Add:
```
# Plugin runtime artifacts (symlinked by preflight)
.claude/rules/naba/
```

## What does NOT change

- No Go source modifications
- No CI workflow changes (`.github/workflows/`)
- No Makefile changes
- No `.goreleaser.yaml` changes
- `go build ./...` and `go test ./... -count=1` unaffected

## Verification

1. `go build ./...` — confirm Go build still works
2. `go test ./... -count=1` — confirm all tests pass
3. `golangci-lint run ./...` — confirm linter unaffected
4. Verify all hook scripts are executable: `ls -la hooks/*.sh`
5. Validate `plugin.json` is well-formed JSON: `python3 -m json.tool .claude-plugin/plugin.json`
6. Verify marketplace discovery: `.claude-plugin/marketplace.json` exists at repo root with `"source": "./"`
