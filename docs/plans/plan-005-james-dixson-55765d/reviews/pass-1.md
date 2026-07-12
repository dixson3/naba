# Red-Team Review — Pass 1

**Plan:** plan-005-james-dixson-55765d
**Date:** 2026-07-12
**Status:** resolved

## Verdict: REVISE

## Strengths

- Reference (`references/yf-reference-report.md`) is load-bearing, not decorative: the port is
  a faithful translation of the yf pipeline (Fetcher seam, sha256-before-swap, swap-destination
  re-exec for post-update refresh, path-primary classification).
- Repo primitives verified to exist as claimed: `embed::skill_status` returns exactly the
  `installed/up_to_date/complete/unmodified` axes Epic C leans on; `doctor.rs` has provider-aware
  `api_key`/`api_live`/`model_reachable` + `resolve_provider`/`provider_api_key`.
- Parity-exemption claim is grounded in the repo's existing mechanism
  (`tests/parity/test_parity.py` port-only skip, `traceability_exemptions.yaml`, `SPEC-MIGRATE-*`
  precedent).
- cargo-dist single-package risk anticipated with a concrete mitigation; Homebrew-remains-default
  held consistently across scope, approach, and success criteria.

## Concerns

| # | Severity | Concern | Recommendation |
|:--|:--|:--|:--|
| 1 | high | Binary-up-to-date preflight axis has no defined behavior when the cache is absent — its default state on every fresh install until a cargo-dist release exists and has been fetched once. If preflight returns non-ok on absent cache, the wired skill gate fails on every invocation of a fresh install. | Make the binary axis tri-state (`up_to_date \| update_available \| unknown`); pin `unknown` (absent/stale cache) as **non-blocking** (overall stays `ok`). Add an absent-cache unit test. |
| 2 | medium | `src/dirs.rs` XDG resolution not reconciled with the existing `NABA_CONFIG_DIR`/`config.rs` config-dir resolver. cargo-dist writes the receipt to a fixed `~/.config/naba`; a user with `NABA_CONFIG_DIR`/`XDG_CONFIG_HOME` set would have `self`/`preflight` look where the installer never wrote. | Have `dirs.rs` defer to the same config-dir resolution `config.rs` uses (single source of truth); document that receipt lookup must match the installer's fixed `~/.config/naba`; add a precedence test. |
| 3 | medium | No follow-on bead files the bootstrapping dependency: `self update` and the binary axis are inert until the first cargo-dist release is cut. | File an explicit follow-on bead ("cut first cargo-dist release; verify `self update` end-to-end") and reference it from Success Criteria 1/3. |
| 4 | low | "Reuse `doctor::resolve_provider` + `provider_api_key`" — both are private `fn` in `doctor.rs`. | Add a task to promote/extract the provider-resolution helpers into a shared surface both `doctor` and `preflight` import. |
| 5 | low | Release-asset naming + tag-trigger glob change silently at cutover (`v*` → `**[0-9]+…`; `naba_darwin_arm64` → `naba-aarch64-apple-darwin.tar.gz`). | Extend the A.3 formula-diff mitigation to diff asset names and the tag glob; note the tag-format change in README/AGENTS. |
| 6 | low | Epic C fully blocked on all of Epic B (C.1 → B.7); only real coupling is the cache read. | Let C depend on the cache *schema* (tri-state `unknown`), not the full B.7 nag wiring, so a self-update slip does not block preflight. |

## Missing

- Absent/stale-cache preflight status contract (concern 1).
- Receipt-path precedence spec reconciling installer-fixed path vs `NABA_CONFIG_DIR`/`XDG_CONFIG_HOME` (concern 2).
- Explicit follow-on bead for the first cargo-dist release (concern 3).
- Test-strategy note that end-to-end `self update` is deferred to post-first-release and how it is verified.

## Gate Assessment

Start Gate (human/operator) appropriate. Capability Gate (cargo-dist available, `dist --version`,
blocks A.3) valid and correctly scoped to A.3; effectively soft via hand-author + attestation
fallback (yf precedent) — acceptable. Release-secret note correctly classified as informational.

## Upstream Assessment

#5 (retire Go source) → exclude is correct — orthogonal, and the Go binary is still the parity
baseline. Single coarse tracking issue at intake is reasonable (greenfield port). Gap: the
disposition table omits the follow-on tracking bead for the first cargo-dist release (concern 3).

## Operator Resolutions

| # | Concern (short) | Resolution | Status |
|:--|:--|:--|:--|
| 1 | Binary axis absent-cache behavior | Plan + SPEC updated: binary axis is tri-state `up_to_date\|update_available\|unknown`; `unknown` (absent/stale cache) is non-blocking (overall `ok`). Absent-cache unit test added to C.2. | resolved |
| 2 | dirs.rs vs config.rs resolver | A.2 updated: `dirs.rs` defers config-dir resolution to `config.rs` (single source of truth incl. `NABA_CONFIG_DIR`); receipt lookup path documented to match the installer's fixed `~/.config/naba`; precedence test added. | resolved |
| 3 | Follow-on bead for first release | Added to plan as an explicit close-time follow-on (Success Criteria + a `## Follow-on Work` note); filed as a deferred bead at intake, referenced from SC 1/3. | resolved |
| 4 | Private provider helpers | A.2/C.2 updated: promote/extract `resolve_provider`+`provider_api_key` to a `pub(crate)` shared surface. | resolved |
| 5 | Asset naming + tag glob change | A.3 mitigation extended to diff asset names + tag glob; README/AGENTS note the tag-format change. | resolved |
| 6 | C over-coupled to B.7 | C.1 dependency relaxed to the cache *schema* (tri-state `unknown`), not full B.7 nag wiring. | resolved |
