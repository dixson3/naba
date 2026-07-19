# Finding 001 — Landscape & ground truth

> **SCOPE UPDATE (supersedes the `/downloads/` mirror language below).** After this
> investigation, the operator narrowed scope: the website hosts **only** a bootstrap
> `install.sh` (mirror of cargo-dist's `naba-installer.sh`). There is **NO `/downloads/`
> binary mirror and NO `latest.json`** on the domain — GitHub release downloads are
> CDN-served and effectively unlimited, so a mirror buys little. Wherever this finding
> says "`/downloads/` mirror" or "`latest.json`", read it as **superseded**; the authoritative
> scope is `plan.md`.

Direct-read investigation (no worktree agents needed); the mechanics are well-understood
from the reference project and naba's own source.

## AWS / DNS state (account REDACTED-ACCOUNT-ID, user `dixson3`)

- AWS CLI credentials work (`aws sts get-caller-identity` succeeds).
- `ysapp.net` is already a Route53 **public hosted zone**: the Route53 zone id (redacted — stored in local .envrc + GitHub repo secret `NABA_HOSTED_ZONE_ID`, not in the repo).
  A `naba.ysapp.net` subdomain record is a single upsert — no domain registration or
  zone delegation needed.

## Reference Pelican pipeline (`~/workspace/ys/thesoftwarefactory`)

Proven pattern to mirror:

- `pelican==4.11.0` + `pelican-sitemap`, `markdown`, `PyYAML` (`requirements.txt`).
- `pelicanconf.py` (dev) + `publishconf.py` (prod, sets `SITEURL`, `RELATIVE_URLS=False`,
  `DELETE_OUTPUT_DIRECTORY=True`).
- `Makefile` targets: `html`, `serve`, `devserver` (`pelican -lr`), `publish`,
  `s3_upload` (`aws s3 sync OUTPUT s3://BUCKET --delete` + `aws cloudfront
  create-invalidation --paths "/*"`), `validate` (isolated build), staging variants.
- Bucket named after the domain; CloudFront distribution id passed via `CF_DISTRIBUTION`.
- Theme is self-contained under `themes/<name>/{templates,static/css}` with
  `base.html`, `index.html`, `page.html`, partials; content is Markdown under `content/`.

## naba self-update — already GitHub-Releases-canonical (do NOT change)

- `src/self_cmd/update.rs`: `manifest_url()` is hardcoded to
  `{CARGO_PKG_REPOSITORY}/releases/latest/download/dist-manifest.json`
  (i.e. `github.com/dixson3/naba/...`). `asset_url()` likewise points at GitHub Releases.
- The updater reads cargo-dist's `dist-manifest.json`, selects the `executable-zip`
  artifact whose `target_triples` contains the host triple, verifies the `.sha256`
  sidecar, and `self_replace`-swaps.
- **Decision (operator):** the website's `/downloads/` + `latest.json` are a
  human-facing mirror; the updater stays on GitHub Releases. Zero Rust changes.

## cargo-dist config (`Cargo.toml [workspace.metadata.dist]`)

- `cargo-dist-version = 0.32.0`, `installers = ["shell", "homebrew"]`,
  `tap = dixson3/homebrew-tap`, `checksum = "sha256"`, `install-path = ~/.local/bin`,
  `unix-archive = ".tar.gz"`.
- Targets: `aarch64-apple-darwin`, `x86_64-apple-darwin`,
  `aarch64-unknown-linux-gnu`, `x86_64-unknown-linux-gnu`.
- cargo-dist default asset naming: `naba-<target-triple>.tar.gz` (+ `.sha256`),
  plus `dist-manifest.json` and the shell installer `naba-installer.sh`.

## Critical dependency: first cargo-dist release is NOT cut yet (issue #7)

- Current GitHub releases (`v0.5.0` and earlier) are **Go-era** artifacts
  (`naba_darwin_amd64.tar.gz` naming) — NOT cargo-dist.
- The cargo-dist artifacts (`naba-<triple>.tar.gz`, `dist-manifest.json`,
  `naba-installer.sh`) only exist once issue **#7** cuts the first `v<semver>` tag.
- **Implication:** the `/downloads/` mirror and the hosted `install.sh` are populated
  FROM a cargo-dist release. This plan builds the tooling + the site and can go live,
  but the mirror/bootstrap only carry real bytes after #7 lands. The mirror script and
  the `/downloads/` page must tolerate "no cargo-dist release yet" gracefully.

## Bootstrap install (operator clarification)

- The domain hosts `install.sh` at the site root = a mirror of cargo-dist's
  `naba-installer.sh`. Bootstrap:
  `curl --proto '=https' --tlsv1.2 -LsSf https://naba.ysapp.net/install.sh | sh`.
- That installer downloads the binary tarball from **GitHub Releases** (cargo-dist
  default), lands it in `~/.local/bin`, writes the receipt; thereafter
  `naba self update` runs off GitHub Releases. Domain = friendly entrypoint; GitHub =
  canonical bytes.
