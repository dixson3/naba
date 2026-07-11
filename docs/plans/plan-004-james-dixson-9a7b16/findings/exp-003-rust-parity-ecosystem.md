# INV-3 — Rust parity ecosystem feasibility

**Verdict: FULL Go→Rust parity is realistic with today's crates. No blocked
concern, no weak-Rust-story item.** The single early architectural decision the
plan must make is the **async/blocking seam** (below).

## Two source facts that lower risk

- **"Preview" is NOT terminal image rendering.** `internal/output/preview.go`
  just shells out to `open`/`xdg-open`/`start` (OS default viewer). The Rust
  `open` crate is a 1:1 replacement — `viuer`/terminal rendering is NOT in scope.
- **No image encode/decode.** `WriteImage` writes API-returned bytes straight to
  disk with JPEG-vs-PNG extension reconciliation. Pure `std::fs` + string logic —
  the `image` crate is not required.

## Parity feasibility table

| # | Concern | Rust crate(s) | Version / license | Risk |
|:-:|:--|:--|:--|:--|
| 1 | CLI + subcommands | `clap` (derive) | 4.x, MIT/Apache | LOW |
| 2 | **MCP server SDK** | `rmcp` (feat `server`) + `rmcp-macros` | v2.1/2.2, 2026-07, Apache — official SDK | LOW–MODERATE |
| 3 | Config YAML + migration | `serde_norway` / `yaml_serde` / `serde-yaml-ng` + serde | maintained forks | LOW |
| 4 | HTTP + runtime | `reqwest` (+ `tokio`) | 0.12, MIT/Apache | LOW |
| 5 | Image write + preview | `std::fs` + `open` | open 5.3.5 | LOW |
| 6 | base64 / MIME / exit codes | `base64` + hand-rolled match + `process::exit` | base64 0.22 | trivial |
| 7 | Testing | `assert_cmd`+`predicates`, `insta`/`insta-cmd`, `wiremock`/`httpmock` | current | LOW |

## MCP server parity — verdict: FEASIBLE (not blocked)

`rmcp` is the **official** modelcontextprotocol Rust SDK (12M downloads, 1,280+
reverse deps, weekly releases). It supports everything Go's MCP server uses:
server mode (`ServerHandler` + stdio), tool registration (`#[tool]`/`#[tool_router]`
macros with `schemars`-generated JSON-Schema covering naba's enum/min/max/default/
required constraints — all 8 tools map cleanly), and resource templates
(`list_resource_templates`/`read_resource` for the `file:///{path}` template).

**Recommended pre-commit spike:** port ONE tool (`generate_image` with its
enum/min/max schema) + the `file://` resource template, confirm the
schemars-generated schema matches Go's, before committing all 8.

## The one architectural decision: async vs blocking

`rmcp` is **async-only (tokio)**, while the CLI's natural style is synchronous
(Go is synchronous; blocking `reqwest` would be the close port). You can't call
blocking reqwest inside a tokio handler without `spawn_blocking`. Options:
- **(a) RECOMMENDED — async provider layer** (async `reqwest` + `tokio`), CLI
  drives it via `#[tokio::main]`, MCP handlers await it directly. One shared
  provider code path for CLI + MCP — which also serves the provider-abstraction goal.
- (b) Blocking provider layer for the CLI + `spawn_blocking` in MCP handlers.

Decide (a) up front to avoid a mid-port refactor. This is a design choice, not a
capability gap.

## Crate-selection landmine (YAML)

`serde_yaml` (dtolnay) is retired; the popular-looking **`serde_yml` is UNSOUND +
unmaintained — RUSTSEC-2025-0068 — and must be forbidden in the plan** despite its
download count. Use `serde_norway`, `yaml_serde` (0.10, official YAML org, drop-in
via package rename), or `serde-yaml-ng`. **Auto-migration is nearly free**:
deserialize old file into a struct of `Option<String>` fields (tolerates
absent/unknown keys), add new `provider`/`model` fields, back up original,
re-serialize. No migration crate needed.

## Minor clap notes (no blockers)

- Cobra persistent flags → `#[arg(global = true)]` (`--json`, `-o/--output`,
  `-q/--quiet`, `-m/--model`, `--no-input`, new `--provider`).
- `config get`/`config set` → nested `#[derive(Subcommand)]` enums.
- Go's TTY autodetect → `std::io::IsTerminal` (stable since 1.70).

## Implications for the plan

1. MCP: proceed with `rmcp` (feature `server`); budget the one-tool schema spike.
2. Adopt an **async provider layer** shared by CLI + MCP; CLI via `#[tokio::main]`.
3. YAML: `serde_norway`/`yaml_serde`; **explicitly forbid `serde_yml`**; migration
   is a plain serde round-trip + backup.
4. Preview: `open` crate, not `viuer`.
5. Image IO: `std::fs` only (reproduce extension-reconciliation logic).
6. Testing: `assert_cmd`+`predicates`+`insta`/`insta-cmd` + `wiremock`/`httpmock`.
7. Exit codes → error enum with `exit_code()` → `process::exit` (direct translation
   of Go's `ExitCode()` interface).

Source: `internal/mcp/{server,tools}.go`, `internal/config/config.go`,
`internal/output/{writer,preview}.go`, `internal/cli/root.go`,
`internal/gemini/client.go` (exit constants 17-33), `cmd/naba/main.go`.
