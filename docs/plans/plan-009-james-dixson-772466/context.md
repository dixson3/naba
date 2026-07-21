# Project Environment Context

_Snapshot taken at plan-authoring time. Cold readers: verify these values
against the current environment before acting. The snapshot header below
records the machine and date of capture._

## Project environment

**naba** is a single-binary **Rust** CLI (`src/*.rs`, `Cargo.toml`) for AI image generation
(generate/edit/restore/icon/pattern/diagram/story + composites) over multiple providers
(Gemini, OpenRouter, Bedrock). It ships a binary-embedded **skill** (`skills/naba/`, rendered
at build time into `cli/`+`mcp/` trees) installed via `naba skills install`, and an **MCP
server** (`naba mcp`) exposing its capabilities as tools + `skill://` resources. Its behavioral
contract lives in **`docs/specifications/*.md`** (split per-domain: skills, mcp, json-output,
commands, configuration, distribution, README index), pinned by a parity suite
(`tests/parity/`, `check_traceability.py`) and a `DRIFT-CHECK.md` manifest. This plan is
**documentation-only**: it authors a new tool-agnostic `docs/specifications/agent-tools.md` and
wires minimal index/drift references — **no Rust/build changes**.

A second repo is in scope read-mostly: **`~/workspace/dixson3/yoshiko-flow`** — the `yf` kernel
+ `yf-*` skills toolchain whose `yf skills` was reverse-engineered from naba. This plan reads it
for reconnaissance and lands **one doc-only cross-reference pointer** there (separate git repo,
separate push authority).

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
- Plan directory: `docs/plans/plan-009-james-dixson-772466`

## Operator identity

- Git user: `james-dixson` (James Dixson), sole operator/maintainer of both the `naba` and
  `yoshiko-flow` repos.
- Attribution: MIT, James Dixson (GitHub `dixson3`). On the `byid-mba-dixson3` machine the
  org/email is Beyond Identity / `james.dixson@beyondidentity.com`; otherwise Yoshiko Studios
  LLC / `dixson3@gmail.com`. This machine is `d3-mbp-m5.local` (Yoshiko Studios context).
- Authority scope: the operator authorizes commits/pushes to both repos; naba pushes go to
  `github.com:dixson3/naba`; the yoshiko-flow pointer push is a separate operator-authorized
  action in that repo.

## Runtime assumptions

- **OS/shell:** macOS (Darwin, Apple Silicon — `d3-mbp-m5.local`), `zsh`.
- **Toolchain:** `bd` ≥ 1.1.0 (beads, dolt `local-only`), `git`, `uv` (for the parity
  Python + markdown-lint), `gh` (GitHub, authed). No Rust build is required by this plan
  (doc-only), though `cargo`/`check_traceability.py` are used to confirm nothing regresses.
- **Repos:** naba at `/Users/james/workspace/dixson3/naba`; yoshiko-flow at
  `~/workspace/dixson3/yoshiko-flow` (present, branch `main`). Both are the operator's.
- **Network:** required only for `gh` (#12 reconcile) and the git pushes; the authoring +
  local validation are offline.
- **Side effects:** edits naba `docs/specifications/` + `DRIFT-CHECK.md` + README index (naba
  repo), and lands **one doc-only pointer commit** in the **separate yoshiko-flow repo**. No
  provider/API calls, no production deploy, no code/build changes. The cross-repo edit is the
  only non-naba side effect and is operator-gated (Cross-repo landing gate).

## Adjacent-concept glossary

_Optional._ Terms, acronyms, or project-specific jargon the plan uses.

## Additional context

_Optional._ Anything else a cold reader needs that does not fit above.
