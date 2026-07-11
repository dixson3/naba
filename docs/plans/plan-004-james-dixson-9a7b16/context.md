# Project Environment Context

_Snapshot taken at plan-authoring time. Cold readers: verify these values
against the current environment before acting. The snapshot header below
records the machine and date of capture._

## Project environment

**naba** is a command-line image tool: it generates and transforms images by
calling a hosted generative-image API and writing the returned bytes to disk. It
ships both a CLI (12 command groups: generate/edit/restore/icon/pattern/diagram/
story/config/doctor/skills/mcp/version) and an MCP server exposing the same image
tools. It also ships a Claude Code skill (`/naba`) whose composite subcommands
(storyboard/batch/brand-kit) orchestrate multiple CLI calls.

**Current stack (pre-plan):** Go 1.25.7, module `github.com/dixson3/naba`;
`spf13/cobra` CLI, `gopkg.in/yaml.v3` config, `mark3labs/mcp-go` MCP server,
stdlib `net/http`/`encoding/json`. Deliberately near-zero-dependency. Config is a
flat YAML file at `~/.config/naba/config.yaml` (override dir via `NABA_CONFIG_DIR`).
Auth is `GEMINI_API_KEY` (env) or config; endpoint override via `GEMINI_BASE_URL`.

**This plan** rewrites naba Go→Rust at full parity AND adds a provider abstraction
(Gemini + OpenRouter) with env-key-driven provider/model selection. Non-obvious
setup: the regression suite (Python+pytest) is authored against the CURRENT Go
binary first (golden capture) then replayed against the Rust binary via a
`$NABA_BIN` switch.

## Tool inventory

<!-- snapshot: host=d3-mbp-m5.local date=2026-07-11 -->

- `bd`: bd version 1.1.0 (Homebrew)
- `git`: git version 2.50.1 (Apple Git-155)
- `uv`: uv 0.11.26 (396ef7ce4 2026-06-30 aarch64-apple-darwin)
- `python`: Python 3.14.2
- `gh`: gh version 2.96.0 (2026-07-02)
- `glab`: glab 1.106.0 (fc1869c7)
- `claude`: 2.1.201 (Claude Code)

## Paths

- Repo root: `/Users/james/workspace/dixson3/naba`
- Working directory at plan creation: `/Users/james/workspace/dixson3/naba`
- Plan directory: `docs/plans/plan-004-james-dixson-9a7b16`

## Operator identity

- Git user: `james-dixson` (James Dixson, GitHub `dixson3`).
- Role/authority: project owner and sole maintainer of `dixson3/naba`; has full
  authority to approve the rewrite, the CLI/config contract changes, and cutover.
- Contact: james@yoshikostudios.com. Attribution for new modules/LICENSE: MIT,
  James Dixson (per the repo's convention).

## Runtime assumptions

- **OS/shell:** macOS (Darwin, Apple Silicon), zsh; development on the host in the
  Tool inventory snapshot. The Rust toolchain (stable, ≥1.95 for the MSRV of the
  chosen crates) must be installed for the port; Go 1.25.7 remains available during
  the transition to build the golden-capture binary.
- **Network/credentials:** live image generation needs `GEMINI_API_KEY` and/or
  `OPENROUTER_API_KEY`. The plan is designed so all build/test work runs against
  **mocked HTTP** (via `GEMINI_BASE_URL` / a new `OPENROUTER_BASE_URL`) with NO live
  keys; only the Issue 2.6 live smoke needs a real `OPENROUTER_API_KEY` (gated).
- **Side effects:** the port writes image files to CWD/`--output` (CLI) or
  `NABA_OUTPUT_DIR`/XDG (MCP); `skills install/upgrade/remove` write skill trees
  under `--target`; config auto-migration may back up `~/.config/naba/config.yaml`.
  Execution runs in an isolated git worktree per the yf-plan EXECUTE model.
- **Safe to run as-is** on the maintainer's macOS host with the Rust toolchain
  present; a cold reader on Linux should confirm the Rust toolchain and the `open`
  crate's viewer-launch behavior (preview) but nothing else is platform-locked.

## Adjacent-concept glossary

_Optional._ Terms, acronyms, or project-specific jargon the plan uses.

## Additional context

_Optional._ Anything else a cold reader needs that does not fit above.
