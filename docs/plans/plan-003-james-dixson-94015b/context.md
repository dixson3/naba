# Project Environment Context

_Snapshot taken at plan-authoring time. Cold readers: verify these values
against the current environment before acting. The snapshot header below
records the machine and date of capture._

## Project environment

`naba` is a **Go CLI** (cobra command tree under `internal/cli/*.go`, entrypoint
`cmd/naba/main.go`) that wraps Google's Gemini image-generation API ("Nano Banana"
models) as image subcommands: generate, edit, restore, icon, pattern, diagram, story.
It also ships an **MCP server** (`internal/mcp`) exposing the same operations as tools,
and a set of **Claude Code skills** under `skills/naba/` (a single `/naba <subcommand>`
skill, post plan-002). The Gemini client is **hand-rolled** (`internal/gemini`): explicit
Go request/response structs (`types.go`) + raw `net/http` against
`https://generativelanguage.googleapis.com/v1beta/models/<model>:generateContent` — there is
no Google SDK, so request-shape changes are manual struct edits. Config lives at
`~/.config/naba/config.yaml` (`internal/config`): `api_key`, `model`, `default_output_dir`;
auth resolves `GEMINI_API_KEY` env > config. Build: `make` / `go build ./cmd/naba`. Issue
tracking is **bd (beads)**, local-only Dolt (no dolt remote; `issues.jsonl` is the portable
record), with GitHub upstream (`dixson3/naba`) enabled for issue sync.

## Tool inventory

<!-- snapshot: host=d3-mbp-m5.local date=2026-06-14 -->

- `bd`: bd version 1.0.5 (Homebrew)
- `git`: git version 2.54.0
- `uv`: uv 0.11.21 (5aa65dd7a 2026-06-11 aarch64-apple-darwin)
- `python`: Python 3.14.2
- `gh`: gh version 2.94.0 (2026-06-10)
- `glab`: glab 1.102.0 (b5a548b3)
- `claude`: 2.1.173 (Claude Code)

## Paths

- Repo root: `/Users/james/workspace/dixson3/naba`
- Working directory at plan creation: `/Users/james/workspace/dixson3/naba`
- Plan directory: `docs/plans/plan-003-james-dixson-94015b`

## Operator identity

- Git user: `james-dixson` (James Dixson, Yoshiko Studios LLC; <dixson3@gmail.com>).
- Role: project maintainer / sole author — full authority to approve scope, intake, and
  land (commit/push to `main`).
- Authority scope: conservative git by default; commits/pushes only on explicit operator
  instruction. MIT license, attribution to James Dixson / Yoshiko Studios LLC.

## Runtime assumptions

- **OS/shell:** macOS (darwin, Apple Silicon), zsh. Go toolchain present (`go build`).
- **Network:** outbound HTTPS to `generativelanguage.googleapis.com` required for any live
  generation/smoke test.
- **Credentials:** a **paid-tier** `GEMINI_API_KEY` in the environment. Image models
  (`gemini-3.1-flash-image`, `gemini-3-pro-image`, `gemini-2.5-flash-image`) have **no free
  tier** — the live smoke (Issue 5.2) and the Capability Gate fail without billing enabled.
  Confirmed working on the maintainer's machine 2026-06-14.
- **Side effects / cost:** execution makes **real, paid** Gemini image calls (a few cents
  per image); the smoke test generates a handful of images. Writes generated files to the
  CWD by default (post plan-002 output convention).
- **Safety to run as-is:** safe on the maintainer's machine; a cold reader on a different
  machine needs their own paid `GEMINI_API_KEY` and should expect per-image API charges.

## Adjacent-concept glossary

_Optional._ Terms, acronyms, or project-specific jargon the plan uses.

## Additional context

_Optional._ Anything else a cold reader needs that does not fit above.
