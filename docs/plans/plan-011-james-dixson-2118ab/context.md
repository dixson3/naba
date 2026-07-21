# Project Environment Context

_Snapshot taken at plan-authoring time. Cold readers: verify these values
against the current environment before acting. The snapshot header below
records the machine and date of capture._

## Project environment

**naba** is a single self-contained **Rust** binary CLI for AI image generation across multiple
providers (Google Gemini, OpenRouter, AWS Bedrock). It exposes its image pipeline three ways: the
CLI (`naba generate|edit|restore|icon|pattern|story|diagram`), an **agent skill** (`/naba
<subcommand>`, embedded in the binary and installed by `naba skills install` into a chosen agent
harness), and an **MCP server** (`naba mcp`, exposing the tools + a `skill://` resource surface to
shell-less assistants like Claude Desktop).

Relevant to this plan: `build.rs` renders the single `skills/naba/` source into **two** trees under
`$OUT_DIR` — `cli/` (deployed by `skills install`, hash-pinned) and `mcp/` (served by the MCP
`skill://` resources) — using minijinja `{% if cli %}` / `{% if mcp %}` gates. The project has a
formal SPEC set (`docs/specifications/`), a golden/behavioral parity suite (`tests/parity/`, run
via `uv`), a SPEC↔test traceability check (`make traceability`), an approved `DRIFT-CHECK.md`
manifest, and a Pelican website under `web/`. CI runs `cargo fmt --check`, `cargo clippy -D
warnings`, `cargo test`, and the parity suite.

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
- Plan directory: `docs/plans/plan-011-james-dixson-2118ab`

## Operator identity

- Git user: `james-dixson` (James Dixson)
- Contact / org: james.dixson@beyondidentity.com (Beyond Identity) on the `byid-mba-dixson3`
  machine; otherwise dixson3@gmail.com (Yoshiko Studios LLC).
- Authority scope: repo owner/maintainer of `dixson3/naba`; sole approver for this plan. Remote
  pushes are operator-authorized (conservative push policy).

## Runtime assumptions

- **OS/shell:** macOS (Darwin, `arm64`), zsh. Executes from the repo root at
  `/Users/james/workspace/dixson3/naba`.
- **Toolchain:** a Rust toolchain (`cargo` build/test/fmt/clippy), `uv` for the Python parity
  suite and yf-plan helper scripts, `make` for the parity/traceability/web targets, and a working
  `bd` (beads) for task tracking.
- **This plan is build/test/docs-only** — it edits SPEC docs, `build.rs`, embedded skill source,
  `src/mcp.rs`, parity tests/goldens, `DRIFT-CHECK.md`, and `web/` content. It needs **no network
  and no provider API keys** to validate (build, `cargo test`, parity against the mock provider,
  `make traceability`, `make validate` are all offline/local). No image generation is performed.
- **Side effects:** local commits on a plan branch; remote push is operator-authorized only.

## Adjacent-concept glossary

_Optional._ Terms, acronyms, or project-specific jargon the plan uses.

## Additional context

_Optional._ Anything else a cold reader needs that does not fit above.
