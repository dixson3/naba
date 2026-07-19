# Red-Team Review — pass 1

**Plan:** plan-007-james-dixson-6b5a59
**Date:** 2026-07-19

## Verdict: REVISE

## Strengths
- Findings genuinely drive the approach (MCP lazy-load "low surgery" from exp-001; migration seam
  pre-existing from exp-002; reqwest-vs-SDK tension from exp-003) — the plan changed its Bedrock
  recommendation *against* the operator's initial answer based on evidence, and gated the disagreement.
- The bedrock-transport capability gate is correctly placed (blocks 3.2 only, records the conflict).
- Docs-last ordering is sound (Epic 5 documents shipped behavior + adds DRIFT-CHECK edges).
- Migration risk treated seriously (`.bak`, idempotency, no-op paths, per-path unit tests) reusing a
  tested seam.
- Registry-refactor precedence risk named + mitigated (parity gates 2.1; SPEC §5 in lockstep).

## Concerns
| # | Severity | Concern | Recommendation |
|:--|:--|:--|:--|
| 1 | high | Config-migration strategy is "unselected" in §Scoping yet fully committed in Issue 1.3 + Success Criterion 1, with no gate — same operator-answer-vs-plan tension Bedrock got a gate for. | Add a `config-migration` decision gate blocking Epic 1.3 (symmetric to bedrock-transport), OR fold strategy confirmation into the Start Gate and change "unselected" → the selected strategy. |
| 2 | high | "Map old flat keys onto default_provider" is under-defined for `api_key` (which is Gemini-scoped historically). A user with `provider: openrouter` + stray Gemini `api_key` would misfile the key. Old config with no `provider` field also undefined. | Specify per-key mapping in 1.3: `api_key`→`providers.gemini.api-key`; `model`/quality→`providers.<default>.model`; `provider`→`default_provider` (explicit default when absent). Add a unit test for "openrouter default + stray gemini api_key". |
| 3 | medium (borderline high) | Epic 1 "independently buildable" vs the flat→nested ripple: 1.1 replaces the struct but the ~9 select.rs sites / `to_config_defaults`/`resolve_*` aren't rewired until Epic 2.1 — Epic 1 may not compile/pass parity alone. | Add an Issue 1.x adapting select.rs call sites to the nested struct (pre-registry shim) so Epic 1 stays green, OR declare epics 1+2 a single landable unit and drop the "Epic 1 independent" claim. |
| 4 | medium | Epic 6 (agent-tools SPEC) is un-investigated (no yoshiko-flow recon finding) and out of scale — a research+authoring effort riding the same approval as shipping-naba work; also tail-coupled to 2.4+4.2. | Split Epic 6 into its own plan seeded by a yoshiko-flow recon finding, OR keep 6.1 as an investigation spike with an explicit go/no-go before 6.2. |
| 5 | medium | "--json universal" has no defined envelope contract or parity assertion — each new envelope shaped ad hoc; Success Criterion 5 not enforceable. | Add a SPEC clause defining the common envelope (status/data/error) + a parity/traceability test enumerating all subcommands asserting envelope presence + schema. |
| 6 | medium | Autodetect "2-bool → 3-key" becomes 3-bool with Bedrock; "preserve precedence" is under-specified for multi-cred combinations the parity suite never covered. | Specify explicit N-provider precedence in SPEC §5 (2.5) + autodetect tests for new multi-cred combos (esp. Bedrock alongside gemini/openrouter). |
| 7 | low | Per-provider default model when a provider entry lacks `model` — migration only sets `model` for the default provider; non-default providers need a built-in fallback. | Define the built-in per-provider default (used when `providers.<name>.model` absent) in 1.1/1.5. |
| 8 | low | Docs 5.1 depends on Bedrock (3.3) but Risk §Scope says Bedrock is deferrable — 5.1 would block or document an unshipped provider. | Make 5.1's Bedrock content a separable sub-item, or state that deferring Bedrock defers its doc rows. |

## Missing
- No investigation finding backing Epic 6 (yoshiko-flow state / SPEC.md / where the SPEC should land) — largest evidence gap.
- No universal `--json` envelope schema in SPEC (Concern 5).
- No explicit downgrade/rollback note beyond `.bak` (one-liner in Risks would close it).
- No stated seam for keeping the image-generation path green across the Epic 1→2 boundary (Concern 3).

## Gate Assessment
- Start Gate: present + appropriate.
- bedrock-transport gate: exemplary (blocks 3.2, records operator-vs-finding conflict, clear condition/test).
- **Gap:** a config-migration gate is missing though structurally identical to the Bedrock situation — add one blocking 1.3 (safer, since it touches every user's on-disk config), or resolve at the Start Gate and drop the "unselected" language.

## Upstream Assessment
- #7 (cargo-dist) exclude: correct (unrelated release mechanics; plan leans on pre-1.0 status to justify the breaking schema without absorbing it).
- #8 (plan-006 website) exclude: reasonable (done/deployed tracker). Nuance: Epic 5 extends #8's deliverable — a one-line lineage note would help. Closing #8 as housekeeping is sound + correctly kept separate.

## Sequencing / Dependency Assessment
- config → registry → Bedrock → MCP → docs is the right spine; Epic 2→1 and Epic 3→2.1 deps correct/non-negotiable.
- Epic 4 (MCP) correctly independent — could parallelize with 1–3 to compress schedule.
- Epic 5 deps correctly force docs to trail shipped behavior; wrinkle = 5.1↔Bedrock coupling vs deferrable claim (Concern 8).
- Hidden coupling: Epic 1→2 build seam (Concern 3) is the top sequencing risk.
- Epic 6 tail coupling (6.2 on 2.4+4.2) + un-de-risked 6.1 → reinforces split/defer.

## Operator Resolutions
| # | Concern (short) | Resolution | Status |
|:--|:--|:--|:--|
| 1 | config-migration gate/strategy | Operator SELECTED auto-migrate + `.bak`, **no separate gate** — confirmation folded into the Start Gate condition; §Scoping wording changed "unselected" → SELECTED. | resolved |
| 2 | api_key per-key mapping | Issue 1.3 now specifies explicit per-key mapping (`api_key`→`providers.gemini.api-key`; `model`/`quality`→default_provider entry; `provider`→`default_provider` w/ fallback) + the "openrouter default + stray gemini api_key" unit test. | resolved |
| 3 | Epic 1→2 build seam / shim | Added Issue 1.6 — a pre-registry shim adapting `select.rs`/`resolve_*`/`to_config_defaults` to the nested Config so Epic 1 builds + parity stays green on its own. | resolved |
| 4 | Epic 6 scope split/defer | Operator chose **SPLIT** — agent-tools SPEC moved to a new follow-on plan (plan-008), seeded by a yoshiko-flow recon finding; removed from plan-007 (note after Epic 5; phasing + SC8 updated). | resolved |
| 5 | --json envelope contract + test | Issue 2.4 now defines a universal envelope contract (SPEC clause) + a traceability/parity test enumerating every subcommand asserting envelope presence + schema. | resolved |
| 6 | N-provider autodetect precedence | Issue 2.1 defines an explicit N-provider precedence (registry ordering as tie-break) + multi-cred autodetect tests incl. Bedrock; SPEC §5 (2.5) records it. | resolved |
| 7 | per-provider default model fallback | Issue 1.1 adds a built-in per-provider default-model fallback when `providers.<name>.model` is absent; SPEC/IG (1.5) document it. | resolved |
| 8 | docs 5.1 Bedrock coupling | Issue 5.1 makes the Bedrock doc rows a separable sub-item (gated on 3.3); the rest of the Config page depends only on 1.4+2.2, so deferring Bedrock defers only its rows. | resolved |

**Final status: all concerns resolved (2026-07-19). Plan revised; re-running red-team for pass 2.**
