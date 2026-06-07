# Project Environment Context

_Snapshot taken at plan-authoring time. Cold readers: verify these values
against the current environment before acting. The snapshot header below
records the machine and date of capture._

## Project environment

`naba` is a standalone Go CLI (+ stdio MCP server) for AI image generation via
Google's Gemini "nanobanana" model. Stack: Go (cobra CLI, `mark3labs/mcp-go`,
`yaml.v3`); standard-library-only tests with `httptest`. Layout: `cmd/naba`
(entry), `internal/{cli,mcp,gemini,output,config}`. Alongside the Go app the repo
ships Claude-facing assets — 9 skills under `skills/` and (currently) a Claude Code
plugin under `.claude-plugin/` plus two top-level `agents/`. This plan modernizes
**only those Claude-facing assets and the beads config**; it does not change the Go
application code. The reference packaging pattern being adopted lives in the sibling
repo `/Users/james/workspace/dixson3/beads-skills` (frontmatter-driven `install.py`).
Issue tracking is beads (`bd`) on a local-only Dolt DB; upstream is GitHub Issues on
`dixson3/naba`.

## Tool inventory

<!-- snapshot: host=d3-mbp-m5.local date=2026-06-07 -->

- `bd`: bd version 1.0.5 (Homebrew)
- `git`: git version 2.50.1 (Apple Git-155)
- `uv`: uv 0.11.19 (7b2cff1c3 2026-06-03 aarch64-apple-darwin)
- `python`: Python 3.14.2
- `gh`: gh version 2.93.0 (2026-05-27)
- `glab`: glab 1.102.0 (b5a548b3)
- `claude`: 2.1.168 (Claude Code)

## Paths

- Repo root: `/Users/james/workspace/dixson3/naba`
- Working directory at plan creation: `/Users/james/workspace/dixson3/naba`
- Plan directory: `docs/plans/plan-001-james-dixson-a9545f`

## Operator identity

- Git user: `james-dixson`
- Operator: James Dixson (Yoshiko Studios LLC), dixson3@gmail.com, GitHub `dixson3`.
- Role: sole maintainer/owner of this repo.
- Authority scope: full authority over repo contents. Git authority is **conservative
  by default** — the executing agent reports the land-the-plane sequence (commit / `bd
  dolt push` is N/A since local-only / `git push`) and runs it only on explicit operator
  authorization. Attribution for new code/LICENSE: MIT, James Dixson / Yoshiko Studios
  LLC, current year.

## Runtime assumptions

- OS/shell: macOS (darwin 25.5.0), zsh. Paths and `git mv`/`rsync` usage assume a
  POSIX environment; the installer targets `~/.claude/skills` by default.
- Tools on PATH (all verified present at plan time): `naba`, `uv`, `gh`, `bd`,
  `git`, `rsync`, `d2`, `python` 3.14.
- Credentials: `gh` is authenticated as `dixson3` (active token) — required for the
  upstream-config sanity check (Epic 4). No Gemini API key is needed for this plan
  (no image generation is exercised).
- Network: needed only for the `gh`/`beads-upstream` dry-run check; the rest is local
  file/DB work.
- Side effects: the plan moves/deletes files (`skills/` rename, delete `.claude-plugin/`
  and `agents/`), mutates the local beads DB/config, and installs skills to the user's
  `~/.claude` (verification uses a throwaway `--target`, not the real user dir). It does
  **not** add a Dolt remote or push to one (local-only by decision). Git commits/pushes
  are conservative — operator-authorized only.
- bd health precondition: the schema migration that was wedged at session start has been
  resolved (DB at v49); a cold reader on a fresh clone should re-run `bd doctor` first.

## Adjacent-concept glossary

_Optional._ Terms, acronyms, or project-specific jargon the plan uses.

## Additional context

_Optional._ Anything else a cold reader needs that does not fit above.
