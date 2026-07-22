---
type: Reference
okf_spec: OKF-PLAN
---
# Upstream #16: Apply VOICE.md across remaining user-facing docs

- **Number:** 16
- **Title:** Apply VOICE.md across remaining user-facing docs
- **URL:** 
- **State:** OPEN
- **Labels:** type::task, priority::medium

## Body

VOICE.md (writing voice for user-facing docs) was authored and applied to README.md and the web/ pages that had violations. Sweep the remaining user-facing prose for the same three rules: (1) verbose/human-friendly exposition, (2) precedence as explicit ordered lists (never 'A > B > C'), (3) name the tool as `naba` (never bare) in prose. Targets: CONTRIBUTING.md, a full pass over web/content/pages/usage.md, and any other reader-facing docs. Lint (yf-markdown-lint authoring subset) and rebuild the web site to confirm.
