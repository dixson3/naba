# Project Environment Context

_Snapshot taken at plan-authoring time. Cold readers: verify these values
against the current environment before acting. The snapshot header below
records the machine and date of capture._

## Project environment

`naba` is a standalone Go CLI (plus a stdio MCP server) for AI image generation via
Google's Gemini API. The Go module lives under `cmd/naba` (entry point) and `internal/`
(`cli`, `mcp`, `gemini`, `output`, `config`). All commands resolve an API key
(`GEMINI_API_KEY`) → enrich the prompt → call Gemini → write the image → print the result.

This plan is about the **Claude Code skills** layer that wraps the CLI, not the Go code.
Those skills live in `skills/naba-*/` (10 dirs today) and are deployed by a frontmatter-
driven installer (`install.sh` → `install.py`, run via `uv`) into a Claude Code / agent
tree. There is no marketplace plugin. `AGENTS.md` is the single source of truth for project
+ agent guidance; `CLAUDE.md` is a thin pointer to it. `docs/specifications/*` is the
source of truth for product requirements (currently CLI/MCP only — see the plan's
Specifications-gap finding). Issue tracking is **local-only beads** (a Dolt DB with no
remote — never `bd dolt push`); open/deferred beads sync to GitHub Issues (`dixson3/naba`)
via the `beads-upstream` skill. The repo has an approved, enforced `DRIFT-CHECK.md`
manifest and (as of this plan) a `.markdown-lint-on-edit` marker.

## Tool inventory

<!-- snapshot: host=d3-mbp-m5.local date=2026-06-13 -->

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
- Plan directory: `docs/plans/plan-002-james-dixson-a508e7`

## Operator identity

- Git user: `james-dixson`
- Name / contact: James Dixson <dixson3@gmail.com>, Yoshiko Studios LLC (GitHub: dixson3).
- Role: project owner and sole maintainer.
- Authority scope: full authority over this repo. Git is conservative by default — commit/
  push only on explicit instruction. New code/modules attributed MIT © current year to
  James Dixson / Yoshiko Studios LLC.

## Runtime assumptions

- OS/shell: macOS (darwin, Apple Silicon), zsh. `cp`/`mv`/`rm` may be aliased to `-i`; use
  non-interactive flags (`-f`, `-rf`).
- Toolchain on PATH: `bd` ≥ 1.0.5, `git`, `uv`, `python` 3.11+, `gh` (authenticated to
  `dixson3`), `rsync` (required by `install.py`). Go toolchain for any build/test (none
  expected — this plan makes no Go changes).
- The plan is almost entirely doc/skill authoring under `skills/`, `docs/`, `README.md`,
  `AGENTS.md`, `DRIFT-CHECK.md`. The only execution-environment side effects are: Issue 0.1
  may install a throwaway test skill; Issue 3.1 installs to a throwaway `--target` dir and
  runs an end-to-end `naba` call — which needs the `naba` binary on PATH and a valid
  `GEMINI_API_KEY` (network access to the Gemini API) to exercise the composite write path.
- Beads is local-only: never `bd dolt push`. Git: conservative — no commit/push without
  explicit operator authorization.
- Rewriting the approved `DRIFT-CHECK.md` drops it to `approved: no`; the engine is a silent
  no-op until the operator re-approves.

## Adjacent-concept glossary

_Optional._ Terms, acronyms, or project-specific jargon the plan uses.

## Additional context

_Optional._ Anything else a cold reader needs that does not fit above.
