---
type: Environment
okf_spec: OKF-PLAN
---
# Project Environment Context

_Snapshot taken at plan-authoring time. Cold readers: verify these values
against the current environment before acting. The snapshot header below
records the machine and date of capture._

## Project environment

`naba` is a Rust CLI for AI image generation/editing (Gemini + OpenRouter providers) that also
ships and manages embedded Agent Skills. The binary embeds skill trees and deploys them to harness
skill dirs (`~/.claude/skills`, `.agents/skills`, etc.) via `naba skills install|upgrade`, with a
self-update path (cargo-dist vendor installer). Releases are cut by cargo-dist. A companion Pelican
static site under `web/` documents the tool. Task tracking is local-only beads (`bd`) mirrored to
GitHub Issues on `dixson3/naba`. Build: `cargo build`/`cargo test`. This plan touches user-facing
docs (`README.md`, `CONTRIBUTING.md`, `web/content/**`), release tooling (`CHANGELOG.md`,
`AGENTS.md` release lockstep), and the skill-install Rust code (`src/skills.rs`,
`src/skills_install.rs`, `src/embed.rs`, `src/self_cmd/update.rs`).

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
- Plan directory: `docs/plans/plan-012-james-dixson-a3cd13`

## Operator identity

- Git user: `james-dixson` (James Dixson, GitHub `dixson3`)
- Contact: james@yoshikostudios.com — Yoshiko Studios LLC
- Authority scope: repo owner/maintainer of `dixson3/naba`; full authority to land, release, and
  push. New code attributed MIT © 2026 James Dixson.

## Runtime assumptions

- macOS (darwin, Apple Silicon), `zsh`. Rust toolchain (`cargo`) present; Pelican available in the
  `web/` environment for the doc build.
- No network or credentials required to *execute* the plan itself: #16 (docs) and #17 (CHANGELOG)
  are file edits; #18 is Rust code + unit tests. Verifying #17's cargo-dist behavior may use
  `parse-changelog` locally (no release published).
- Side effects: Epic 3's runtime GC does `remove_dir_all` on skill dirs, but that runs at
  *product* runtime (`skills upgrade`), not during plan execution; tests use temp dirs.
- GitHub access (`gh`, authenticated) is needed only at reconcile to close #16/#17/#18.

## Adjacent-concept glossary

_Optional._ Terms, acronyms, or project-specific jargon the plan uses.

## Additional context

_Optional._ Anything else a cold reader needs that does not fit above.
