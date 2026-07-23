---
type: Environment
okf_spec: OKF-PLAN
---
# Project Environment Context

_Snapshot taken at plan-authoring time. Cold readers: verify these values
against the current environment before acting. The snapshot header below
records the machine and date of capture._

## Project environment

naba is a single-binary Rust CLI for AI image generation across multiple providers
(Gemini, OpenRouter, AWS Bedrock). This plan concerns the **marketing/docs website** under
`web/` — a **Pelican** static site (Python), built via the `web/Makefile` (`make html`),
themed with a hand-authored theme under `web/themes/` (no build step for CSS/JS), with
homepage hero + feature cards injected from markdown by the local `home_content` Pelican
plugin. Output is generated into `web/output/` and deployed to S3/CloudFront. The sibling
project **yoshiko-flow** (checked out at `~/workspace/dixson3/yoshiko-flow`) provides the
source theme being ported.

## Tool inventory

<!-- snapshot: host=d3-mbp-m5.local date=2026-07-23 -->

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
- Plan directory: `docs/plans/plan-013-james-dixson-338393`

## Operator identity

- Git user: `james-dixson` (James Dixson, GitHub `dixson3`)
- Role: project owner and sole maintainer — full authority to approve, merge, and deploy.
- Contact: james@yoshikostudios.com

## Runtime assumptions

- OS/shell: macOS (`d3-mbp-m5.local`), zsh.
- The sibling **yoshiko-flow** repo is checked out locally at
  `~/workspace/dixson3/yoshiko-flow` — the source of the theme being ported. This is a hard
  machine-local dependency (see "Execution environment assumption" below); the plan is not
  portable to a machine lacking that checkout.
- Build is fully local and offline: `cd web && make html` (Pelican via `uv`/venv in
  `web/.venv`). No network, credentials, or deploy is required to execute or verify this
  plan — deployment (S3/CloudFront) is out of scope.
- Side effects are confined to `web/` (theme files, `pelicanconf.py`, regenerated
  `output/`).

## Adjacent-concept glossary

_Optional._ Terms, acronyms, or project-specific jargon the plan uses.

## Additional context

_Optional._ Anything else a cold reader needs that does not fit above.

## Execution environment assumption (plan-013)

This plan copies the source theme from a **machine-local** checkout:
`~/workspace/dixson3/yoshiko-flow/web/themes/yoshikoflow/`. The execute session MUST run on a
machine where that checkout is present (operator decision, red-team pass-1 concern #1). The
plan is intentionally not portable to a fresh clone on another machine.
