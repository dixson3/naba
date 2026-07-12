# Plan: Rewrite naba in Rust with multi-provider support (Gemini + OpenRouter)

**ID:** plan-004-james-dixson-9a7b16
**Author:** james-dixson
**Created:** 2026-07-11
**Status:** reconciling
**Epic:** naba-mol-4dz
**Fingerprint:** e1658264d0e6f8edeaab1d00723f0768374b2bee14d03127f2b317b914b3cda5
**Phase log:**
- 2026-07-11 scoping: initial scope captured
- 2026-07-11 investigating: 4 investigations identified (INV-1 provider-layer spike, INV-2 OpenRouter image API, INV-3 Rust parity ecosystem, INV-4 regression-suite strategy)
- 2026-07-11 drafting: 4 investigations complete; bespoke provider layer, /api/v1/images, concrete OpenRouter default resolved; synthesizing plan
- 2026-07-11 review: plan v1 drafted (5 epics, 20 issues); entering review
- 2026-07-11 review: pass-2 red-team APPROVE (pass-1 REVISE resolutions verified)
- 2026-07-11 ready-for-approval: ready-check green — last red-team APPROVE (pass-2) + audit pass
- 2026-07-11 approved: operator approved
- 2026-07-11 intake: epic naba-mol-4dz poured
- 2026-07-11 executing: start gate resolved
- 2026-07-12 reconciling: DAG drained (20/20 issues); merge-back

## Objective

Port the naba image CLI from Go to Rust (full feature parity) and, in the same
effort, introduce a **provider abstraction** so images can be generated through
either **Gemini** (current) or **OpenRouter** (new). Configuration gains a
**default provider** and **default model**; both are selectable on the CLI. When
a model is specified on the CLI, its provider must also be specified. Whether the
Rust provider layer is built bespoke or on top of a general Rust multi-provider
aggregation crate is decided by an **investigation spike** (below), not assumed.

## Motivation

naba today is a Go CLI with a single, hardcoded Gemini provider — `gemini.Client`
is instantiated identically in ~9 call sites with no provider abstraction (Gemini's
`x-goog-api-key` header, `:generateContent` URL shape, and request/response structs
are baked in). The operator wants (a) to reach the far larger model catalog behind
**OpenRouter** (200+ models, including OpenRouter's `auto` router) in addition to
Gemini, and (b) to move naba to **Rust**. Rather than bolt a second provider onto
the Go codebase and then rewrite, this plan does the rewrite and the provider
abstraction together: the Rust port introduces the provider seam that Go never had.

## Scope Decisions (from scoping Q&A, 2026-07-11)

1. **Language / rewrite (Q: Language → "Rust rewrite is in scope"; Q: Rewrite scope
   → "Full parity port").** naba is ported Go → **Rust** with **full feature
   parity**. **CORRECTED by INV-4:** the binary has **12 real command groups**, not
   the 15 first listed — `generate`, `edit`, `restore`, `icon`, `pattern`,
   `diagram`, `story`, `config` (get/set), `doctor`, `skills`
   (install/upgrade/remove/status), `mcp`, `version`. `storyboard`, `batch`, and
   `brand-kit` are **skill-layer composites** (the `/naba` skill orchestrates
   multiple real CLI calls), NOT subcommands — they are out of the binary parity
   surface (pinned, if at all, at the skill/prompt layer). The Rust binary is a
   drop-in replacement for these 12 groups + the MCP server; nothing real is dropped.

2. **Provider parity (Q: Workflow scope → "Full parity").** Every subcommand routes
   through the **selected provider**. OpenRouter must back not just text-to-image
   `generate` but the image-input paths (`edit`, `restore`) and every composite
   command. Per-provider capability differences (image size / aspect ratio /
   quality tiers) must be handled per provider, not assumed uniform.

3. **Provider layer build approach (Q: Build approach → "Decide after an
   investigation spike").** Do NOT pre-commit to bespoke vs library. An
   investigation spike prototypes the realistic Rust crate candidate(s) —
   `rath-rs` (capability `images` module + `openrouter`/`gemini`/`fal` adapters,
   `ModelUrl` locators) and `edgequake-llm` (broad, but its image backends do NOT
   include OpenRouter) — against naba's real requirements (image gen through BOTH
   Gemini and OpenRouter, `imageConfig` knobs, image *input* for edit/restore),
   versus a bespoke Rust provider layer (two thin HTTP clients + a provider trait +
   a selector factory). The spike's verdict picks the approach.

4. **Default provider / model resolution (Q: Default provider → environment-driven).**
   The effective provider/model is resolved by **which API keys are present in the
   environment**, with config and CLI overriding:
   - Precedence: **CLI flags > config (`provider`/`model`) > env-key-based
     autodetect > built-in fallback.**
   - Env autodetect: if only `GEMINI_API_KEY` is set → Gemini (+ Gemini default
     model); if only `OPENROUTER_API_KEY` is set → OpenRouter. **REVISED by INV-2:**
     for the multiple-keys-no-config-default case, `openrouter/auto` **cannot
     generate images** (it is a text-only router), so it cannot be the default.
     Replacement pending operator decision (see "Scope decision #4 revision" below).
   - CLI rule (operator-stated, retained): **if `--model` is given on the CLI,
     `--provider` must also be given** (a model name alone is ambiguous across
     providers).

   **Scope decision #4 revision (RESOLVED, operator 2026-07-11).** Multiple keys +
   no config default → **OpenRouter with a concrete default image model slug**,
   `google/gemini-3.1-flash-image-preview` (OpenRouter's own documented default);
   naba may later resolve the default via OpenRouter's image-models discovery
   endpoint. `openrouter/auto` is retained only for any future text path, never for
   image generation.

5. **OpenRouter API surface (RESOLVED, operator 2026-07-11).** The bespoke
   OpenRouter provider targets the **dedicated `POST /api/v1/images`** Unified Image
   API (not legacy chat-completions+modalities): `aspect_ratio`, `resolution`
   (512/1K/2K/4K), native `quality`, and `input_references[]` for edit/restore map
   near-1:1 onto naba's `imageConfig`. A live-key smoke test confirms exact response
   field names during implementation.

6. **UX contract (Q: UX contract → "Improve where sensible" + explicit riders).**
   The rewrite is NOT bound to a byte-identical contract, but with hard riders:
   - **Capture the current UX in a `SPEC.md`** — the authoritative description of
     subcommands, flags, config schema, exit codes (1/2/3/4/5/10), and JSON output
     shapes as they exist in Go naba today. This is the port's source of truth.
   - **Generate a comprehensive UX regression test suite BEFORE implementing the
     port** — executable tests that pin the current behavior, which the Rust port
     must pass (allowing for the sanctioned improvements below).
   - **Config auto-migration**: the Rust binary auto-migrates existing
     `~/.config/naba/config.yaml` files to whatever the new config schema is (new
     `provider`/`model` keys, any restructuring) without manual intervention.
   - Sanctioned improvements: add the `provider` flag/config key and any config
     restructuring needed for multi-provider; update the `naba` skill + docs to
     match. Improvements must be captured in `SPEC.md` and covered by the suite.

## Open Investigations (INVESTIGATE phase)

- **INV-1 — Rust provider-layer spike (decides approach #3).** Prototype
  `rath-rs` and `edgequake-llm` vs bespoke Rust for: image gen through Gemini AND
  OpenRouter; `imageConfig` (aspectRatio + imageSize) + quality tier; image input
  (edit/restore). Assess crate maturity, license, dependency weight, and whether
  they expose naba's knobs. Output: a bespoke-vs-library recommendation.
- **INV-2 — OpenRouter image API capabilities.** Map OpenRouter's image-generation
  wire format (chat-completions + `modalities:["image","text"]`, Bearer auth),
  whether it exposes aspect ratio / image size / quality knobs, image-input support
  for edit, and the behavior of the `auto` model for image generation.
- **INV-3 — Rust parity ecosystem.** Confirm mature Rust crates exist for the
  parity-critical pieces: CLI (`clap`), an **MCP server SDK** comparable to Go's
  `mark3labs/mcp-go`, config + auto-migration, image encode/write/preview. MCP
  server parity is the highest parity risk.
- **INV-4 — Regression-suite strategy.** Decide how to capture current Go UX as an
  executable suite the Rust port must pass: golden CLI tests (args → stdout/stderr/
  exit code), mocked provider HTTP, JSON-output snapshots, and the exit-code matrix.

## Upstream Issues

Scan complete (2026-07-11). GitHub issues #1–#3 are all **closed** (homebrew-tap
work); no open GitHub issues. The only open tracked item is a local bead.

| Issue | Title | Disposition | Notes | Resolved By |
|:--|:--|:--|:--|:--|
| naba-a3a (local bead) | imageSize 512 rejected by gemini-3.1-flash-image (per-model size support) | include (carry-forward) | Root cause is that `ValidImageSizes` is validated globally but 512 is model-dependent. The rewrite's per-provider/per-model capability handling (scope #2) is the natural fix: image-size validation becomes provider/model-aware rather than a single global list. | Issue 2.3 (Gemini per-model size handling) + SPEC §4 validation clause; carried into the OpenRouter provider's per-model knob handling (2.4). |

## Investigation Findings

All four investigations complete (findings/exp-001..004). Summary:

- **INV-1 (provider layer) → BESPOKE, compiler-validated.** No Rust crate exposes
  OpenRouter image generation behind a unified abstraction (confirmed via source,
  the compiler `E0599`, and a crate scan). `rath-rs` is a FAL-only images shim
  (disqualified); `edgequake-llm` covers the Gemini side well but ZERO of the
  OpenRouter side while pulling 180 deps. Bespoke = ~500–800 LOC over the
  reqwest/serde/tokio baseline (Go layer is 477 LOC) and preserves naba's
  near-zero-dep posture. `Provider` trait + `GeminiProvider` + `OpenRouterProvider`
  + selector factory. (Only `openrouter-rs` has a real OpenRouter `/images` surface —
  optional isolated dep for the OpenRouter half, not adoption.)
- **INV-2 (OpenRouter image API) → target the dedicated `POST /api/v1/images`.** It
  maps naba's `imageConfig` ~1:1 (`aspect_ratio`, `resolution` 512/1K/2K/4K, native
  `quality`), supports image input via `input_references[]` (edit/restore), and
  errors map cleanly to naba's exit codes. **CRITICAL: `openrouter/auto` does NOT
  do image output** — scope #4's `auto` default is invalid and revised below.
- **INV-3 (Rust parity) → FULL parity realistic.** Every parity-critical concern has
  a mature crate; MCP server via the official `rmcp` (feature `server`). One early
  decision: **async provider layer** (async reqwest + tokio, CLI via
  `#[tokio::main]`) shared by CLI + MCP. YAML: use `serde_norway`/`yaml_serde`,
  **forbid `serde_yml` (RUSTSEC-2025-0068)**; migration = plain serde round-trip +
  backup. Preview = `open` crate (OS viewer launch, not terminal render). Image IO =
  `std::fs`.
- **INV-4 (regression suite) → Python+pytest black-box harness**, `$NABA_BIN`
  selects Go (golden capture) or Rust (replay); `pytest-httpserver` mock via
  `GEMINI_BASE_URL` + a NEW `OPENROUTER_BASE_URL`; PTY subset for TTY/human cases;
  PATH-stub for `--preview`; separate MCP-protocol harness; Rust-only migration
  tests. SPEC.md with CI-enforced clause↔test traceability.

Library landscape pre-scans: `references/rust-multiprovider-libraries.md` and
`references/go-multiprovider-libraries.md`.

## Approach

A **spec-and-test-first Go→Rust rewrite** with a bespoke provider abstraction. The
ordering is fixed by the UX riders: pin the contract before touching Rust, so the
port is validated against an executable target captured from today's Go binary.

**Sequencing (five stages, each an epic):**

1. **Pin the contract** — author `SPEC.md` (the authoritative UX: 12 command groups,
   flags/defaults, config schema + precedence, exit-code matrix incl. the
   cobra-parse→1 subtlety, JSON shapes, error strings, MCP surface) and stand up the
   **Python+pytest black-box harness**, then **capture goldens from the Go binary**
   (`--update-golden`). This happens entirely before any Rust exists and is the
   drop-in-replacement acceptance target.
2. **Rust skeleton + provider abstraction** — new Rust crate: `clap` CLI skeleton
   (all 12 command groups, global flags, TTY autodetect via `IsTerminal`), the
   **async** `Provider` trait (`generate`, `generate_with_image`, `list_models`) with
   `GeminiProvider` (port of the 477-LOC Go client: `x-goog-api-key` +
   `:generateContent` + `imageConfig`) and `OpenRouterProvider` (bespoke, targeting
   `POST /api/v1/images`), plus the **selector factory** (CLI > config > env-key
   autodetect > fallback; `--model` requires `--provider`; multi-key → OpenRouter
   `google/gemini-3.1-flash-image-preview`). Add `OPENROUTER_BASE_URL` (mirroring
   `GEMINI_BASE_URL`) for mockable tests. CLI runs via `#[tokio::main]`.
3. **Config + migration + output** — YAML config (`serde_norway`/`yaml_serde`; NOT
   `serde_yml`) with the new `provider`/`model` keys; **auto-migration** (serde
   round-trip + `.bak` backup, idempotent, no data loss); the output layer
   (`std::fs` write with extension-reconciliation, JSON `Result`/array shapes, `open`
   preview, exit-code enum → `process::exit`).
4. **Command parity + MCP server** — implement each command group's behavior against
   the harness (generate/edit/restore/icon/pattern/diagram/story/config/doctor/skills/
   version), then the **MCP server** via `rmcp` (feature `server`): the 8 tools +
   `file://` resource template, schema-verified against Go's `tools.go` (start with
   the one-tool schema spike). Iterate until the Rust binary passes the full suite
   (Go-captured goldens + Rust-only migration + MCP-protocol tests).
5. **Docs, skill, cutover** — update the `naba` skill, README, and AGENTS.md for the
   new `--provider` surface and the sanctioned divergences (help text, skill hashes,
   version); wire the regression suite into CI (both `$NABA_BIN` targets +
   SPEC↔test traceability check); replace the Go build with the Rust binary.

**Live-key smoke tests** (small, gated on operator-supplied keys) confirm the two
runtime unknowns INV-2 flagged: exact `/api/v1/images` response field names and that
`openrouter/auto` is rejected for image gen. These validate but do not block the
API-surface design.

## Epics

### Epic 1: Contract capture — SPEC.md + regression harness (Go goldens)
- Issue 1.1: Author `SPEC.md` with stable clause IDs (`SPEC-<AREA>-NNN`) covering all
  12 command groups, global flags + TTY autodetect, config schema + precedence
  chains (model, imageConfig, api_key, the CLI-vs-MCP output-dir asymmetry),
  quality→model map, validation enums, exit-code matrix (incl. cobra-parse→1), JSON
  Result/array + doctor envelope shapes, verbatim error strings, MCP surface, and the
  enumerated sanctioned-divergence zones. **Provider-layer clauses (§5)** must specify
  **per-provider `quality` semantics** (Gemini: quality→model tier; OpenRouter:
  native `quality` param on `/api/v1/images`, model slug separate — resolve what
  `--provider openrouter --quality high` means) and **document the multi-key default
  as an intentional precedence outcome** (adding `OPENROUTER_API_KEY` reroutes an
  existing Gemini user to OpenRouter — Concern 6). **Config-migration clauses (§10)**
  cover the migration policy incl. any accepted YAML-comment loss (Concern 5). Add a
  one-line note that storyboard/batch/brand-kit are skill-layer composites protected
  only transitively via primitive-command goldens (M4).
- Issue 1.2: Stand up the `tests/parity/` Python+pytest harness — `$NABA_BIN`
  runner, per-case tmp CWD + `NABA_CONFIG_DIR`, nondeterministic-field normalizer
  (`elapsed_ms`, timestamped paths, version), `pytest-httpserver` provider mock
  (Gemini `:generateContent` + `/models`; OpenRouter `/api/v1/images`) returning a
  canned image, request-recording for outgoing-JSON assertions, PTY runner mode,
  `--preview` PATH-stub. depends-on: 1.1
- Issue 1.3: Author the case table (`cases/*.yaml`) — per-command cases, the
  exit-code matrix, the precedence matrix (**incl. a per-provider `quality` case**) —
  and **capture normalized goldens from the Go binary** (`--update-golden`); commit
  `golden/`. depends-on: 1.2
- Issue 1.4: MCP-protocol harness (`test_mcp.py`) via the MCP Python SDK: `initialize`
  → `tools/list` (assert 8 tools, params/enums/defaults) → `tools/call` (mock
  provider) → `resources/read` (`file://`); capture Go-side expectations. depends-on: 1.2

### Epic 2: Rust skeleton + provider abstraction
- Issue 2.1: Scaffold the Rust crate (Cargo, MIT license/attribution, `clap` derive
  CLI with all 12 command groups + global flags + `config get/set`/`skills`
  subcommands, `IsTerminal` TTY autodetect, `#[tokio::main]`). Wire the exit-code
  error enum → `process::exit`. The command/flag/enum surface is defined by SPEC §3,
  so this depends on the authored SPEC (contract-first). Also stand up
  **version-injection** (build.rs / compile-time env for Version/Commit/Date,
  replacing Go ldflags — M3). depends-on: gate-start-gate, 1.1
- Issue 2.2: Define the async `Provider` trait + shared request/response model
  (`ImageConfig{aspect, size}`, model, input image, and a **`quality` field whose
  interpretation is per-provider** — Gemini maps it to a model tier, OpenRouter passes
  it as the native `/api/v1/images` `quality` param; the trait carries the raw value
  and each impl resolves it). depends-on: 2.1
- Issue 2.3: `GeminiProvider` — port the Go Gemini client (`x-goog-api-key`,
  `:generateContent`, `imageConfig` in `generationConfig`, `responseModalities`,
  `generate_with_image` for edit/restore, `list_models` for doctor), honoring
  `GEMINI_BASE_URL`. Validated against Epic-1 Gemini goldens + recorded-request
  assertions. depends-on: 2.2
- Issue 2.4: `OpenRouterProvider` — bespoke client against `POST /api/v1/images`
  (Bearer auth, `aspect_ratio`/`resolution`/`quality`, `input_references[]` for
  edit/restore, base64 `data[].b64_json` decode, error→exit mapping incl.
  moderation-403→content-policy, `Retry-After`), honoring a new `OPENROUTER_BASE_URL`.
  depends-on: 2.2
- Issue 2.5: Provider selector factory — CLI > config > env-key autodetect > fallback;
  `--provider` flag/config key; `--model` requires `--provider` (usage-exit
  otherwise); multi-key-no-default → OpenRouter `google/gemini-3.1-flash-image-preview`.
  Resolve the per-provider `quality` mapping here (route the raw `--quality` value to
  Gemini model-tier vs OpenRouter native param, per SPEC §5). depends-on: 2.3, 2.4
- Issue 2.6: **Live-key smoke (pulled forward — Concern 2)** — with an operator key,
  confirm the two INV-2 runtime unknowns BEFORE the full mocked surface is frozen on
  them: exact `/api/v1/images` response field names (`data[].b64_json`/`media_type`)
  and that `openrouter/auto` is rejected for image gen. Reconcile any drift into 2.4 +
  SPEC §5 + the OpenRouter mock/goldens. **Blocked by Capability Gate: live-keys.** If
  keys never arrive, this stays blocked and shipping OpenRouter live-unvalidated is the
  explicitly accepted risk (recorded here + on 5.3). depends-on: 2.4

### Epic 3: Config, migration, and output layer
- Issue 3.1: YAML config load/save (`serde_norway`/`yaml_serde`; **forbid
  `serde_yml`**) at `~/.config/naba/config.yaml` (honoring `NABA_CONFIG_DIR`), with
  the new `provider`/`model` keys; `config get`/`config set` + validation. depends-on: 2.1
- Issue 3.2: Config auto-migration (Concern 5). The schema change is
  **additive-optional** (`provider`/`model` are new optional keys), so the default is
  **zero-rewrite**: absent keys resolve to defaults on read, leaving the user's
  hand-edited `config.yaml` (and its comments) untouched. A file rewrite is performed
  ONLY if a genuine structural migration is later required; if so, it does a serde
  round-trip with `.bak` backup, is idempotent and graceful on
  empty/missing/malformed/already-new, and its accepted YAML-comment loss is noted in
  SPEC §10. Validated by the Rust-only migration tests. depends-on: 3.1
- Issue 3.3: Output layer — `std::fs` write with JPEG/PNG extension reconciliation +
  dedup naming, JSON `Result`/array + `doctor` envelope, `open`-crate preview, the
  CLI-vs-MCP output-dir asymmetry (`NABA_OUTPUT_DIR` for MCP only). depends-on: 2.1

### Epic 4: Command parity + MCP server
- Issue 4.0: **Skill-embed infrastructure (M4/Concern 4).** Embed the skill tree into
  the Rust binary (`include_dir`/`rust-embed`), reproduce Go's tree-hash algorithm
  (`EmbeddedTreeHash`/`DeployedTreeHash`) and marker injection (`InjectMarker`) so
  `doctor`/`skills status` hash comparisons behave identically — OR consciously accept
  a hash-format divergence and require a post-cutover `skills upgrade` (see 5.3). This
  is load-bearing work `doctor`/`skills` depend on. depends-on: 2.1
- Issue 4.1: Implement the core image commands (generate/edit/restore) with prompt
  enrichment + imageConfig/quality resolution, to green against Epic-1 goldens.
  depends-on: 2.5, 3.3
- Issue 4.2: Implement the composite-prompt commands (icon/pattern/diagram/story) with
  their per-command flags/defaults and deterministic prompt builders; `story` array
  output + `--steps` 2-8 validation. depends-on: 4.1
- Issue 4.3: Implement `doctor` (check semantics + JSON envelope, mocked liveness) and
  `skills` (install/upgrade/remove/status); pin semantics not Go-specific hashes.
  Cover the **filesystem side effects** (M2): install to a tmp `--target` asserts files
  + injected marker present; upgrade prunes stale entries; remove clears the tree.
  depends-on: 4.1, 4.0
- Issue 4.4: MCP server via `rmcp` (feature `server`) — the one-tool schema spike
  first, then **all 8 tools** (incl. `list_images`, the MCP-only output-dir listing
  tool with no CLI counterpart — M1) + `file://` resource template over stdio, green
  against the MCP-protocol harness. depends-on: 4.1, 1.4
- Issue 4.5: Full-suite green — the Rust binary passes the entire pytest parity suite
  (Go goldens + migration + MCP) under `$NABA_BIN=<rust>`. depends-on: 4.2, 4.3, 4.4, 3.2

### Epic 5: Docs, skill, cutover
(The live-key smoke moved to Issue 2.6, pulled forward to de-risk the mocked surface —
Concern 2.)
- Issue 5.2: Update the `naba` skill, README, AGENTS.md for the `--provider` surface,
  env-key autodetect (**incl. the intentional multi-key → OpenRouter reroute** —
  Concern 6), sanctioned divergences, and the migration behavior. depends-on: 4.5
- Issue 5.3: Wire the regression suite into CI (both `$NABA_BIN` targets +
  SPEC↔test traceability check), cut the build over to the Rust binary, and — because
  the Rust skill-embed hash may differ from Go's — **re-run `naba skills upgrade` as a
  documented post-cutover step** (Concern 4) so existing installs don't report
  `outdated`/`modified`. The OpenRouter-enabling portion of cutover **depends on the
  live-key smoke (2.6)**; if keys never arrive, shipping OpenRouter live-unvalidated is
  the explicitly accepted risk. depends-on: 4.5, 5.2, 2.6

## Gates

### Start Gate (mandatory)
- Type: human
- Approvers: operator

### Capability Gate: provider-layer decision (INV-1) — RESOLVED
- Type: human
- Condition: INV-1 spike complete; bespoke-vs-library approach chosen and recorded.
- Resolution: **BESPOKE** (findings/exp-001). Compiler-validated: no Rust crate
  provides OpenRouter image gen behind a unified abstraction. Provider trait +
  Gemini/OpenRouter impls over reqwest/serde/tokio.

### Capability Gate: live-keys (blocks Issue 2.6)
- Type: human
- Condition: operator supplies a live `OPENROUTER_API_KEY` (and `GEMINI_API_KEY`) so
  the smoke tests can confirm the two runtime unknowns INV-2 flagged.
- Test: `test -n "$OPENROUTER_API_KEY"` (and the smoke run returns 200 + a decodable
  image from `POST /api/v1/images`).
- Blocks: Issue 2.6 (live-key smoke, pulled forward) and — transitively — the
  OpenRouter-enabling portion of cutover (5.3 depends-on 2.6). All API-*surface* work
  (Epic 2/4) proceeds against mocks without it — this gate blocks only the live
  confirmation, not the port. If keys never arrive, shipping OpenRouter
  live-unvalidated is the explicitly accepted risk (recorded on 2.6 and 5.3).
- Instructions: export the keys in the execution environment, or hand naba the smoke
  run to perform interactively.

## Risks & Mitigations

- **Full Go→Rust parity is a large effort.** Reproduces 12 command groups + MCP +
  config + doctor. Mitigation: SPEC.md + the Go-captured regression suite are the
  executable acceptance target; the port is green-driven, epic by epic. (Effort is
  the dominant risk — the rewrite is large regardless of the multi-provider add.)
- **MCP server parity.** Resolved favorably (INV-3): the official `rmcp` crate covers
  server mode + tools + resource templates. Residual: `rmcp` is tokio-only vs a
  synchronous CLI. Mitigation: adopt an **async provider layer** shared by CLI + MCP
  (decided in Approach); one-tool schema spike (4.4) before all 8.
- **`openrouter/auto` cannot generate images** (INV-2). Mitigation: scope #4 revised
  to a concrete default slug (`google/gemini-3.1-flash-image-preview`); `auto`
  reserved for any future text path only.
- **OpenRouter runtime wire behavior unconfirmed** (exact response field names;
  `auto` image rejection; per-model knob honoring). Mitigation: live-key smoke tests
  (2.6) behind the live-keys gate; the `/api/v1/images` surface design is conclusive
  from docs (INV-2) and does not block mocked implementation.
- **YAML crate hazard.** `serde_yml` is unsound (RUSTSEC-2025-0068). Mitigation: plan
  forbids it; use `serde_norway`/`yaml_serde`.
- **Config auto-migration must not lose data.** Mitigation: serde round-trip + `.bak`
  backup + idempotency, covered by Rust-only migration tests (3.2) with edge cases.
- **Sanctioned-divergence false failures.** Help text (cobra→clap), skill integrity
  hashes (Go embed→Rust embed), and version strings cannot be byte-identical.
  Mitigation: SPEC enumerates these zones; the suite pins inventory/semantics via
  regex/contains, not full snapshots.

## Success Criteria

- Rust naba is a drop-in replacement for Go naba across the **12 real command groups**
  + MCP server + config + doctor + version, passing the full Python+pytest parity
  suite (Go-captured goldens + MCP-protocol + Rust-only migration) under
  `$NABA_BIN=<rust>`.
- `SPEC.md` authoritatively describes the UX with stable clause IDs and a CI-enforced
  SPEC↔test traceability check; sanctioned divergences are enumerated.
- Images generate through either Gemini or OpenRouter, selected by CLI
  `--provider`/`--model`, config defaults, or env-key autodetect — multiple keys +
  no config default → **OpenRouter with `google/gemini-3.1-flash-image-preview`**
  (never `auto` for images).
- `--model` on the CLI requires `--provider` (usage-exit otherwise).
- The OpenRouter provider targets `POST /api/v1/images`; live-key smoke tests confirm
  response field names and `auto` image rejection.
- Existing `~/.config/naba/config.yaml` files auto-migrate cleanly (no data loss,
  original backed up, idempotent).
- naba is near-zero-dependency-preserving: the bespoke provider layer avoids the
  150–180 transitive deps the surveyed aggregation crates would impose.
