# Project Environment Context

_Snapshot taken at plan-authoring time. Cold readers: verify these values
against the current environment before acting. The snapshot header below
records the machine and date of capture._

## Project environment

**naba** is a single-binary **Rust** CLI for AI image generation across multiple providers
(Google Gemini, OpenRouter, AWS Bedrock) — generate/edit/restore/icon/pattern/story/diagram, plus
`skills`, `mcp`, `self`, `doctor`, `config`, `preflight` command groups. The legacy **Go**
implementation was retired post-cutover (plan-004); the Rust binary is the sole implementation.

Stack & layout: Cargo workspace; entry `src/main.rs`; CLI via clap-derive (`src/cli.rs`); image
pipeline `src/commands.rs`; providers under `src/provider/`; MCP stdio server `src/mcp.rs`;
compile-time skill embed via `include_dir` + tree-hash integrity marker (`src/embed.rs`); skill
install/upgrade/status/remove (`src/skills.rs`); XDG dirs + receipts (`src/dirs.rs`); self-update
(`src/self_cmd/`). The **authoritative spec** is the root `SPEC.md` (§1–§18); this plan splits it
into `docs/specifications/`. A single `/naba` skill is embedded at `skills/naba/` (SKILL.md +
`commands/`). Non-obvious setup: a Python **parity/golden** conformance suite lives at
`tests/parity/` (`uv`-run, `NABA_BIN` defaults to the built Rust binary); a `DRIFT-CHECK.md`
on-edit engine enforces content-agreement edges (skill↔spec, cli-source→web pages) and requires
§0 re-approval after node changes. A separate `web/` Pelican site publishes usage/skills/mcp docs.

Validation: `cargo build`, `cargo test`, `cargo clippy --all-targets -- -D warnings`,
`cargo fmt --check`, and the parity suite (`make parity`) must all pass.

## Tool inventory

<!-- snapshot: host=d3-mbp-m5.local date=2026-07-20 -->

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
- Plan directory: `docs/plans/plan-008-james-dixson-24173a`

## Operator identity

- Git user: `james-dixson` (James Dixson, GitHub `dixson3`).
- Role/authority: repo owner and sole maintainer of `dixson3/naba`; full authority to approve,
  merge, and release. Attribution defaults to MIT / James Dixson.
- Contact: james@yoshikostudios.com (Yoshiko Studios LLC).

## Runtime assumptions

- **OS/shell:** macOS (Darwin, Apple Silicon — `d3-mbp-m5.local`), `zsh`. Rust toolchain + Cargo
  present; `uv` for the Python parity suite; `bd` ≥ 1.1.0 for beads.
- **Harnesses (for the live smoke-test tier, Issue 4.3):** `claude-code`, `opencode`
  (`~/.opencode/bin`), `pi` (pi.dev), and `codex` (OpenAI Codex CLI) are all installed locally and
  were verified runnable headlessly — opencode→Bedrock, pi→OpenRouter, codex→OpenRouter (custom
  `-c model_provider`). CI does **not** have these harnesses; the live tier self-skips on
  `command -v` and the portable path-assertion tests (Issue 1.3) are the CI baseline.
- **Credentials in-env:** `OPENROUTER_API_KEY`, `AWS_PROFILE` + `AWS_REGION` (Bedrock via profile).
  Valid Bedrock + OpenRouter credentials confirmed by the operator. Live smoke-tests make real
  (billable) provider calls; keep them cheap-model + local-only.
- **Side effects:** the plan edits code, specs, and `web/` docs; installs skills into real harness
  dirs during the local smoke-test (idempotent, marker-guarded). No production deploy is in scope.
- **Network:** required for the live smoke-test and any provider call; not required for the CI
  path-assertion tests.

## Adjacent-concept glossary

_Optional._ Terms, acronyms, or project-specific jargon the plan uses.

## Additional context

### Push authorization (operator directive, 2026-07-20)

The operator **pre-authorized the `git push` at plan-execution completion** (yf-plan Phase 6
push handoff / land-the-plane) — *not* at intake. The INTAKE commits are already on **local
`main`, unpushed** (`1502c6e` approved + `f63b161` land-merge); leave them unpushed. When
execution completes and Phase 6 merges the execute branch back to `main` and re-validates the
merged tree, **push then** (`git push`, and `bd dolt push` if a Dolt remote is configured),
bundling the intake commits with the execution commits. This satisfies the conservative
authorized-only push contract — the authorization is recorded here rather than executed early.
