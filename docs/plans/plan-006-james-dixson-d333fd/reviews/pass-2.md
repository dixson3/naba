# Red-Team Review — Pass 2

**Plan:** plan-006-james-dixson-d333fd
**Date:** 2026-07-18

## Verdict: APPROVE

Cycle 1 returned REVISE. This cycle verifies the pass-1 concerns are resolved and checks for
regressions. All seven concerns (C1–C7) and all three Missing items are closed in the plan
body (not just the resolutions table). One trivial low-severity item was introduced by the C1
fix and has been folded in (below).

## Strengths
- Every pass-1 concern is traceably closed in the actual Approach/Issues/Gates/Success-Criteria
  text, so a cold reader building from the plan gets the corrected design.
- C1 (the sole high) resolved coherently: URL style pinned first (Issue 1.1, pretty `{slug}/`),
  the CloudFront viewer-request Function is an explicit Issue 3.1 deliverable, the risk is
  restated, and Issue 3.4 verifies non-root pages resolve before go-live. The 1.1 → 1.2 nav
  chain is internally consistent.
- C2 softening is honest — no over-claimed installer verification; degrades to pinned-tag HTTPS
  floor, Success Criteria #3 wording matches.
- C5 idempotency is specific: check-before-create per resource, capture/reuse of cert ARN +
  distribution id + OAC id + function ARN, explicit `wait certificate-validated` + timeout.
- C3 is owned work (Issue 4.3 follow-on bead, depends-on #7), corroborated by
  `references/upstream-7.md`.
- C4 findings banner, C6 per-key `--cache-control max-age=300`, C7 cost ballpark, and the
  Issue 4.1 teardown note are all present and consistent.

## Concerns
| # | Severity | Concern | Resolution |
|:--|:---------|:--------|:-----------|
| C8 | low (new) | The C1 fix added a 403/404 → `404.html` CloudFront error mapping, but no issue authored `404.html` (Pelican emits none by default) — a dangling reference. | Resolved: Issue 1.3 now authors a `404.html` page as the mapping target. |
| C9 | low (advisory) | Custom 403→404 mapping can mask an OAC/bucket-policy lockout during bring-up. | No change required; Issue 3.4's end-to-end fetch verification mitigates. Optionally inspect raw origin responses during first provisioning before trusting the error mapping. |

## Missing
- Nothing blocking. The three pass-1 Missing items (CloudFront 404 behavior, URL-style
  decision, teardown path) are all addressed. C8 (the only residual) is now folded into
  Issue 1.3.
- Minor/advisory: Issue 4.3 carries no `depends-on` — defensible (filing the bead is
  order-independent); not a defect.

## Gate Assessment
Unchanged from pass-1 and sound. Both capability gates load-bearing and correctly fenced
(look-approved blocks go-live via 3.2 ← 1.5; go-live blocks the billable apply via 3.3 ← 3.2).
C7 cost estimate now in the go-live Instructions. `sts get-caller-identity` proves creds not
authorization, but operator confirmation is the real control (appropriate for a human gate).
Start Gate present. Not over-gated.

## Upstream Assessment
Unchanged and correct. #7 — partial (related, not resolved): plan consumes #7's output as a
soft dependency, tolerates its absence, does not cut the tag; the pass-1 unowned-redeploy gap
is closed by Issue 4.3. #5 — exclude (Go retirement unrelated). No supersedes claimed.

## Operator Resolutions
| # | Concern | Resolution | Status |
|:--|:--------|:-----------|:-------|
| C8 | `404.html` referenced but not authored | Issue 1.3 extended to author a `404.html` page (target of the 403/404 error mapping). | resolved |
| C9 | 403→404 mapping may mask OAC lockout | Advisory only; Issue 3.4 end-to-end verification mitigates. No plan change. | resolved (no change) |
