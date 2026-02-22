# Engineering Design Document (EDD) -- Core

## 1. Overview

naba is structured as a four-package Go module behind a thin `cmd/naba/main.go` entry point. The architecture reflects the key insight from Plan-01: nanobanana is a thin prompt-engineering layer where all 7 "tools" call the same Gemini generateContent endpoint. The only variation is prompt enrichment and whether an input image is included.

The design prioritizes:
- Minimal dependency footprint (stdlib HTTP, cobra, yaml.v3)
- Testability via dependency injection through environment variables
- Consistent command patterns (resolve key -> enrich prompt -> call API -> write output -> print result)
- Scriptability via semantic exit codes and auto-detected JSON output

## 2. Non-Functional Requirements

| ID | Category | Requirement | Measure | Source |
|----|----------|-------------|---------|--------|
| NFR-001 | Testability | All packages testable with stdlib only | No external test dependencies in go.mod | Plan-02, CLAUDE.md |
| NFR-002 | Testability | API client testable via mock HTTP servers | GEMINI_BASE_URL env override enables httptest usage | Plan-02 Phase 1 |
| NFR-003 | Reliability | HTTP client timeout prevents hung connections | 120-second timeout on http.Client | `internal/gemini/client.go` |
| NFR-004 | Portability | Cross-platform binary with no CGO dependencies | goreleaser builds for darwin/linux on amd64/arm64 | `.goreleaser.yaml` |
| NFR-005 | Usability | Actionable error messages guide user to fix | Auth errors suggest export/config commands; rate limit errors suggest waiting | `internal/gemini/client.go` parseAPIError |
| NFR-006 | Scriptability | JSON output auto-enabled for piped stdout | TTY detection in PersistentPreRun sets flagJSON | `internal/cli/root.go` |
| NFR-007 | Reliability | File deduplication prevents accidental overwrites | dedup() appends -N suffix up to 999 | `internal/output/writer.go` |
| NFR-008 | Testability | CLI flag state isolation between test cases | resetFlags() function resets all package-level flag vars | `internal/cli/cli_test.go` |
| NFR-009 | Security | API key not logged or included in JSON output | Key passed via header only, not in Result struct | `internal/gemini/client.go`, `internal/output/json.go` |

## 3. Design Decisions

### DD-001: Single Gemini Endpoint for All Commands

**Context:** The nanobanana MCP server exposes 7 tools (generate, edit, restore, icon, pattern, story, diagram), but they all call the same Gemini `generateContent` endpoint. The only differences are: (1) whether an input image is included in the request, and (2) how the user's prompt is enriched with domain-specific instructions.

**Decision:** Implement a single `Client` type with two methods -- `Generate(prompt)` for text-only and `GenerateWithImage(prompt, imagePath)` for image+text. All command-specific behavior lives in prompt enrichment functions in `internal/gemini/prompt.go`.

**Rationale:** Avoids duplicating HTTP client logic across 7 commands. Prompt enrichment is the only variable, and it is a pure string transformation that is trivially testable.

**Consequences:**
- (+) Simple, maintainable API client with only 2 methods
- (+) Prompt enrichment functions are pure and independently testable
- (+) Adding new command types requires only a new enrichment function and CLI wiring
- (-) No per-command request customization (e.g., different generationConfig). All commands use the same `responseModalities: ["TEXT", "IMAGE"]`

### DD-002: Environment Variable Override for API Base URL

**Context:** Plan-02 identified that CLI integration testing requires routing API calls to a mock server. The alternative approaches were: (1) accept base URL as a constructor parameter, requiring refactoring all command files, or (2) use an env var override checked at client construction time.

**Decision:** Add `GEMINI_BASE_URL` env var check in `NewClient()`. When set, it overrides the default `https://generativelanguage.googleapis.com/v1beta` base URL.

**Rationale:** 4-line change (Plan-02 Phase 1) that unlocks full CLI integration testing with `httptest.NewServer`. No refactoring of command layer needed. Env vars are the established pattern for test configuration in this codebase (see GEMINI_API_KEY, NABA_CONFIG_DIR).

**Consequences:**
- (+) CLI tests can use `t.Setenv("GEMINI_BASE_URL", server.URL)` for full end-to-end testing
- (+) Zero impact on production code paths
- (-) Hidden dependency on environment state. Could theoretically be set accidentally in production.

### DD-003: Package-Level Flag Variables with resetFlags() for Testing

**Context:** Cobra stores flag values in package-level variables. When running multiple test cases in the same process, flag values from one test persist into the next.

**Decision:** All CLI flag variables are package-level vars. A `resetFlags()` function in `cli_test.go` resets every flag to its default before each test case.

**Rationale:** This is the standard cobra pattern. The alternative (creating a new root command per test) would require restructuring the init() registration.

**Consequences:**
- (+) Standard cobra idiom, no framework fighting
- (+) Tests are explicit about starting state
- (-) Adding a new flag requires updating resetFlags() -- easy to forget
- (-) Tests must call resetFlags() or risk cross-contamination

### DD-004: exitCodeError Type for Semantic Exit Codes

**Context:** The CLI needs to return specific exit codes (0-10) to callers. Go's cobra framework returns errors from RunE, but standard errors do not carry exit code information.

**Decision:** Define `exitCodeError` struct implementing `Error() string` and `ExitCode() int`. The `main.go` entry point checks for the `ExitCode()` interface via type assertion. All command RunE functions return `exitError(code, msg)` for non-zero exits.

**Rationale:** Clean separation between error semantics (in internal/cli) and process exit (in cmd/naba/main.go). The interface assertion pattern avoids coupling main.go to internal types.

**Consequences:**
- (+) Exit codes are testable without os.Exit
- (+) main.go is 8 lines with no business logic
- (-) Every error path must use exitError() rather than plain fmt.Errorf

### DD-005: Auth Resolution Chain (Env Var > Config File)

**Context:** Users need flexibility in how they provide their API key. Some prefer env vars (for CI/scripts), others prefer persistent config files.

**Decision:** `ResolveAPIKey()` checks `GEMINI_API_KEY` env var first. If empty, loads config file and returns `cfg.APIKey`. Env var always wins.

**Rationale:** Follows the principle of least surprise -- env vars override file config, matching behavior of tools like gh, docker, and kubectl.

**Consequences:**
- (+) Works in both interactive and CI environments
- (+) Users can temporarily override config with env var
- (-) No keyring/keychain integration

### DD-006: Prompt Enrichment as Pure Functions

**Context:** Each command needs to transform the user's prompt into a domain-specific instruction for Gemini. The enrichment logic varies by command type.

**Decision:** Implement enrichment as standalone pure functions in `internal/gemini/prompt.go`: `EnrichGeneratePrompt`, `EnrichEditPrompt`, `EnrichRestorePrompt`, `EnrichIconPrompt`, `EnrichPatternPrompt`, `EnrichStoryPrompt`, `EnrichDiagramPrompt`.

**Rationale:** Pure functions with no side effects are trivially testable. Each function takes primitive parameters and returns a string. No dependency on client state or config.

**Consequences:**
- (+) 100% testable with table-driven tests
- (+) Clear separation: CLI handles flags, prompt.go handles enrichment, client.go handles HTTP
- (-) Enrichment is string concatenation only -- no template engine for complex formatting

## 4. Package Architecture

```
cmd/naba/main.go              Entry point: Execute() -> exitCodeError -> os.Exit(code)
    |
    v
internal/cli/                  Cobra command tree
    root.go                    Root command, global flags, TTY detection
    generate.go                generate subcommand + exitCodeError type + resolveAPIKey + handleAPIError
    edit.go                    edit subcommand
    restore.go                 restore subcommand
    icon.go                    icon subcommand (multi-size loop)
    pattern.go                 pattern subcommand
    story.go                   story subcommand (multi-frame loop)
    diagram.go                 diagram subcommand
    config.go                  config get/set subcommands
    version.go                 version subcommand (ldflags)
    |
    +-- internal/config/       Configuration layer
    |     config.go            YAML load/save, ConfigDir, Get/Set, ValidKeys
    |     auth.go              ResolveAPIKey (env > config)
    |
    +-- internal/gemini/       API client layer
    |     client.go            HTTP client, Generate, GenerateWithImage, error handling
    |     types.go             Request/response structs
    |     prompt.go            7 enrichment functions (pure, no side effects)
    |
    +-- internal/output/       Output layer
          writer.go            WriteImage, filename generation, dedup
          json.go              Result struct, PrintJSON, PrintJSONMulti
          preview.go           System viewer launch (open/xdg-open)
```

## 5. API Integration

**Endpoint:** `POST {baseURL}/models/{model}:generateContent`

**Default model:** `gemini-2.0-flash-exp-image-generation`

**Authentication:** `x-goog-api-key` header

**Request body:** JSON with `contents` (parts array with text and optional inlineData) and `generationConfig` (responseModalities: ["TEXT", "IMAGE"])

**Response parsing:** Extract `candidates[*].content.parts[*].inlineData.data` (base64), decode to bytes, write to file

**Error mapping:**
- HTTP 401/403 -> ExitAuth (3) with auth fix suggestion
- HTTP 429 -> ExitRateLimit (4) with wait suggestion
- HTTP 5xx -> ExitAPI (5) with retry suggestion
- Prompt blocked -> ExitAPI (5)
- No images in response -> ExitAPI (5)
