# Red-Team Review — pass 1

**Plan:** plan-004-james-dixson-9a7b16
**Date:** 2026-07-11

## Verdict: REVISE

Findings are strong and well-grounded — every load-bearing claim spot-checked
against the Go source is accurate. No high-severity blockers, no need for more
investigation. A cluster of concrete medium sequencing/acceptance gaps, cheap to fix
in-place, should be resolved before execution.

## Strengths

- Contract-first ordering is genuinely sound — Epic 1 (SPEC + pytest harness +
  Go-captured goldens) needs no Rust to exist; the `$NABA_BIN` design makes
  "capture goldens before any Rust" real.
- Findings are compiler/source-validated and faithfully carried (INV-1 BESPOKE via
  `E0599`; INV-2 `/api/v1/images` + `openrouter/auto`-can't-image; INV-3
  async-layer/`rmcp`/`serde_yml`-forbid; INV-4 12-groups correction).
- Parity claims accurate: 8 MCP tools, 12 command groups, exit codes 1/2/3/4/5/10,
  6 config keys, cobra-parse→exit-1 subtlety, MCP-only `NABA_OUTPUT_DIR` asymmetry.
- Request-golden assertions carried (mock records outbound JSON → pins the 7
  `Enrich*` functions across the language switch).
- Sanctioned-divergence zones enumerated with inventory/regex pinning.

## Concerns

1. **[medium] Epic 2 skeleton has no dependency edge to SPEC (1.1)**, contradicting
   the contract-first narrative. 2.1 depends only on `gate-start-gate`, yet the clap
   command tree/flags/enums it scaffolds are defined by SPEC §3.
   *Rec:* add `2.1 depends-on 1.1`, or drop the "before any Rust exists" absolutism
   and state skeleton-in-parallel is intended.
2. **[medium] Highest external unknown (OpenRouter live wire behavior) validated last
   and skippable.** 5.1 live smoke depends only on 2.4 and sits in Epic 5; the entire
   OpenRouter mocked surface (1.2/1.3/2.4/goldens) is built on assumed field names.
   Cutover (5.3) does not depend on 5.1 → OpenRouter can ship having never made a real
   call. *Rec:* pull live smoke forward to right after 2.4; make the OpenRouter-
   enabling part of 5.3 depend on 5.1, or state shipping-live-unvalidated as accepted.
3. **[medium] `quality` cross-provider impedance mismatch unresolved.** Go: `quality`
   selects a Gemini *model*. OpenRouter `/api/v1/images`: `quality` is a native
   orthogonal param + separate model slug. `--provider openrouter --quality high` has
   no defined meaning under the shared model (2.2). *Rec:* 2.2/2.5 + SPEC §5 must
   specify per-provider `quality` semantics; add a precedence-matrix case.
4. **[medium] Skills embed + tree-hash port understated; live cutover hazard.**
   `skills`/`doctor` compute a tree hash over the Go `embed.FS` and compare deployed
   vs embedded marker hashes. (a) the Rust embed mechanism (`include_dir`/`rust-embed`
   + matching tree-hash + marker injection) is load-bearing work not called out as a
   task; (b) at cutover Go-installed skills report `outdated`/`modified` under Rust
   unless the hash algorithm is reproduced OR cutover re-runs `skills upgrade`. 5.3
   omits this. *Rec:* add an embed-infrastructure sub-task; add "re-run `naba skills
   upgrade` after cutover" to 5.3 (or require + test hash-algo reproduction).
5. **[low] Config migration-on-every-Load rewrites the file and drops YAML comments.**
   Schema change is additive-optional; a serde round-trip discards comments/reorders
   keys even once. *Rec:* confirm whether any rewrite is needed for additive-optional
   fields (missing→defaults on read = zero migration); if retained, note comment loss
   in SPEC §10 as accepted.
6. **[low] Multi-key default silently switches existing Gemini users to OpenRouter.**
   Both keys + no config default → OpenRouter. A current Gemini user who adds
   `OPENROUTER_API_KEY` gets rerouted. Operator-sanctioned but a behavior change.
   *Rec:* surface in SPEC §5 + skill docs (5.2) as an intentional precedence outcome.

## Missing

- **`list_images` MCP tool has no owning issue.** It is the 8th registered tool
  (`server.go:43`), MCP-only, lists the output dir. 4.1–4.3 don't cover it. *Rec:* add
  its filesystem-listing logic explicitly to 4.4's scope.
- **Skills filesystem-state test coverage.** `skills install/upgrade/remove` produce
  filesystem trees via `--target`; the plan describes no filesystem-state assertions.
  *Rec:* add cases (install to tmp `--target`, assert files+marker; upgrade prunes
  stale; remove clears).
- **Build/release + version-injection tooling.** Go uses ldflags; the Rust equivalent
  (build.rs / compile-time env) + release/cross-compile pipeline are only glancing in
  5.3. *Rec:* name the version-injection mechanism as a concrete task.
- **Composites regression-unprotected.** storyboard/batch/brand-kit correctly out of
  binary scope, but nothing catches breakage if composed primitives shift. *Rec:*
  one-line SPEC note that they rely on primitive-command goldens transitively.

## Gate Assessment

Two active gates + one resolved, no gate-inflation. Start Gate standard. Provider-
layer gate correctly RESOLVED (BESPOKE, compiler-validated). live-keys gate condition
+ test valid and executable; scoping right — **but** downstream wiring too loose: 5.3
cutover doesn't depend on it, so it can stay permanently unresolved while OpenRouter
ships (see Concern 2). Tighten the edge or state the accepted risk.

## Upstream Assessment

Upstream Issues table empty ("scan pending, Phase 1.4"). For a rewrite of a shipping
tool this is a real gap: open naba issues/beads (bugs, deferred work) need dispositions
(carry-forward / fixed-by-rewrite / superseded) reconciled against this plan — a full
rewrite is exactly where "superseded" dispositions accumulate. *Rec:* complete the
upstream scan before approval.

## Operator Resolutions

| # | Concern (sev) | Resolution | Status |
|:--|:--|:--|:--|
| 1 | Epic 2 skeleton→SPEC edge (med) | Added `2.1 depends-on 1.1` (contract-first: skeleton needs the SPEC-defined command/flag/enum surface). | resolved |
| 2 | Live smoke last/skippable (med) | Live smoke pulled forward to new **Issue 2.6** (depends-on 2.4), de-risking the mocked surface before it freezes; cutover **5.3 now depends-on 2.6** for the OpenRouter-enabling portion; keys-never-arrive → explicitly accepted risk on 2.6/5.3/gate. | resolved |
| 3 | `quality` cross-provider semantics (med) | 2.2 carries a raw `quality` resolved per-provider (Gemini→model tier, OpenRouter→native `/api/v1/images` param); 2.5 routes it; SPEC §5 specifies semantics; 1.3 adds a per-provider `quality` precedence case. | resolved |
| 4 | Skills embed/tree-hash + cutover (med) | New **Issue 4.0** owns skill-embed infra (embed + tree-hash reproduction OR accepted divergence + marker injection); 5.3 adds documented post-cutover `skills upgrade`. | resolved |
| 5 | Migration rewrites file/comments (low) | 3.2 defaulted to **zero-rewrite** (additive-optional keys → defaults on read; file untouched); rewrite only if genuine structural migration needed, with accepted comment-loss noted in SPEC §10. | resolved |
| 6 | Multi-key silent switch (low) | Documented as intentional precedence outcome in SPEC §5 (1.1) and skill/README/AGENTS docs (5.2). | resolved |
| M1 | `list_images` tool unowned | Added explicitly to Issue 4.4 scope (MCP-only output-dir listing tool, 8th tool). | resolved |
| M2 | Skills filesystem-state tests | 4.3 now covers install/upgrade/remove filesystem side effects via tmp `--target` (files+marker, prune, clear). | resolved |
| M3 | Version-injection/release tooling | 2.1 stands up version-injection (build.rs/compile-time env replacing Go ldflags). | resolved |
| M4 | Composites regression note | 1.1 SPEC gets a one-line note that storyboard/batch/brand-kit are skill-layer, protected transitively via primitive-command goldens. | resolved |
| U1 | Upstream scan pending | Scan complete: GitHub #1–3 closed, no open issues; only local bead naba-a3a → include/carry-forward (per-provider image-size validation), added to Upstream Issues table. | resolved |

**Final status:** frozen — all concerns resolved 2026-07-11.
