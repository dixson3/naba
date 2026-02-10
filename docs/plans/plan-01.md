# Plan 01: nba CLI — Standalone Nanobanana Image Generation Tool

**Status:** Draft
**Date:** 2026-02-09

## Overview

The nanobanana MCP server wraps the Google Gemini `generateContent` API for image generation, editing, icons, patterns, stories, and diagrams. We want a standalone CLI tool (`nba`) that provides the same capabilities without requiring an MCP host. The CLI should follow `gh`/`bd` conventions: structured subcommands, `--json` output, LLM-friendly design, and env-var auth.

**Key insight from research:** Nanobanana is a thin prompt-engineering layer. All 7 "tools" call the same Gemini endpoint — the only variation is prompt enrichment and whether an input image is included. The CLI can be built as a straightforward HTTP client with prompt templates.

## Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Language | Go | Fast builds, trivial cross-compilation, cobra ecosystem |
| Binary name | `nba` | Short, matches repo name |
| Auth envvar | `GEMINI_API_KEY` | Compatible with other Gemini tools |
| Repo | This repo (`/Users/james/workspace/dixson3/nba`) | Single context |
| API client | `net/http` (stdlib) | No external dependency for HTTP |
| CLI framework | `cobra` | De facto standard, used by gh/kubectl/docker |

## Command Structure

```
nba generate <prompt>          Text-to-image generation
nba edit <file> <prompt>       Edit existing image with instructions
nba restore <file> [prompt]    Restore/enhance an image
nba icon <prompt>              Generate app icons
nba pattern <prompt>           Generate seamless patterns/textures
nba story <prompt>             Generate sequential image series
nba diagram <prompt>           Generate technical diagrams
nba config [get|set] <key>     Manage configuration
nba version                    Show version info
```

### Global Flags

```
--json              Structured JSON output (absolute paths, echoed params)
--output / -o       Output file path or directory
--quiet / -q        Suppress progress to stderr
--model / -m        Override Gemini model (default: gemini-2.5-flash-image)
--no-input          Disable interactive prompts (auto-detected when not TTY)
```

### Command-Specific Flags

**generate:**
```
--style / -s        Art style: photorealistic, watercolor, oil-painting, sketch,
                    pixel-art, anime, vintage, modern, abstract, minimalist
--count / -n        Number of variations (1-8, default: 1)
--seed              Seed for reproducible output
--format            Grid or separate output (default: separate)
--variation / -v    Variation types: lighting, angle, color-palette, composition,
                    mood, season, time-of-day (repeatable flag)
--preview           Auto-open in system viewer
```

**edit / restore:**
```
--preview           Auto-open result in system viewer
```

**icon:**
```
--style             flat, skeuomorphic, minimal, modern (default: modern)
--size              Icon sizes in px (repeatable: --size 64 --size 256 --size 512)
--format            png or jpeg (default: png)
--background        transparent, white, black, or color name
--corners           rounded or sharp (default: rounded)
```

**pattern:**
```
--style             geometric, organic, abstract, floral, tech (default: abstract)
--colors            mono, duotone, colorful (default: colorful)
--density           sparse, medium, dense (default: medium)
--tile-size         Pattern tile size e.g. "256x256" (default: 256x256)
--repeat            tile or mirror (default: tile)
```

**story:**
```
--steps             Number of frames 2-8 (default: 4)
--style             consistent or evolving (default: consistent)
--transition        smooth, dramatic, fade (default: smooth)
--layout            separate, grid, comic (default: separate)
```

**diagram:**
```
--type              flowchart, architecture, network, database, wireframe,
                    mindmap, sequence (default: flowchart)
--style             professional, clean, hand-drawn, technical (default: professional)
--layout            horizontal, vertical, hierarchical, circular (default: hierarchical)
--complexity        simple, detailed, comprehensive (default: detailed)
--colors            mono, accent, categorical (default: accent)
```

### Output Behavior

- **Default (TTY):** Write image to file, print human-friendly summary to stdout, progress to stderr
- **`--json`:** Write image to file, print JSON metadata to stdout (absolute path, dimensions, elapsed_ms, prompt, params)
- **`-o path`:** Write to specified path instead of auto-generated name
- **Piped (no TTY):** Same as `--json` automatically (LLM-friendly)

### Exit Codes

```
0  Success
1  General error
2  Usage error (bad flags/args)
3  Authentication error (GEMINI_API_KEY not set or invalid)
4  Rate limit exceeded
5  API error (Gemini server-side)
10 File I/O error (can't read input or write output)
```

## Project Layout

```
nba/
  cmd/
    nba/
      main.go                 Entry point
  internal/
    cli/
      root.go                 Root cobra command + global flags
      generate.go             generate subcommand
      edit.go                 edit subcommand
      restore.go              restore subcommand
      icon.go                 icon subcommand
      pattern.go              pattern subcommand
      story.go                story subcommand
      diagram.go              diagram subcommand
      config.go               config subcommand
      version.go              version subcommand
    gemini/
      client.go               Gemini API HTTP client (generateContent)
      client_test.go          Client tests with mock server
      prompt.go               Prompt enrichment templates
      prompt_test.go          Prompt template tests
      types.go                Request/response types
    output/
      writer.go               File output handling (naming, dedup, format)
      writer_test.go          Output writer tests
      json.go                 JSON output formatting
      preview.go              System viewer launch (open/xdg-open)
    config/
      config.go               Config file loading (~/.config/nba/config.yaml)
      auth.go                 Auth resolution (envvar > config file)
  go.mod
  go.sum
  LICENSE
  README.md
```

## Implementation Sequence

### Phase 1: Project Scaffold + API Client (foundation)
1. Initialize Go module (`github.com/dixson3/nba`)
2. Add cobra dependency
3. Create `cmd/nba/main.go` entry point
4. Create `internal/cli/root.go` with global flags
5. Create `internal/gemini/types.go` — Gemini API request/response structs
6. Create `internal/gemini/client.go` — HTTP client for `generateContent`
7. Create `internal/config/auth.go` — `GEMINI_API_KEY` resolution
8. Create `internal/cli/version.go`

### Phase 2: Core Generate Command
1. Create `internal/gemini/prompt.go` — prompt enrichment for styles/variations
2. Create `internal/output/writer.go` — file output (naming, dedup, directory creation)
3. Create `internal/output/json.go` — JSON result formatting
4. Create `internal/cli/generate.go` — full generate command with all flags

### Phase 3: Edit + Restore Commands
1. Add base64 image reading to `internal/gemini/client.go`
2. Create `internal/cli/edit.go`
3. Create `internal/cli/restore.go`

### Phase 4: Specialized Commands (icon, pattern, story, diagram)
1. Add prompt templates to `internal/gemini/prompt.go` for each domain
2. Create `internal/cli/icon.go`
3. Create `internal/cli/pattern.go`
4. Create `internal/cli/story.go` (multi-call sequential generation)
5. Create `internal/cli/diagram.go`

### Phase 5: Config + Polish
1. Create `internal/config/config.go` — YAML config file support
2. Create `internal/cli/config.go` — config get/set commands
3. Create `internal/output/preview.go` — system viewer launch
4. Add `LICENSE` (MIT, Yoshiko Studios LLC)
5. Add `README.md` with usage examples

### Phase 6: Tests + Build
1. Unit tests for gemini client (mock HTTP server)
2. Unit tests for prompt enrichment
3. Unit tests for output writer
4. Integration test: `nba generate` end-to-end (requires API key)
5. Build verification: `go build ./cmd/nba`
6. Cross-compile smoke test: `GOOS=linux go build ./cmd/nba`

## API Integration Details

**Endpoint:** `POST https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent`

**Headers:**
```
Content-Type: application/json
x-goog-api-key: <GEMINI_API_KEY>
```

**Text-to-image request body:**
```json
{
  "contents": [{
    "role": "user",
    "parts": [{"text": "<enriched prompt>"}]
  }],
  "generationConfig": {
    "responseModalities": ["TEXT", "IMAGE"]
  }
}
```

**Image edit request body:**
```json
{
  "contents": [{
    "role": "user",
    "parts": [
      {"text": "<prompt>"},
      {"inlineData": {"data": "<base64>", "mimeType": "image/png"}}
    ]
  }],
  "generationConfig": {
    "responseModalities": ["TEXT", "IMAGE"]
  }
}
```

**Response parsing:** Extract `candidates[0].content.parts[].inlineData.data` (base64 image), decode, write to file.

## Completion Criteria

- [ ] All 7 subcommands implemented (generate, edit, restore, icon, pattern, story, diagram)
- [ ] config get/set works with `~/.config/nba/config.yaml`
- [ ] `GEMINI_API_KEY` auth works
- [ ] `--json` output on all commands with absolute paths
- [ ] `--output / -o` works for file targeting
- [ ] Image input works for edit/restore
- [ ] Semantic exit codes (0, 1, 2, 3, 4, 5, 10)
- [ ] Actionable error messages with fix suggestions
- [ ] Unit tests for client, prompt, output modules
- [ ] `go build` produces working binary
- [ ] MIT LICENSE with Yoshiko Studios LLC attribution
