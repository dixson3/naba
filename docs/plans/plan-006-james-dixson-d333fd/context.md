# Project Environment Context

_Snapshot taken at plan-authoring time. Cold readers: verify these values
against the current environment before acting. The snapshot header below
records the machine and date of capture._

## Project environment

**naba** is a standalone CLI (a single Rust binary) for AI image generation across
multiple providers — Google Gemini and OpenRouter. It generates, edits, restores, and
transforms images from the command line. The repo (`github.com/dixson3/naba`, remote
`git@github.com:dixson3/naba.git`) also ships a Claude Code skill (`skills/naba/`) that
wraps the CLI. Distribution is via cargo-dist (GitHub Releases + a `curl | sh` vendor
installer + a Homebrew tap `dixson3/homebrew-tap`), with an in-binary `naba self update`
that reads cargo-dist's `dist-manifest.json` from GitHub Releases.

This plan adds a **new, separate deliverable**: a Pelican static website under `web/`,
published to `naba.ysapp.net` via AWS (S3 + CloudFront + ACM + Route53). The website stack
(Pelican/Python) is independent of the Rust CLI — it does not touch `src/` or the release
pipeline. The pattern mirrors the operator's sibling site
`~/workspace/ys/thesoftwarefactory` (Pelican + Makefile `s3_upload` + CloudFront).
Non-obvious: the first cargo-dist release is NOT cut yet (GitHub issue #7); current
releases are Go-era artifacts, so the hosted `install.sh` mirror only carries real bytes
after #7.

## Tool inventory

<!-- snapshot: host=d3-mbp-m5.local date=2026-07-18 -->

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
- Plan directory: `docs/plans/plan-006-james-dixson-d333fd`

## Operator identity

- Git user: `james-dixson` (James Dixson, GitHub `dixson3`)
- Contact / attribution: `james.dixson@beyondidentity.com` (Beyond Identity) on the
  `byid-mba-dixson3` machine; otherwise `dixson3@gmail.com` (Yoshiko Studios LLC). New
  code/LICENSE attribution defaults to MIT, current year, James Dixson.
- Authority scope: repo owner and sole maintainer; owns the AWS account
  (`REDACTED-ACCOUNT-ID`, IAM user `dixson3`) and the `ysapp.net` Route53 hosted zone. Authorized
  to provision billable AWS infra for `naba.ysapp.net` — gated by the plan's go-live gate.

## Runtime assumptions

- **OS/shell:** macOS (Darwin, `zsh`); the reference site's Makefile/`aws` flow is
  POSIX-portable.
- **Toolchain:** Python 3.14 + `uv` for Pelican (deps pinned in `web/requirements.txt`);
  `pelican` run from `web/`. `gh` authenticated for GitHub Release lookups.
- **AWS credentials:** `aws` CLI configured for account `REDACTED-ACCOUNT-ID` (user `dixson3`)
  with permissions for S3, CloudFront, ACM, and Route53. `aws sts get-caller-identity`
  succeeds. ACM cert MUST be requested in `us-east-1` (CloudFront constraint).
- **DNS:** `ysapp.net` is an existing Route53 public hosted zone
  (`REDACTED-ROUTE53-ZONE-ID`); `naba.ysapp.net` is a single record upsert.
- **Side-effects / permissions:** Epics 1–2 are local-only (no network side effects beyond
  fetching a GitHub Release asset). Epic 3 creates **real, billable** AWS infrastructure
  and is fenced behind the go-live capability gate — nothing billable is created before the
  operator releases it. Network access to GitHub Releases and AWS endpoints is required.
- **Non-interactive shell flags** (`rm -f`, `cp -f`, etc.) per repo AGENTS.md to avoid
  hanging on aliased prompts.

## Adjacent-concept glossary

_Optional._ Terms, acronyms, or project-specific jargon the plan uses.

## Additional context

_Optional._ Anything else a cold reader needs that does not fit above.
