# Red-Team Review — pass-1

**Plan:** plan-003-james-dixson-94015b
**Date:** 2026-06-14
**Verdict:** REVISE

## Strengths

- Core bug is real and well-evidenced: `client.go:19` hardcodes the retired model; `NewClient` falls back to it whenever config `model` is empty. "Fresh install broken" verified by code path + `models.list` (model absent).
- Load-bearing schema is live-verified, not assumed (Schema A confirmed with 200s; `responseFormat.image` disproven). `omitempty`/byte-identical-bare-call design is correct against the real `GenerationConfig` struct.
- Permissive-API finding correctly converted to a hard requirement (client-side enum validation, 1.3).
- Risk table honest ("no working baseline to preserve" is correct — old model 404s).
- Regression-guard test (5.1) targets the exact bug class that shipped before.

## Concerns (verbatim, with severity)

- **C1 [high] MCP `resolveClient()` has no model-override parameter — scope #4 "full parity" is not achievable as written.** `internal/mcp/server.go:59-66` builds the client from `cfg.Model` only; handlers call `resolveClient()` with no per-call model/imageConfig. Issue 3.1 treats this as param-passing but it is a signature change: `resolveClient`, `generateAndReturn`, `generateWithImageAndReturn` must thread model + imageConfig. **Rec:** expand 3.1 to state the helper-signature refactor; add to Approach §4.
- **C2 [high] MCP output path hardcodes `image/png` before the image exists — the JPEG mismatch is concrete and currently in MCP.** `server.go:187,259,325` call `output.OutputPath(outDir, "...", "image/png")` to build the filename before generation; `WriteImage` only derives extension from mimeType when `outputPath == ""`. So MCP-generated JPEGs are written to `.png` files today. **Rec:** a concrete issue must fix the three call sites to defer extension to the returned mimeType; add an MCP-path assertion to the smoke test.
- **C3 [medium] `-o <name>.png` with a JPEG response writes mislabeled bytes — pre-existing, but this plan makes it the default case.** A user `-o foo.png` is passed straight to `WriteImage`, which honors the literal extension. Before, the dead model's mimeType was moot; now the default returns JPEG. **Rec:** decide+document policy (warn/correct extension vs "user path authoritative"); add a line to Approach/Success Criteria; smoke-test the chosen behavior.
- **C4 [medium] Config `quality` vs config `model` precedence unspecified when both set in config.** `--model > --quality > config > default` collapses both config keys into one rung. **Rec:** define intra-config tiebreak (recommend config `model` beats config `quality`, mirroring flags); add to 2.3 + SC3.
- **C5 [medium] `--aspect`/`--resolution` excluded from icon/pattern/diagram/story without stated rationale; `icon --size` (px) vs `--resolution` will confuse.** All call `client.Generate()` and would honor imageConfig. **Rec:** add an explicit out-of-scope line with reason (+ follow-up note); clarify in docs that icon `--size` is pixel canvas, not imageConfig `imageSize`.
- **C6 [medium] `--quality` vs `--model` precedence needs cobra `Changed()`, not empty-string sentinels.** Both are persistent root flags; empty-string "unset" is ambiguous with config `quality` in play. **Rec:** 2.2 should specify `cmd.Flags().Changed("model")/Changed("quality")`.
- **C7 [low] DRIFT-CHECK manifest is genuinely missing a model-id edge; plan defers it ("consider").** Model id duplicated across `client.go:19`, `README.md:99`, `IG/configuration.md:21,39`, `EDD/CORE.md:161` — the exact cross-source duplication drift-check exists to catch, and the bug class that recurred. **Rec:** promote to a concrete sub-task: add a `gemini-source` node (fixed authority) + `value-equal` edge to the doc references; sequence before Epic 4 closes (manifest re-approval per §0).

## Missing

- No issue explicitly owns the `-o`-wrong-extension / MCP-hardcoded-png fix (C2/C3).
- No note on `EnrichIconPrompt` size semantics vs `imageConfig.imageSize` (future icon resolution).
- MCP `count>1` + imageConfig interaction unspecified (all N reuse same imageConfig — fine, but state it).
- No backward-compat note: existing users with `model: gemini-2.5-flash-image` keep working (precedence preserves it). Only the dead-2.0 case is reasoned about.

## Gate Assessment

Two gates appropriate and minimal. Capability Gate test is valid and matches the verified shape, correctly scoped to block only 5.2. Gap: the gate test exercises only the *bare* request, not `imageConfig`/bad-enum (the load-bearing new behaviors) — consider an `imageConfig` variant in the gate test. Start Gate standard. No over-gating.

## Upstream Assessment

Clean. 0 open issues (verifiable, dated). No dispositions/supersedes/partials. Only upstream-adjacent action is DRIFT-CHECK manifest re-approval if the model-id edge (C7) is added — internal gate, not external issue.

## Operator Resolutions

| # | Concern | Severity | Resolution | Status |
|:-:|:--------|:---------|:-----------|:-------|
| C1 | MCP resolveClient model-override = signature refactor | high | Issue 3.1 rewritten as an explicit signature refactor of `resolveClient`/`generateAndReturn`/`generateWithImageAndReturn`; Approach §4 updated. | resolved |
| C2 | MCP hardcoded image/png writes JPEG to .png | high | New Issue 1.4 owns the fix to the three `server.go` call sites + writer; Risk row updated; 5.2 asserts the MCP path. | resolved |
| C3 | -o <name>.png + JPEG mislabel — policy decision | medium | Operator chose: **correct the extension + warn + surface requested-vs-actual in JSON**. Captured as Scope #6 + Issue 1.4 + SC6. | resolved |
| C4 | config quality vs config model tiebreak | medium | Issue 2.3 + SC3: intra-config tiebreak config `model` > config `quality`. | resolved |
| C5 | aspect/resolution scope + icon --size clarity | medium | Operator chose **all generative commands** (generate/edit/restore/pattern/diagram/story); icon px-only. Scope #2, Issue 2.1, docs (4.1) clarify `icon --size` ≠ `imageSize`. | resolved |
| C6 | --quality/--model precedence via cobra Changed() | medium | Issue 2.2 specifies `cmd.Flags().Changed()`; Approach §2 + SC3. | resolved |
| C7 | DRIFT-CHECK model-id edge: promote to concrete sub-task | low | New Issue 4.3 adds `gemini-source` node + `value-equal` edges (manifest re-approval); 5.3 runs drift-check against it. | resolved |

Missing-items also folded in: MCP `count>1`+imageConfig note (3.1), backward-compat for existing `model:` configs (Risk + 4.1 + SC7), capability-gate test now exercises `imageConfig`.

**Status:** resolved (all concerns addressed in plan v2; awaiting operator approval)
