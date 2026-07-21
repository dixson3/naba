# Red-Team Review — pass 2 (re-review after pass-1 REVISE)

**Plan:** plan-011-james-dixson-2118ab
**Date:** 2026-07-21

## Verdict: APPROVE

## Pass-1 concern re-verification (all resolved, spot-checked against source)

- **C1 (high) — resolved.** Issue 2.1 now enumerates every CLI-framed element to `{% if cli %}`-gate
  (frontmatter `description`, intro, `allowed-tools`, dispatch table, `--flag` prose) with authored
  `{% if mcp %}` counterparts; the jinja-in-frontmatter decision is explicit (keep `name` shared,
  gate the `description` body). Verified safe: `build.rs render_skill_trees` runs the whole file
  (frontmatter included) through `render_str`; only the rendered trees are consumed, so no deployed
  YAML-validity risk.
- **C2 (medium) — resolved.** `check_traceability.py`/`make traceability` now in blast radius,
  Issue 1.1 (clause citation/exemption + `traceability_exemptions.yaml:98` update), Issue 2.3, and
  success criteria. The stale line-98 "mcp/ subtractive" text confirmed still present.
- **C3 (medium) — resolved.** Issue 2.3(b) names `read_skill_resource_returns_file_and_index`
  (asserts `### Prompt engineering` present, `## Router`/`## Preflight` absent) and
  `skill_resources_enumerate_files_and_index` (iterates `skill_files_mcp`) — both confirmed in
  `src/mcp.rs`.
- **C4 (medium) — resolved.** Capability gate is now a pre-2.1 block **and** an exit criterion of
  Issues 1.2 and 2.1, with the shared-SKILL.md leak-window rationale.
- **C5 (low) — resolved.** Issue 3.2 calls out the intermixed-source drift risk and directs scoping
  the manifest cli-framed contracts if the prose edges false-positive.
- **C6 (low) — resolved.** Issue 3.1 adds the README MCP-section verification (`e-readme-web-install`).

## Strengths

- Revisions are surgical and each lands on a verifiable source anchor.
- The frontmatter-jinja tradeoff is documented rather than latent, and genuinely safe given the
  existing template model.
- Capability gate as dual entry+exit criterion closes the real 2.1 leak window.

## Concerns

| # | Severity | Concern | Recommendation |
|:--|:---------|:--------|:---------------|
| C7 | low | The plan says "each of the 8 tool descriptions"; the 8 include `list_images` (a non-generation utility). A guidance pointer there is harmless but arguably noise. | During Issue 2.2, make a conscious choice whether the pointer belongs on `list_images` or only the 7 generation tools; keep the `test_mcp.py` `EXPECTED` update consistent. |

## Missing

- Nothing blocking. All pass-1 gaps are now present.

## Gate Assessment

Start Gate standard. Capability Gate correctly positioned as pre-2.1 block **and** exit criterion of
1.2 and 2.1; `cargo test embed::` is genuinely sufficient. "No reconcile gate" valid. No gate over-used.

## Upstream Assessment

Unchanged and consistent: no open GitHub issue, single tracking issue at intake, no partials/supersedes.

## Operator Resolutions

| # | Concern (short) | Status | Resolution |
|:--|:----------------|:-------|:-----------|
| C7 | pointer on list_images vs. 7 generation tools | resolved | Issue 2.2 reworded to require a conscious choice (all 8 vs. the 7 generation tools) with the `test_mcp.py` `EXPECTED` update kept consistent. |
