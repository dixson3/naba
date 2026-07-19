# Red-Team Review — Pass 1

**Plan:** plan-006-james-dixson-d333fd
**Date:** 2026-07-18

## Verdict: REVISE

## Strengths
- Scope discipline: "GitHub Releases canonical; host only `install.sh`; no `/downloads/`
  mirror, no `latest.json`; zero Rust changes" stated consistently across Objective,
  Approach, and Success Criteria #3, grounded in `src/self_cmd/update.rs`.
- Gating is load-bearing, not ceremony: look-approved blocks public push; go-live fences the
  billable apply. Local-first keeps AWS out of the loop until approved.
- #7 tolerance is designed (fail-safe placeholder `install.sh`), decoupling site go-live from
  the unreleased cargo-dist tag.
- Proven-pattern reuse (thesoftwarefactory Pelican + Makefile + `s3_upload`).
- Dependency DAG is coherent and acyclic; gate wiring matches (3.2 ← 1.5, 3.1).

## Concerns
| # | Severity | Concern | Recommendation |
|:--|:---------|:--------|:---------------|
| C1 | high | Private S3 + OAC does not auto-resolve subdirectory index documents; CloudFront default-root-object only rewrites `/` → `index.html`, not `/install/` → `/install/index.html`. Pretty Pelican URLs would 403/404. This is where the plan diverges from the "proven pattern." | Add a CloudFront viewer-request Function that appends `index.html` to path-terminating requests (explicit deliverable in 3.1), OR pin Pelican to non-pretty `.html` URLs. Decide before 1.2/1.3 author nav. |
| C2 | medium | `sync_installer.sh` claims to verify `naba-installer.sh` against `dist-manifest.json`, but cargo-dist may not publish a checksum for the installer script itself (only tarball `.sha256` sidecars). Mitigation could be hollow. | Confirm (post-#7 or from cargo-dist 0.32 docs) the installer has a verifiable digest; if not, downgrade to "pin the manifest-named release tag + HTTPS fetch" and adjust Success Criteria #3 wording. |
| C3 | medium | "Lights up automatically once #7 lands" has no owner and no follow-on bead; a manual re-sync + deploy + invalidate is required and will silently not happen. | File a follow-on bead (discovered-from this plan, depends-on #7) to re-run `sync_installer` + deploy + invalidate `/install.sh` + verify `curl \| sh` end-to-end. |
| C4 | medium | `findings/exp-001-landscape.md` still describes a `/downloads/` + `latest.json` mirror — contradicts the finalized scope; a cold reader would build the wrong thing. | Reconcile the finding to the final decision (strike/annotate the `/downloads/` mirror language as superseded). |
| C5 | medium | Issue 3.1 bundles bucket+OAC+policy+cert+DNS+distribution+alias as "idempotent where practical." Partial failure (esp. ACM validation wait) can orphan/duplicate certs or half-build the distribution; naive re-run may request a second cert. | Capture/reuse resource identifiers (cert ARN, distribution id, OAC id), check-before-create per resource, persist to local config; state cert-validation polling/timeout behavior. |
| C6 | low | Short-TTL `/install.sh` needs `Cache-Control: max-age=300`, but tree-wide `s3 sync` won't set per-key metadata. | Make the `sync_installer`/upload step set `--cache-control max-age=300` on the `install.sh` key explicitly. |
| C7 | low | Go-live gate authorizes "billable" infra with no cost figure. | Add a one-line expected-cost estimate to the gate Instructions. |

## Missing
- CloudFront error/404 behavior (custom error responses; 403→404 mapping for the private-bucket case) — ties to C1.
- URL-style decision (pretty vs `.html`) not pinned — the pivot for C1 and theme link authoring; make it an explicit Epic 1 decision.
- Rollback/teardown path for the AWS resources — the plan is create-only; add a teardown note to `web/README.md` (Issue 4.1).

## Gate Assessment
Both capability gates justified and correctly fenced (look-approved blocks go-live path via
3.2←1.5; go-live blocks billable apply via 3.3←3.2). `sts get-caller-identity` proves creds
not authorization, but operator confirmation is the real control (appropriate for a human
gate). Add cost estimate (C7). Start Gate present. Not over-gated.

## Upstream Assessment
- **#7 — partial (related, not resolved):** correct. Plan consumes #7's output as a soft
  dependency, tolerates its absence, does not cut the tag. Gap: unowned post-#7 redeploy (C3)
  — file the follow-on bead.
- **#5 — exclude:** reasonable (Go retirement unrelated). No supersedes claimed (correct).

## Operator Resolutions
| # | Concern | Resolution | Status |
|:--|:--------|:-----------|:-------|
| C1 | OAC subdir index resolution | Approach + Issue 3.1 updated: CloudFront viewer-request Function appends `index.html`; pretty URLs retained; URL-style pinned in Epic 1 (Issue 1.1); CloudFront error/404 behavior specified in 3.1. | resolved |
| C2 | installer checksum verification | Issue 2.1 + Success Criteria #3 softened: pin the manifest-named release tag + HTTPS fetch; verify sha256 only if the manifest publishes one for the installer. | resolved |
| C3 | post-#7 redeploy unowned | Added Issue 4.3: file a follow-on bead (discovered-from, depends-on #7) for the re-sync + deploy + invalidate + end-to-end verify. | resolved |
| C4 | findings contradict scope | `findings/exp-001-landscape.md` reconciled — `/downloads/` + `latest.json` language annotated as superseded. | resolved |
| C5 | provisioning idempotency | Issue 3.1 expanded: check-before-create per resource, capture/reuse cert ARN + distribution id + OAC id, persist to local config, explicit cert-validation polling/timeout. | resolved |
| C6 | per-object Cache-Control | Issue 2.1/3.3 updated: upload `install.sh` with explicit `--cache-control max-age=300`. | resolved |
| C7 | go-live cost estimate | go-live gate Instructions now include a one-line cost ballpark. | resolved |
| M1 | teardown path | Issue 4.1 now includes a teardown/rollback note for the AWS stack. | resolved |
