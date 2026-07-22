---
type: Reference
okf_spec: OKF-PLAN
---
# Upstream Issue Triage: issues 16 17 18

Instructions: For each issue, set disposition to: include, exclude, partial, supersede.
Add notes as needed. When done, say "triage ready".

_Full issue bodies are inlined under `references/upstream-<N>.md` (regenerated on re-triage)._

## #16 — Apply VOICE.md across remaining user-facing docs
Labels: type::task, priority::medium
> VOICE.md (writing voice for user-facing docs) was authored and applied to README.md and the web/ pages that had violations. Sweep the remaining user-facing prose for the same three rules: (1) verbose/...

**Disposition:**
**Notes:**

## #17 — Add a curated CHANGELOG.md for release notes
Labels: type::task, priority::medium
> cargo-dist currently generates the GitHub Release title/body from commit summaries (no CHANGELOG present). Add a hand-curated CHANGELOG.md (Keep a Changelog format) so release notes are readable and i...

**Disposition:**
**Notes:**

## #18 — skills lifecycle: add whole-skill garbage collection to install/upgrade

> ## Summary

The install/upgrade skill lifecycle prunes **stale files within a still-shipped skill** but has **no whole-skill garbage collection**. If a skill is dropped from a future binary, its previ...

**Disposition:**
**Notes:**
