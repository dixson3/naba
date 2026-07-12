# Project Environment Context

_Snapshot taken at plan-authoring time. Cold readers: verify these values
against the current environment before acting. The snapshot header below
records the machine and date of capture._

## Project environment

naba is a standalone AI image-generation CLI (plus a stdio MCP server) supporting multiple
providers (Google Gemini + OpenRouter). As of plan-004 it is a **Rust** binary (crate `naba`,
`src/`, single-package `Cargo.toml` — no `[workspace]` table). The former **Go** implementation
(`cmd/`, `internal/`, `go.mod`) is retained **only** as the CI parity baseline (`make *-go`
targets, `naba-go` binary) that proves the Go-captured golden outputs still hold; it is not
shipped and a separate follow-on (GitHub issue #5) tracks retiring it. Key async stack: `tokio`
+ `reqwest` (rustls), `clap` (derive), `serde_json`, `sha2`, `include_dir` (skill embedding),
`rmcp` (MCP). Distribution today is a **hand-rolled** `.github/workflows/release.yml` that
cross-compiles 4 targets (`{darwin,linux} × {amd64,arm64}`) and pushes a prebuilt-binary formula
to `dixson3/homebrew-tap` via `HOMEBREW_TAP_TOKEN`. The naba skill is embedded in the binary and
deployed via `naba skills install`. This plan ports yoshiko-flow's self-update + cargo-dist
vendor install and a skills preflight; see `references/yf-reference-report.md`.

## Tool inventory

<!-- snapshot: host=d3-mbp-m5.local date=2026-07-12 -->

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
- Plan directory: `docs/plans/plan-005-james-dixson-55765d`

## Operator identity

- Git user: `james-dixson`
- Attribution: James Dixson (GitHub `dixson3`), sole maintainer/owner of `dixson3/naba` and the
  `dixson3/homebrew-tap`. Authority: full — may author code, cut releases, and manage upstream
  issues. New code/module attribution is MIT, current year, per the user's global convention.

## Runtime assumptions

- **OS/shell:** macOS (darwin, Apple Silicon) with zsh at authoring time; the shipped code must
  also build/run on Linux (the release matrix covers `{darwin,linux} × {amd64,arm64}`). XDG dir
  logic must work on both macOS and Linux (deliberately not `~/Library`).
- **Toolchain:** stable Rust + cargo; `cargo build/test`, `cargo clippy -D warnings`,
  `cargo fmt --check`, and the `tests/parity` (uv/pytest) suite must pass. The Go baseline
  (`go`, `golangci-lint`) is needed only for the parity-baseline targets.
- **cargo-dist (`dist`):** NOT assumed installed — Issue A.3 is gated on it and falls back to
  hand-authoring the workflow (yf precedent). Version to pin: yf used `0.32.0`.
- **Network/credentials:** plan **execution** needs no external image-provider API keys (the
  `self update` pipeline is unit-tested behind a `Fetcher` seam; the preflight auth axis only
  checks key *presence*). It does **not** cut a tagged release — `HOMEBREW_TAP_TOKEN` and a live
  `dist-manifest.json` endpoint are exercised only by the follow-on first-release bead.
- **Side effects:** execution edits repo source + workflows on a branch; it must not push tags,
  publish releases, or mutate the Homebrew tap. `self update` at runtime performs an in-place
  binary swap (`self_replace`) and writes under `~/.config/naba` / `~/.cache/naba`.

## Adjacent-concept glossary

_Optional._ Terms, acronyms, or project-specific jargon the plan uses.

## Additional context

_Optional._ Anything else a cold reader needs that does not fit above.
