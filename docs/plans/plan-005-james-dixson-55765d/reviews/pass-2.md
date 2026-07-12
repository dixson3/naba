# Red-Team Review — Pass 2 (re-review after pass-1 REVISE)

**Plan:** plan-005-james-dixson-55765d
**Date:** 2026-07-12
**Status:** resolved

## Verdict: APPROVE

## Summary

All six pass-1 concerns verified genuinely resolved in the plan body (not merely asserted in the
resolutions table). Load-bearing repo claims re-verified live: `config.rs:config_dir()` is the
referenced resolver; `doctor.rs` provider helpers are private `fn`; `embed::skill_status` returns
exactly `installed/up_to_date/complete/unmodified`; `Cargo.toml` is single-package; the
`v*`/`naba_darwin_arm64` workflow shape matches the A.3 diff targets.

## Strengths

- The tri-state fix (pass-1 concern 1) is threaded consistently through Approach prose, Issue
  C.2, Success Criterion 4, and the SPEC pin in D.1 — no residual place where an absent cache
  could block a fresh-install skill gate.
- All repo-primitive claims verified live.
- The `Fetcher`-seam + follow-on-release split cleanly separates what execution can prove
  (config/pipeline behind a seam) from what needs a live endpoint; Success Criteria 1/3 are
  honest about that boundary.

## Concerns

| # | Severity | Concern | Recommendation | Status |
|:--|:--|:--|:--|:--|
| 1 | low | C.1 relaxation note attributed the update-check cache schema to `receipt.rs`; that module handles the cargo-dist receipt, a different file. The decoupling still holds (C.2 needs only the cache path from `dirs.rs` + the absent→`unknown` contract). | Reword the C.1 note to cite `dirs.rs` (cache path) + the tri-state contract, not `receipt.rs`. | resolved |

## Missing

None material. The four pass-1 "missing" items (absent-cache status contract, receipt-path
precedence spec, follow-on release bead, deferred-e2e test-strategy note) are all present.

## Gate Assessment

Unchanged from pass-1 and sound. Start Gate (human/operator) appropriate. Capability Gate
(cargo-dist available; `dist --version`; blocks A.3) correctly scoped with an operator-attestation
hand-author fallback. Release-secret note correctly informational.

## Upstream Assessment

#5 (retire Go source) → exclude remains correct (orthogonal; Go is still the parity baseline).
The pass-1 gap (follow-on tracking bead absent from the disposition narrative) is closed — the
Upstream Issues section records the deferred follow-on bead hoisted at land-the-plane.

## Operator Resolutions

| # | Concern (short) | Resolution | Status |
|:--|:--|:--|:--|
| 1 | C.1 note cites wrong module | Reworded C.1 note: cache **path** from `dirs.rs` (A.2) + absent→`unknown` tri-state contract; explicitly not the B.7 `update_check.rs` writer. | resolved |
