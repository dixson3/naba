# naba — Project & Agent Instructions

Standalone CLI (plus stdio MCP server) for AI image generation via multiple providers —
Google Gemini and OpenRouter. This file is the single source of truth for both human and
agent guidance.

## Build & Test

```bash
go build ./...              # build all packages
go test ./... -count=1      # run all tests
go test ./internal/cli/...  # test CLI commands only
go run ./cmd/naba generate "a red apple"  # run locally
make build                  # build with version ldflags
```

## Architecture

```
cmd/naba/main.go         # entry point, exit code handling
internal/cli/             # cobra commands (root, generate, edit, restore, icon, pattern, story, diagram, config, version, mcp)
internal/mcp/             # MCP server, tool definitions, handlers (stdio-based, exposes 8 tools + resource template)
internal/gemini/          # API client, types, prompt enrichment
internal/output/          # file writer, JSON formatter, system preview
internal/config/          # YAML config (~/.config/naba/config.yaml), auth resolution
```

All commands follow: resolve API key -> enrich prompt -> call Gemini -> write output -> print result.

## Key Conventions

- **Go standard library only for tests** — no testify, no gomock
- **httptest.NewServer** for API mocking; `GEMINI_BASE_URL` env var overrides the API base URL
- **t.TempDir()** for filesystem isolation, **t.Setenv()** for env var isolation
- **Package-internal tests** (same package, not `_test` suffix)
- **CLI tests must reset package-level flag vars** between tests — cobra flag state persists across `rootCmd.Execute()` calls. See `resetFlags()` in `internal/cli/cli_test.go`
- **Semantic exit codes**: 0=ok, 1=general, 2=usage, 3=auth, 4=rate-limit, 5=api, 10=file-io
- `exitCodeError` type implements `ExitCode() int` for main.go to extract codes
- `--json` auto-enabled when stdout is piped

## Environment Variables

| Variable             | Purpose                                                       |
|:---------------------|:-------------------------------------------------------------|
| `GEMINI_API_KEY`     | Gemini API auth (env > config `api_key`)                     |
| `OPENROUTER_API_KEY` | OpenRouter API auth (env-only — no config key)               |
| `NABA_CONFIG_DIR`    | Override config directory (default: `~/.config/naba`)        |
| `NABA_OUTPUT_DIR`    | Override output directory for generated images (MCP and CLI) |
| `GEMINI_BASE_URL`    | Override Gemini API base URL (used by tests)                 |
| `OPENROUTER_BASE_URL`| Override OpenRouter API base URL (used by tests)             |

## Providers

naba routes every image command through one of two providers (`gemini` | `openrouter`),
selected by the global `--provider` flag or the `provider` config key. **Agents shelling
out to naba must know:**

- **Resolution precedence:** CLI `--provider` > config `provider` > env-key autodetect
  (only `GEMINI_API_KEY` → gemini; only `OPENROUTER_API_KEY` → openrouter) > gemini fallback.
- **Multi-key reroute (intentional):** both keys set + no configured `provider` → autodetect
  picks **openrouter** (default slug `google/gemini-3.1-flash-image-preview`). To stay on
  Gemini, pin `provider: gemini` in config (config beats autodetect).
- **`--model` requires `--provider`** on the CLI (a bare model is ambiguous → usage error,
  exit 2). Config `model` without config `provider` is allowed.
- **`--quality` is per-provider:** Gemini maps `fast`/`high` to a model tier; OpenRouter
  passes `quality` through as a native request param without swapping the model slug.
  `openrouter/auto` cannot generate images and is rejected early.

**MCP mode**: When no output directory is configured, MCP handlers default to `~/.local/share/naba/images` (not CWD). Tool results return file paths + `ResourceLink` (no inline base64) to stay under Claude Desktop's ~1MB response limit.

## Dependencies

- `github.com/spf13/cobra` — CLI framework
- `gopkg.in/yaml.v3` — config file parsing
- `github.com/mark3labs/mcp-go` — MCP server SDK

## Claude Code Skills

The Claude-facing skill lives in `skills/naba`, is embedded in the binary via `go:embed`
(repo-root package), and is deployed with `naba skills install` (see the README "Claude
Code Skills" section). There is no marketplace plugin and no installer script. It is one
skill invoked as `/naba <subcommand>` (e.g. `/naba generate`): seven
inline subcommands map 1:1 to cobra commands and three composites (`storyboard`, `batch`,
`brand-kit`) dispatch a subagent. Shared guidance lives once in `skills/naba/SKILL.md`;
per-subcommand detail in `skills/naba/commands/*.md`. See
`docs/specifications/IG/skills.md`.

## Specifications

- Always reference `docs/specifications/*` as the source of truth for test plans
- When an implementation plan conflicts existing specifications, ask the operator to confirm the specification change before implementation
- Always persist a copy of the current implementation plan in `docs/plans` using a sequenced/hashed name

```
docs/decisions - important design and implementation decisions from previous sessions
docs/diary - implementation diary
docs/plans - archive of all implementation plans
docs/research - research used in design and implementation
docs/todos - historical todos
docs/specifications - specification collection (source of implementation requirements)
  EDD/ - engineering design document
  IG/  - implementation guides for key subsystems
  PRD.md - the functional/non-functional product requirements
```

## Agent Operating Conventions

Issue tracking uses **beads (`bd`)**; the generic bd workflow conventions live in your
user-scope agent rules and are not duplicated here. naba-specific facts:

- **Local-only beads.** A local Dolt DB with **no remote** — never run `bd dolt push`.
  `.beads/issues.jsonl` is the git-tracked portable record; open/deferred beads sync to
  GitHub Issues (`dixson3/naba`) via the `beads-upstream` skill.

### Non-Interactive Shell Commands

**ALWAYS use non-interactive flags** with file operations to avoid hanging on confirmation prompts (cp/mv/rm may be aliased to `-i`):

```bash
cp -f source dest      # NOT: cp source dest
mv -f source dest      # NOT: mv source dest
rm -rf directory       # NOT: rm -r directory
```

Also: `scp`/`ssh` → `-o BatchMode=yes`; `apt-get` → `-y`; `brew` → `HOMEBREW_NO_AUTO_UPDATE=1`.
