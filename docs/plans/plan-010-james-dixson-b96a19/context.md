# Project Environment Context

_Snapshot taken at plan-authoring time. Cold readers: verify these values
against the current environment before acting. The snapshot header below
records the machine and date of capture._

## Project environment

**naba** is a Rust CLI (formerly a Go tool, ported in plan-004) for AI image generation and
editing (generate/edit/restore/icon/pattern/diagram/story), plus a Claude-Code skill
(`skills/naba`, `/naba <subcommand>`) and an MCP server (`naba mcp`). Stack: Rust (clap CLI,
`build.rs` minijinja skill render, `include_dir!` binary-embedded skill tree), a Python parity
test suite under `tests/parity/` (run via `uv run`, PEP-723 scripts), and a static website under
`web/`. The image capabilities call an external provider (Gemini / OpenRouter / Bedrock) gated by
an API key. The UX contract is pinned by `SPEC-*` clauses in `docs/specifications/` and enforced
by `tests/parity/check_traceability.py`; artifact agreement is enforced by `DRIFT-CHECK.md`. This
plan touches only the enum/param inventory in `src/enums.rs` (new), `src/mcp.rs`, `src/cli.rs`,
`skills/naba/commands/*.md`, and a new Rust golden test — no provider/network work.

## Tool inventory

<!-- snapshot: host=d3-mbp-m5.local date=2026-07-21 -->

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
- Plan directory: `docs/plans/plan-010-james-dixson-b96a19`

## Operator identity

- Git user: `james-dixson` (James Dixson)
- Role: sole maintainer / author of naba; full authority to approve, land, and push in this repo.
- Contact / attribution: James Dixson (GitHub `dixson3`), per `~/.claude/CLAUDE.md`.

## Runtime assumptions

- **OS/shell:** macOS (darwin), zsh. A recent stable Rust toolchain (`cargo build`/`cargo test`)
  and `uv` for the Python parity suite.
- **No network / no credentials required** to execute this plan: it is a pure source refactor +
  Rust golden test. The image provider API key is *not* exercised (no image generation runs).
- **Side effects:** edits within the naba repo only (`src/`, `skills/naba/commands/*.md`, a new
  test). No external services, no cross-repo work. Landing follows the repo default (trunk-based
  merge to `main`); push is operator-authorized per the conservative push policy.
- A cold reader on another machine needs only Rust + `uv` + the repo checkout; the plan is safe to
  run offline.

## Adjacent-concept glossary

_Optional._ Terms, acronyms, or project-specific jargon the plan uses.

## Additional context

_Optional._ Anything else a cold reader needs that does not fit above.
