# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.8.0] - 2026-07-21

### Added

- Whole-skill garbage collection on `naba skills upgrade`: a skill dropped from a
  newer binary is now swept from disk instead of lingering forever. Removal is gated
  by the naba-ownership marker (a co-located non-naba skill is never touched), and
  sweeps surface in the per-target `gc` report under `--json` and `--dry-run`. The
  post-`self update` skills refresh inherits the behavior.
- A curated `CHANGELOG.md` (Keep a Changelog format), so cargo-dist generates readable
  GitHub Release notes from a hand-written section instead of a raw commit summary.

### Changed

- Swept the remaining user-facing docs (`CONTRIBUTING.md` and the `web/content` prose)
  to the `VOICE.md` writing-voice rules for a consistent reader experience.
- Folded a CHANGELOG-finalize step into the release lockstep so each tagged release
  ships an intentional set of notes.

## [0.7.0] - 2026-07-21

### Added

- `--harness` flag with idiomatic per-harness skills install and dual-purpose
  CLI/MCP skill renders.
- Tool-agnostic agent-tools SPEC covering the skills self-management lifecycle,
  the MCP-over-CLI interface, and `--json` agent output with envelopes.
- Genuine MCP-specific skill and resource guidance (how to invoke the MCP tools),
  replacing the render that only re-exposed CLI slash-command guidance.
- `VOICE.md` style guide, plus `CONTRIBUTING.md`, `SECURITY.md`,
  `CODE_OF_CONDUCT.md`, and issue/PR templates.
- Release-version string shown in the naba.ysapp.net site header.

### Changed

- Renamed the `--surface` flag to `--harness` (the deprecated `--surface` alias
  is retained).
- The vendor `curl | sh` installer is now the documented default install path.
- Per-domain enums are single-sourced with a golden drift-guard, resolving the
  skill-markdown ↔ `src/mcp.rs` enum drift.

## [0.6.1] - 2026-07-19

### Changed

- The website republishes automatically on release; the `web-deploy` workflow now
  runs off the release event.
- Bumped GitHub Actions to Node-24 majors (aws-credentials v6).

## [0.6.0] - 2026-07-19

### Added

- Consistent multi-provider configuration: nested per-provider config with
  migration and uniform API-key resolution.
- Provider registry with `naba provider` and `naba models` commands, and `--json`
  output across commands.
- AWS Bedrock image provider.
- MCP lazy-loading of skills as resources.
- Self-update and vendor install with a `naba skills preflight` surface.
- Pelican-based website (naba.ysapp.net) with a hosted `install.sh` bootstrap and
  production GA4 analytics.

### Changed

- Hardened build-time version derivation.

### Fixed

- Recognize AWS profile / SigV4 credentials in the Bedrock provider and model
  validity probe (#11).

### Removed

- Legacy Go source (the Rust cutover is complete).
