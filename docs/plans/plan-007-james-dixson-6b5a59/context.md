# Project Environment Context

_Snapshot taken at plan-authoring time. Cold readers: verify these values
against the current environment before acting. The snapshot header below
records the machine and date of capture._

## Project environment

**naba** is a single self-contained **Rust** binary — a CLI for AI image generation across
providers (Google Gemini + OpenRouter today; this plan adds AWS Bedrock). It also ships an
embedded Claude Code skill (`/naba <subcommand>`, installed via `naba skills`) and an MCP server
(`naba mcp`). Stack: Rust (clap CLI, `reqwest` HTTP providers, `rmcp` v2.2.0 MCP server,
`serde_norway` YAML config, `include_dir` skill embedding, `wiremock` test mocking). Validation:
`cargo build/test`, `cargo clippy -D warnings`, `cargo fmt --check`, and a Python `tests/parity`
suite (run via `uv`, binary under `NABA_BIN`) plus a SPEC↔test traceability check. The legacy Go
source was retired (Rust-only). A companion Pelican website lives under `web/` (published to
naba.ysapp.net). Key modules: `src/config.rs` (config), `src/provider/` (providers + selection),
`src/mcp.rs` (MCP), `src/skills.rs`+`src/embed.rs` (skills). Specs: root `SPEC.md` (SPEC-xxx ids) +
`docs/specifications/IG/` and `EDD/`.

## Tool inventory

<!-- snapshot: host=d3-mbp-m5.local date=2026-07-19 -->

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
- Plan directory: `docs/plans/plan-007-james-dixson-6b5a59`

## Operator identity

- Git user: `james-dixson` (James Dixson), GitHub `dixson3`.
- Role/authority: repo owner and maintainer of `github.com/dixson3/naba`; sole approver for this
  plan's gates (Start Gate, bedrock-transport). Has AWS account access (for Bedrock testing) and the
  provider API keys.
- Attribution/licensing: MIT, © James Dixson (per repo `LICENSE` / global attribution convention).

## Runtime assumptions

- **OS/shell:** macOS (Apple Silicon, `d3-mbp-m5.local`), `zsh`; direnv loads the repo `.envrc`.
- **Toolchain:** Rust stable (cargo/clippy/fmt), `uv` for the Python parity suite, `bd` ≥ 1.1.0
  (local-only beads, no Dolt remote), `gh` authenticated to `dixson3/naba`.
- **Credentials (execution, mostly test-time):** provider API keys via env (`GEMINI_API_KEY`,
  `OPENROUTER_API_KEY`) already present in `.envrc`; AWS creds/profile + optional
  `AWS_BEARER_TOKEN_BEDROCK` for Bedrock — **unit tests are HTTP-mocked (`wiremock`), so no live
  cloud calls or spend are required** to complete the plan. Secrets are never committed (env +
  `.envrc` + GitHub repo secrets convention, per AGENTS.md).
- **Side effects:** the one irreversible on-disk change is the config-schema auto-migration, guarded
  by `config.yaml.bak`. Code lands via the normal git/parity flow; naba is pre-1.0 with no released
  binary depending on the old schema. No billable infra is created by this plan.
- **Network:** needed for `cargo` fetch (new crates: aws-sigv4/aws-sdk depending on the
  bedrock-transport gate) and for live provider/Bedrock calls if the operator chooses to smoke-test
  beyond the mocked unit tests.

## Adjacent-concept glossary

_Optional._ Terms, acronyms, or project-specific jargon the plan uses.

## Additional context

_Optional._ Anything else a cold reader needs that does not fit above.
