# Red-Team Review — pass 1

**Plan:** plan-011-james-dixson-2118ab
**Date:** 2026-07-21

## Verdict: REVISE

## Strengths

- The load-bearing de-risker is **verified true**: `embedded_tree_hash("naba")` → `skill_files` →
  `SKILLS = include_dir!("$OUT_DIR/cli")`; `SKILLS_MCP` (`$OUT_DIR/mcp`) is never hashed and never
  deployed. Authoring MCP content cannot change install-integrity or force an upgrade.
- The capability-gate test is the right proof: `cargo test embed::` runs
  `embedded_hash_matches_go_reference` (pins `NABA_TREE_HASH` against the cli render).
- "Oracle is a guard" is concrete: `test_mcp.py` hard-codes `commands/generate.md` and
  `commands/edit.md` in the required `skill://` enumeration set — removing them WILL fail it, so
  hand-review over blind `--update-golden` is well-founded.
- Epic sequencing (SPEC → build.rs → content → pointers → oracle → docs) is a valid dependency
  order; the "subtractive" text-change targets are real (skills.md:57, mcp.md:77/89, DRIFT-CHECK:46/138).

## Concerns

| # | Severity | Concern | Recommendation |
|:--|:---------|:--------|:---------------|
| C1 | high | **Issue 2.1 under-scopes the render rewrite.** Only `## Preflight`/`## Router`/`### Global flags` are `{% if cli %}`-gated today. The frontmatter `description` ("naba CLI, invoked as `/naba …`"), the intro line, `allowed-tools: [Bash, Read, Agent]`, and the dispatch table render into BOTH trees — so the mcp render still emits `/naba` framing. Satisfying "no `/naba`/no `--flags`" requires gating ALL of it + authoring `{% if mcp %}` counterparts (whole-file dual-authoring). Also an unresolved decision: gating the frontmatter `description` puts jinja inside source YAML frontmatter. | Rewrite 2.1 to enumerate every CLI element to `{% if cli %}`-gate (frontmatter description, intro, allowed-tools, dispatch table, `--flag` prose) with MCP counterparts; record an explicit decision on jinja-in-frontmatter vs. leaving frontmatter shared. |
| C2 | medium | **`check_traceability.py` + `traceability_exemptions.yaml` unlisted.** `make traceability` is a separate target (not in `cargo test`/parity/`make validate`). New `[NEW]`/`[PINNED]` SPEC clauses must be cited by a test or exempted; `traceability_exemptions.yaml:98` still says SPEC-EMBED-005 is "mcp/ subtractive" (stale after amend). | Add `check_traceability.py`/`make traceability` to blast radius, success criteria, and Issue 1.1/2.3; update the SPEC-EMBED-005 exemption text; cite or exempt any new clause id. |
| C3 | medium | **`src/mcp.rs` Rust unit tests will break, unlisted.** `read_skill_resource_returns_file_and_index` asserts the mcp `SKILL.md` contains `"### Prompt engineering"` and NOT `"## Router"`/`"## Preflight"`; `skill_resources_enumerate_files_and_index` iterates `skill_files_mcp("naba")`. Re-authoring + removing `commands/*.md` shifts both. | List these `src/mcp.rs` unit tests in 2.1/2.2 blast radius; update the content/enumeration assertions to the intended new MCP framing. |
| C4 | medium | **Capability gate guards entry to 2.1 but not the authoring within 2.1.** The `{% if mcp %}` blocks are authored into the shared `skills/naba/SKILL.md`; a malformed block/whitespace-control leaks into cli during 2.1. The gate ("Blocks: 2.1") only proves the pre-authoring state. | Make `cargo test embed::` green an explicit **exit** criterion of Issues 1.2 and 2.1, not just a pre-2.1 block. |
| C5 | low | **Drift-check reads the intermixed *source* SKILL.md** (`skill-md` globs the raw source now carrying both cli+mcp prose); edges `e-skill-spec`/`e-readme-desc`/`e-web-skills-subcommands` do LLM prose comparisons and may false-positive on MCP-only content. | In 3.2 note the intermixed-source risk; if needed scope the manifest contracts to cli-framed sections. |
| C6 | low | **README MCP section unverified** (`README.md:331,371-374`, bound by `e-readme-web-install`). Likely stays accurate (no command enumeration) but not listed. | Add a one-line "verify README MCP section still accurate" check to 3.1/3.2. |

## Missing

- `check_traceability.py` + `traceability_exemptions.yaml:98` — blast radius, gates, success criteria.
- `src/mcp.rs` unit tests (`read_skill_resource_returns_file_and_index`, `skill_resources_enumerate_files_and_index`).
- Explicit enumeration in 2.1 of CLI framing to gate + jinja-in-frontmatter decision.
- `README.md` MCP section as a possible edit.

## Gate Assessment

Start Gate standard/appropriate. Capability Gate (cli byte-identity, `cargo test embed::`) correctly
conceived and its test genuinely sufficient — but mispositioned as pre-2.1 only; should also be an
exit criterion of 1.2 and 2.1. "No reconcile gate" justification valid. No gate over-used.

## Upstream Assessment

Consistent: no open GitHub issue, single tracking issue at intake, no partials/supersedes. No concern.

## Operator Resolutions

| # | Concern (short) | Status | Resolution |
|:--|:----------------|:-------|:-----------|
| C1 | 2.1 under-scopes CLI-framing gating + frontmatter decision | resolved | Rewrote Issue 2.1 to enumerate every CLI element to gate (frontmatter `description`, intro line, `allowed-tools`, dispatch table, `--flag` prose) + author `{% if mcp %}` counterparts; added an explicit design decision to Scope + Approach: **leave the shared frontmatter's `name` intact but gate the `description` body via `{% if cli %}`/`{% if mcp %}`** (minijinja renders the frontmatter as text, so the deployed cli render stays valid YAML; the raw source frontmatter carrying jinja is an accepted, documented tradeoff). Added a Success Criterion asserting the rendered mcp SKILL.md contains no `/naba` and no `--flag`. |
| C2 | traceability target + stale exemption | resolved | Added `check_traceability.py` / `make traceability` to Issue 1.1 (clause citation/exemption) and Issue 2.3 (run it), the blast radius, and Success Criteria; Issue 1.1 now updates `traceability_exemptions.yaml` line-98 SPEC-EMBED-005 text. |
| C3 | src/mcp.rs unit tests break | resolved | Issue 2.3 now explicitly lists and updates `read_skill_resource_returns_file_and_index` and `skill_resources_enumerate_files_and_index` in `src/mcp.rs` to the new MCP framing/enumeration. |
| C4 | gate as exit criterion | resolved | Capability gate reworded: `cargo test embed::` green is an **exit** criterion of Issues 1.2 AND 2.1 (not just a pre-2.1 block); Issue text updated accordingly. |
| C5 | intermixed-source drift false-positive | resolved | Issue 3.2 now calls out the intermixed-source risk and scopes the manifest cli-framed contracts if drift-check false-positives on MCP-only content. |
| C6 | README MCP section | resolved | Added a "verify README.md MCP section still accurate (e-readme-web-install)" check to Issue 3.1. |
