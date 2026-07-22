# naba — Project & Agent Instructions

Standalone CLI (plus stdio MCP server) for AI image generation via multiple providers —
Google Gemini and OpenRouter. This file is the single source of truth for both human and
agent guidance.

## Build & Test

naba is a single **Rust** binary (the legacy Go source was retired post-cutover).

```bash
cargo build --release                       # build the shipped binary (target/release/naba)
cargo test                                  # run all tests
cargo clippy --all-targets -- -D warnings   # lint
cargo fmt --check                           # format check
cargo run -- generate "a red apple"         # run locally
make build                                  # build + copy to ./naba
make parity                                 # parity suite against the Rust binary
```

### Web site (`web/`)

Pelican static site (see `web/README.md`). Use the pinned venv at `web/.venv`.

```bash
# rebuild output/ after any content/theme/plugin change
web/.venv/bin/pelican web/content -o web/output -s web/pelicanconf.py

# serve the built output/ for local preview at http://127.0.0.1:8000/
web/.venv/bin/python3 -m http.server 8000 --bind 127.0.0.1 --directory web/output
```

Prefer this static serve + manual rebuild over `make devserver` (`pelican -lr`): the
autoreload server is single-threaded and wedges after a rebuild. Homepage content is markdown,
not theme HTML: the hero is `web/content/home/hero.md` and the feature cards are
`web/content/cards/*.md`, both read by the `home_content` plugin (`web/plugins/home_content.py`).

naba is a single Rust binary (ported from Go in plan-004). Module layout:

```
src/main.rs        # entry point: parse CLI, TTY autodetect, dispatch, exit-code mapping
src/cli.rs         # clap-derive command surface (all groups incl. `self`, `skills preflight`)
src/commands.rs    # dispatch + image pipeline (generate/edit/restore/icon/pattern/diagram/story)
src/config.rs      # YAML config (~/.config/naba/config.yaml), auth resolution; config_dir is the single source of truth
src/dirs.rs        # XDG dir resolution (config/cache/data/bin) + receipt/marker/update-check paths (SPEC-DIRS)
src/doctor.rs      # full environment health sweep (provider-aware; network-touching)
src/preflight.rs   # `naba skills preflight` fast offline skill-gate (auth/skills/binary axes, SPEC-PREFLIGHT)
src/embed.rs       # compile-time skill embed (include_dir), tree-hash + integrity marker
src/skills.rs      # `naba skills install|upgrade|remove|status`
src/self_cmd/      # `naba self update|install|uninstall`: source, receipt, archive, update, update_check, nag (SPEC-SELF)
src/provider/      # provider layer (gemini, openrouter, selection)
src/mcp.rs         # MCP stdio server
src/output.rs      # file writer, JSON envelopes, system preview
src/version.rs     # build-injected version/commit/date/host-triple (build.rs)
```

All image commands follow: resolve provider + API key -> enrich prompt -> call provider -> write output -> print result.

## Key Conventions

- **Rust standard test harness** (`#[test]` / `#[tokio::test]`); `wiremock` for HTTP mocking. No extra test frameworks.
- **`GEMINI_BASE_URL` / `OPENROUTER_BASE_URL`** env vars override the provider API base URL in tests.
- **Filesystem isolation** via `std::env::temp_dir()` scratch dirs; **env isolation** via a module-local `Mutex` lock (env is process-global).
- **Seams for I/O**: `naba self update` puts the network behind a `Fetcher` trait and the binary swap behind a closure, so the pipeline is unit-tested without a network (SPEC-SELF-004).
- **Semantic exit codes**: 0=ok, 1=general, 2=usage, 3=auth, 4=rate-limit, 5=api, 10=file-io — carried by `error::AppError { code, message }`.
- `--json` auto-enabled when stdout is piped (SPEC-GLOBAL-003).
- **Validation:** `cargo build`, `cargo test`, `cargo clippy -D warnings`, `cargo fmt --check`, and the parity suite (`tests/parity/`) must all pass.

## Writing Voice

When writing or editing **user-facing documentation** — `README.md`, `CONTRIBUTING.md`, the `web/`
site, and similar prose — consult [`VOICE.md`](VOICE.md) first. It covers verbose, human-friendly
exposition; precedence written as explicit ordered lists (never `A > B > C`); and naming the tool
as `` `naba` `` (never bare) in prose.

## Environment Variables

| Variable             | Purpose                                                       |
|:---------------------|:-------------------------------------------------------------|
| `GEMINI_API_KEY`     | Gemini API auth (env > config `api_key`)                     |
| `OPENROUTER_API_KEY` | OpenRouter API auth (env-only — no config key)               |
| `NABA_CONFIG_DIR`    | Override config dir (precedence: `NABA_CONFIG_DIR` > `$XDG_CONFIG_HOME/naba` > `~/.config/naba`) |
| `NABA_OUTPUT_DIR`    | Override output directory for generated images (MCP and CLI) |
| `NABA_NO_UPDATE_CHECK`| Suppress the `self` upgrade nag (also suppressed under `CI`) |
| `XDG_CONFIG_HOME`    | Config base (`$XDG_CONFIG_HOME/naba`); `NABA_CONFIG_DIR` wins. Matches the vendor installer's receipt location |
| `XDG_CACHE_HOME`     | Cache base (`$XDG_CACHE_HOME/naba`) — holds the update-check cache |
| `XDG_DATA_HOME`      | Data base (`$XDG_DATA_HOME/naba`, reserved)                  |
| `XDG_BIN_HOME`       | Vendor binary dir (default `~/.local/bin`)                   |
| `CI`                 | When set, suppresses the `self` upgrade nag                  |
| `GEMINI_BASE_URL`    | Override Gemini API base URL (used by tests)                 |
| `OPENROUTER_BASE_URL`| Override OpenRouter API base URL (used by tests)             |
| `NABA_SITE_DOMAIN`   | Bare site host for `web/` (e.g. `naba.ysapp.net`) — S3 bucket + ACM cert + Route53 record; `PUBLISH_URL` derives from it (local `.envrc` + GitHub repo secret) |
| `PUBLISH_URL`        | Canonical public site URL — Pelican's production `SITEURL` (`publishconf.py`) and the absolute-URL source in dev. Keep `SITEURL` empty in `pelicanconf.py` so dev builds stay relative (local `.envrc` + GitHub repo secret) |
| `NABA_CF_DISTRIBUTION`| CloudFront distribution id for `web/` deploys (local `.envrc` + GitHub repo secret; `make provision` prints the id to set) |
| `NABA_HOSTED_ZONE_ID`| Route53 zone id for `web/` provisioning (local `.envrc` + GitHub repo secret; never committed) |
| `NABA_GA_MEASUREMENT_ID`| GA4 measurement id for the `web/` site — production build only (local `.envrc` + GitHub repo secret; never committed) |

## Secrets & Account-Specific Config

- **Never commit secrets or account-specific references** — API keys, AWS account ids,
  IAM/profile names, Route53 zone ids, GA measurement ids, ARNs, and similar.
- Scripts/config read such values from the **environment** (e.g. `NABA_SITE_DOMAIN`,
  `PUBLISH_URL`, `NABA_CF_DISTRIBUTION`, `NABA_HOSTED_ZONE_ID`), sourced locally from the
  gitignored `.envrc` (direnv) and stored canonically as **GitHub repo Secrets**
  (`gh secret set …`) — **Secrets, not Variables**, since Actions logs on this public repo are
  world-readable and Secrets are masked there. Fail closed when a required value is unset (e.g.
  `publishconf.py` raises without `PUBLISH_URL`; `make` aborts with an empty `S3_BUCKET`).
- Generated/captured artifacts that hold ids (e.g. `web/.aws-provision-state.json`) are
  **gitignored**, never committed.

## Providers

naba routes every image command through one of two providers (`gemini` | `openrouter`),
selected by the global `--provider` flag or the `provider` config key. **Agents shelling
out to naba must know:**

- **Resolution precedence:** CLI `--provider` > config `provider` > env-key autodetect
  (only `GEMINI_API_KEY` → gemini; only `OPENROUTER_API_KEY` → openrouter) > gemini fallback.
- **Multi-key reroute (intentional):** both keys set + no configured `provider` → autodetect
  picks **openrouter** (default slug `google/gemini-3.1-flash-image-preview`). To stay on
  Gemini, pin `provider: gemini` in config (config beats autodetect).
- **`--model` requires `--provider`** on the CLI (a bare model is ambiguous → usage error,
  exit 2). Config `model` without config `provider` is allowed.
- **`--quality` is per-provider:** Gemini maps `fast`/`high` to a model tier; OpenRouter
  passes `quality` through as a native request param without swapping the model slug.
  `openrouter/auto` cannot generate images and is rejected early.

**MCP mode**: When no output directory is configured, MCP handlers default to `~/.local/share/naba/images` (not CWD). Tool results return file paths + `ResourceLink` (no inline base64) to stay under Claude Desktop's ~1MB response limit.

## Dependencies

- `clap` (derive) — CLI framework
- `serde` / `serde_json` / `serde_norway` — (de)serialization; `serde_norway` for YAML config (`serde_yml` is forbidden, RUSTSEC-2025-0068)
- `reqwest` (rustls) + `tokio` — async HTTP + runtime
- `rmcp` — MCP server SDK
- `include_dir` — compile-time skill embed
- `sha2` / `hex` — hashing (embed tree-hash, self-update checksum)
- `flate2` + `tar` + `self-replace` — `naba self update` (pure-Rust `.tar.gz` extract + in-place binary swap)

## Distribution

Releases are cut by [cargo-dist](https://opensource.axo.dev/cargo-dist/): `[workspace.metadata.dist]`
in `Cargo.toml` drives a generated `.github/workflows/release.yml` (tag glob
`**[0-9]+.[0-9]+.[0-9]+*`) that publishes `.tar.gz` tarballs + `dist-manifest.json` to the GitHub
Release and pushes the Homebrew formula to `dixson3/homebrew-tap` (`HOMEBREW_TAP_TOKEN`). The
`curl|sh` vendor installer (→ `~/.local/bin` + a receipt) is the **documented default** and the
self-update-capable path; Homebrew is a documented alternative. A successful Release chains to
`web-deploy.yml`, which republishes naba.ysapp.net (GitHub OIDC — no local AWS keys). See SPEC §15
(SPEC-DIST) and the README.

**Releasing (lockstep rule).** Cutting a release means, in order: (1) bump the `Cargo.toml`
package `version`; (2) bump `web/pelicanconf.py` `NABA_RELEASE` to the **same** `vX.Y.Z` (the site
header shows this string so visitors see the latest version); (3) finalize `CHANGELOG.md` — move the
`## [Unreleased]` items into a new `## [X.Y.Z] - YYYY-MM-DD` section (dated, version without a `v`
prefix inside the brackets); (4) commit to `main`; (5) tag `vX.Y.Z` and push the tag, which triggers
`release.yml` → `web-deploy.yml`. **Never push a release tag without updating `NABA_RELEASE`** — a
stale header string would misreport the latest version. **Likewise, never push a release tag without
finalizing the CHANGELOG section** — cargo-dist matches the tag `vX.Y.Z` to the `## [X.Y.Z]` heading
to generate the GitHub Release notes, so a missing or mis-formatted section silently falls back to a
raw commit dump.

## Claude Code Skills

The Claude-facing skill lives in `skills/naba`, is embedded in the binary at compile time via
`include_dir` (`src/embed.rs`), and is deployed with `naba skills install` (see the README "Claude
Code Skills" section). There is no marketplace plugin and no installer script. It is one
skill invoked as `/naba <subcommand>` (e.g. `/naba generate`): seven
inline subcommands map 1:1 to CLI commands and three composites (`storyboard`, `batch`,
`brand-kit`) dispatch a subagent. `skills/naba/SKILL.md` runs `naba skills preflight --json` at
trigger time (SPEC-PREFLIGHT). Shared guidance lives once in `skills/naba/SKILL.md`;
per-subcommand detail in `skills/naba/commands/*.md`. See
`docs/specifications/skills.md`.

## Specifications

- Always reference `docs/specifications/*` as the source of truth for test plans
- When an implementation plan conflicts existing specifications, ask the operator to confirm the specification change before implementation
- Always persist a copy of the current implementation plan in `docs/plans` using a sequenced/hashed name

```
docs/decisions - important design and implementation decisions from previous sessions
docs/diary - implementation diary
docs/plans - archive of all implementation plans
docs/research - research used in design and implementation
docs/todos - historical todos
docs/specifications - per-domain UX-contract specs (source of implementation requirements);
    stable append-only clause IDs (SPEC-<AREA>-NNN). README.md is the index; the former
    monolithic SPEC.md is now a redirect stub. Domain files: commands.md, image-config.md,
    providers.md, configuration.md, exit-and-errors.md, json-output.md, mcp.md, skills.md,
    distribution.md
```

## Agent Operating Conventions

Issue tracking uses **beads (`bd`)**; the generic bd workflow conventions live in your
user-scope agent rules and are not duplicated here. naba-specific facts:

- **Local-only beads.** A local Dolt DB with **no remote** — never run `bd dolt push`.
  `.beads/issues.jsonl` is the git-tracked portable record; open/deferred beads sync to
  GitHub Issues (`dixson3/naba`) via the `beads-upstream` skill.

### Non-Interactive Shell Commands

**ALWAYS use non-interactive flags** with file operations to avoid hanging on confirmation prompts (cp/mv/rm may be aliased to `-i`):

```bash
cp -f source dest      # NOT: cp source dest
mv -f source dest      # NOT: mv source dest
rm -rf directory       # NOT: rm -r directory
```

Also: `scp`/`ssh` → `-o BatchMode=yes`; `apt-get` → `-y`; `brew` → `HOMEBREW_NO_AUTO_UPDATE=1`.
