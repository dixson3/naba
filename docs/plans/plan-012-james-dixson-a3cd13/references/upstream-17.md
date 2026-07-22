---
type: Reference
okf_spec: OKF-PLAN
---
# Upstream #17: Add a curated CHANGELOG.md for release notes

- **Number:** 17
- **Title:** Add a curated CHANGELOG.md for release notes
- **URL:** 
- **State:** OPEN
- **Labels:** type::task, priority::medium

## Body

cargo-dist currently generates the GitHub Release title/body from commit summaries (no CHANGELOG present). Add a hand-curated CHANGELOG.md (Keep a Changelog format) so release notes are readable and intentional. cargo-dist reads the matching version section for the release body. Backfill notable entries for recent releases (v0.6.x, v0.7.0) and adopt it going forward as a release step (fold into the AGENTS.md 'Releasing' lockstep rule).
