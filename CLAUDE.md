# naba CLI

Standalone CLI for AI image generation via Google Gemini API.

## Build & Test

```bash
go build ./...              # build all packages
go test ./... -count=1      # run all 81 tests
go test ./internal/cli/...  # test CLI commands only
go run ./cmd/naba generate "a red apple"  # run locally
make build                  # build with version ldflags
```

## Architecture

```
cmd/naba/main.go         # entry point, exit code handling
internal/cli/             # cobra commands (root, generate, edit, restore, icon, pattern, story, diagram, config, version)
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

| Variable | Purpose |
|---|---|
| `GEMINI_API_KEY` | API authentication (required for generation commands) |
| `NABA_CONFIG_DIR` | Override config directory (default: `~/.config/naba`) |
| `GEMINI_BASE_URL` | Override API base URL (used by tests) |

## Dependencies

- `github.com/spf13/cobra` — CLI framework
- `gopkg.in/yaml.v3` — config file parsing
