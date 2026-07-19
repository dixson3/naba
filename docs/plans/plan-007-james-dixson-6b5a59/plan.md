# Plan: Consistent multi-provider config + provider ecosystem for naba (per-provider defaults, api-key resolution, AWS Bedrock provider, naba provider/models commands, --json everywhere, MCP lazy-loading skills) with comprehensive web/ docs (full config schema, separate skills + mcp pages, lifecycle coverage)

**ID:** plan-007-james-dixson-6b5a59
**Author:** james-dixson
**Created:** 2026-07-19
**Status:** complete
**Epic:** naba-mol-a7v
**Fingerprint:** e05af01f94a0082ccf3382e0ff1b5e24aac18ef69235e301beb65bc8a537139c
**Phase log:**
- 2026-07-19 scoping: initial scope captured
- 2026-07-19 investigating: 3 experiments: MCP internals, config/migration, Bedrock SDK
- 2026-07-19 drafting: synthesizing plan from 3 findings
- 2026-07-19 review: plan v1 presented (6 epics incl. agent-tools SPEC)
- 2026-07-19 review: red-team pass 2: APPROVE (pass-1 REVISE concerns resolved; Epic 6 split to plan-008)
- 2026-07-19 ready-for-approval: ready-check green — last red-team APPROVE + audit pass
- 2026-07-19 approved: operator approved
- 2026-07-19 intake: epic naba-mol-a7v poured
- 2026-07-19 executing: start gate resolved (auto-migrate confirmed); bedrock-transport gate resolved: thin reqwest
- 2026-07-19 reconciling: post-execution reconciliation (all 5 epics landed on execute branch)
- 2026-07-19 complete: plan complete — 5 epics merged to main + validated; push pending operator authorization

## Objective
Consistent multi-provider config + provider ecosystem for naba (per-provider defaults, api-key resolution, AWS Bedrock provider, naba provider/models commands, --json everywhere, MCP lazy-loading skills) with comprehensive web/ docs (full config schema, separate skills + mcp pages, lifecycle coverage)

## Motivation
naba today has a **flat, single-provider-centric** config (`api_key` is Gemini-only; one
`model`; one `provider`) and only two providers (Gemini, OpenRouter). As the tool grows, this
is inconsistent: there is no way to keep a default model *per provider*, api-key resolution is
special-cased per provider (a Gemini config key + two hard-coded env vars) rather than a
uniform scheme, and there is no first-class way to discover which providers are configured or
what models a provider offers. There is also no AWS Bedrock support despite it being a major
image-model host. Separately, the CLI's machine-readable output is uneven, and the MCP server
front-loads everything rather than letting an agent lazily pull tool/skill detail. Finally, the
website docs (and the project README) under-document configuration, skills, and MCP. This plan
makes the provider/config surface **consistent and discoverable**, adds **Bedrock**, makes
**`--json`** universal, gives MCP a **lazy-loading** shape, and brings the **web docs + README**
into full, in-sync coverage of the tool and its lifecycle.

## Upstream Issues
| Issue | Title | Disposition | Notes | Resolved By |
|:------|:------|:------------|:------|:------------|
| #7 | Cut the first cargo-dist release of naba | exclude | Unrelated (release mechanics). Relevant only in that pre-1.0 status permits a breaking config schema change. | — |
| #8 | Complete execution of plan-006 (website) | exclude | plan-006 tracker; complete + deployed. Should be closed as housekeeping (separate from this plan). | — |

## Scoping Decisions
- **Config migration (SELECTED — auto-migrate; confirmed at the Start Gate):** auto-migrate old
  flat config → new nested schema on load, writing a `config.yaml.bak` first (extends the existing
  `migrate_if_needed` pattern), idempotent. No separate migration gate — strategy confirmation is
  folded into the **Start Gate** (operator decision, red-team pass-1 #1). Per-key mapping is
  specified in Issue 1.3. **Each provider designates its own default model** — a firm requirement.
- **New config schema (nested):** a top-level `default_provider`, plus a `providers` map keyed
  by provider name; each entry carries `model` (per-provider default model) and an api-key
  source resolved in precedence: inline `api-key` → custom env var `api-key-envvar` → the
  provider's conventional default env var (e.g. `GEMINI_API_KEY`). Image defaults (aspect,
  resolution, quality, output dir) stay top-level.
- **AWS Bedrock:** official `aws-sdk-bedrockruntime`. Support **both** a Bedrock API key
  (bearer) **and** an AWS profile / default credential chain (SigV4). Curated model set
  enumerable via the existing `Provider::list_models`. Unit tests required.
- **New commands:** `naba provider` (list providers + which have valid credentials);
  `naba models [--provider <name>]` (list a provider's models; default provider if unspecified,
  building on the existing `Provider::list_models` trait method).
- **`--json` universal:** every subcommand accepts `--json` and emits a documented envelope
  (audit which commands currently lack one; keep the existing pipe-auto-enable, SPEC-GLOBAL-003).
- **MCP lazy-loading:** expose skill/subcommand guidance as **MCP resources** (or a compact
  index tool) so clients list cheaply and fetch full instructions on demand; keep tool schemas
  lean. Inject an mcp-friendly version of the skills.
- **Docs:** Config page shows the **complete config-file schema**; split **Skills** and **MCP**
  into their own pages (Skills: subcommands + implicit-trigger emphasis, claude-code default
  surface vs generic `.agents/skills`, user- vs project-scope; MCP: all tools + lazy-loading);
  docs cover all capabilities + lifecycle. **Also sync the project README install/setup with
  the web install/config docs** (operator request) and add a DRIFT-CHECK edge to keep them in
  sync.
- **Portable "agent-tools" SPEC (operator request):** capture the reusable pattern naba embodies
  as a tool-agnostic spec — **three pillars**: (1) **skills self-management** (embedded skill tree +
  `<tool> skills install/upgrade/status/remove/preflight`, integrity marker, claude default +
  generic `.agents` surfaces, user + project scopes); (2) **MCP interface** (`<tool> mcp` exposes
  all CLI interactions as MCP tools + lazily-loaded skills-as-resources); (3) **`--json`
  agent-friendly output** (every command emits a documented machine-readable envelope +
  pipe-auto-enable). naba is the reference implementation. **Reconcile with
  `~/workspace/dixson3/yoshiko-flow`** (the `yf` kernel + `yf-*` skills, `yf-skill-authoring`
  conventions, skill surfaces/scopes/preflight, its `SPEC.md`). A portable skill/scaffolding
  template is a desirable stretch. **This is SPLIT into its own follow-on plan (plan-008)** — see the
  note after Epic 5; the description here seeds that plan.
- **Phasing:** one plan, ordered epics (config → provider/models+json → Bedrock → MCP → docs). The
  portable **agent-tools SPEC is a separate follow-on plan (plan-008)**, not an epic here.
- **No-secrets convention preserved:** api-keys via env/`api-key-envvar`, never committed;
  keep DRIFT-CHECK web/ edges in sync with the new capabilities.

## Investigation Findings
Full detail in `findings/exp-001-mcp-json.md`, `exp-002-config-provider.md`, `exp-003-bedrock.md`.

- **MCP (exp-001):** server uses **rmcp v2.2.0**, already advertises `tools` + `resources` (not
  `prompts`). Lazy-loading is **low surgery**: add a `resources/list` override + extend
  `resources/read` for a `skill://naba/<rel>` scheme, served from the existing `embed.rs`
  accessors (`skill_files`/`read_skill_file`). 8 tools today. **`--json` gaps:** only
  `config get/set`, all `skills` verbs, and `version` lack an envelope (everything else has one;
  global pipe-auto-enable via SPEC-GLOBAL-003 already exists). **No `provider`/`models`
  subcommands exist** — those are net-new (provider/model are global flags today).
- **Config/provider (exp-002):** config is a **flat all-`String`** struct (serde_norway YAML,
  `config.yaml`) with a hand-written `get`/`set`/`VALID_KEYS` triad. The **migration seam already
  exists** (`needs_structural_migration` hardcoded `false`; `migrate_file` writes `.bak`,
  idempotent) but flat→nested needs a **real transform**, not a re-serialize. The provider
  "registry" is **~9 hardcoded edit sites** (no abstraction) — this work should **introduce a
  registry abstraction**. `Provider` trait already has `list_models` → `naba models` has a
  foundation. SPEC.md §5/§6/§10 + `IG/configuration.md` govern and must be updated.
- **Bedrock (exp-003):** `InvokeModel` (sync, raw per-model JSON). Models: Nova Canvas, Titan v1/v2
  (shared Amazon schema), Stability Core/Ultra/SD3.5 (different schema); all return base64.
  **Design tension:** the full `aws-sdk` is ~70-110 crates AND the **api-key bearer path isn't
  cleanly supported by the Rust SDK** (needs an interceptor/hand-roll regardless). Since naba's
  providers are already thin `reqwest` clients, a **thin reqwest Bedrock client** (bearer for
  api-key, `aws-sigv4` only for profile) is recommended over the heavy SDK — **confirm at review.**
  Default region `us-east-1`.

## Approach

Land the work as **five ordered epics** (config → provider registry + commands + `--json` →
Bedrock → MCP → docs), each independently buildable and validated by `cargo fmt/clippy/test` +
the parity suite (and `web/ make validate` for the docs epic). Cross-cutting design choices:

- **Nested config, additive to the runtime, migrated on load.** A new nested `Config`
  (`default_provider` + a `providers` map of `{model, api-key?, api-key-envvar?}` + top-level
  image defaults) replaces the flat struct. The **existing migration seam** (exp-002) is
  extended: implement `needs_structural_migration` to detect the flat shape and give
  `migrate_file` a real flat→nested transform, writing `config.yaml.bak` first (idempotent).
  Old flat `naba config set model X` maps onto the current `default_provider`.
- **Uniform api-key resolution** (one function, all providers): inline `api-key` → custom
  `api-key-envvar` → the provider's conventional default env var. This **centralizes** the env
  names that are currently duplicated between `config.rs` and `select.rs`.
- **A real provider registry** replaces the ~9 hardcoded match sites (exp-002) so adding a
  provider is one registration, not a shotgun edit. `naba provider`/`naba models` read from it;
  Bedrock registers through it.
- **Bedrock via a thin `reqwest` client (recommended, exp-003)** — bearer header for the
  api-key path (no signing), `aws-sigv4` only for the profile path — matching naba's existing
  reqwest providers and avoiding the ~100-crate `aws-sdk`. The transport choice is a **decision
  gate** (below) because the operator's initial answer was `aws-sdk`; the finding argues for
  reqwest. Either way both auth modes (profile + api-key) are supported.
- **`--json` everywhere** by closing the three gaps (config get/set, skills verbs, version)
  with documented envelopes; keep the SPEC-GLOBAL-003 pipe-auto-enable.
- **MCP lazy-loading via resources** (low surgery, exp-001): a `resources/list` override + a
  `skill://naba/<rel>` `resources/read`, served from `embed.rs`, so a client lists skills
  cheaply and pulls detail on demand; tool schemas stay lean.
- **Docs + README + DRIFT-CHECK** last, so they document the *shipped* surface: full config
  schema, separate Skills + MCP pages, README install/setup synced to web, new DRIFT-CHECK edges.
- **SPEC.md + IG updates** ride with the epic that changes the behavior (config→§6/§10, provider→§5,
  mcp→MCP/IG) so the specs never lag the code (DRIFT-CHECK enforces it).

## Epics

### Epic 1: Nested per-provider config + migration + uniform api-key resolution
- Issue 1.1: Implement the nested `Config` schema — `default_provider`, a `providers` map keyed
  by provider name (`{model, api-key?, api-key-envvar?}`), top-level image defaults (aspect,
  resolution, quality, output_dir). serde_norway (de)serialize; **each provider designates its
  own default model**. **Built-in per-provider default model fallback** (red-team #7): when
  `providers.<name>.model` is absent, resolve to that provider's compiled-in default (e.g.
  `gemini::DEFAULT_MODEL`, `openrouter::DEFAULT_MODEL`, and the Bedrock default from Epic 3) — no
  provider is ever left model-less.
- Issue 1.2: Uniform api-key resolution — one resolver: inline `api-key` → `api-key-envvar` →
  provider's conventional default env var (e.g. `GEMINI_API_KEY`). Centralize env-var names
  (remove the `config.rs`/`select.rs` duplication).
  - depends-on: 1.1
- Issue 1.3: Auto-migration flat→nested — implement `needs_structural_migration` (detect flat
  shape) + a real transform in `migrate_file`; write `config.yaml.bak`; idempotent. **Per-key
  mapping (red-team #2), explicit:** `api_key` → `providers.gemini.api-key` (its historical
  Gemini-scoped meaning, regardless of `provider`); `model`/`quality` → the resolved
  `default_provider`'s entry; `provider` → `default_provider` (with an explicit fallback default
  when the old `provider` field is absent). Unit tests: backup written, idempotent re-run,
  comment-loss accepted, malformed/empty no-op, **and the "openrouter default + stray gemini
  api_key" case** (the stray key lands under `gemini`, not `openrouter`).
  - depends-on: 1.1
- Issue 1.4: `naba config` command surface for nested keys — dotted addressing
  (`default-provider`, `<provider>.model`, `<provider>.api-key`, `<provider>.api-key-envvar`, plus
  top-level image defaults); validation + error messages; **add `--json` to `config get`/`config
  set`** (closes a `--json` gap). Tests. *(Envelope shape is finalized by Issue 2.4's universal
  contract — treat 1.4's config envelope as provisional, not frozen, so 2.4 can normalize it
  without rework; red-team pass-2, low.)*
  - depends-on: 1.1, 1.2
- Issue 1.5: Update `SPEC.md` §6 (SPEC-CFGSCHEMA rewrite: nested schema, drop "no openrouter key"
  special-case, **per-provider default-model fallback**) and §10 (SPEC-MIGRATE: reclassify
  flat→nested as **structural**, record the per-key mapping), plus
  `docs/specifications/IG/configuration.md` (format example + valid keys + resolution order).
  - depends-on: 1.1, 1.2, 1.3, 1.4
- Issue 1.6: **Adapt the existing `select.rs`/`resolve_*`/`to_config_defaults` call sites to the
  nested `Config` (pre-registry shim)** so Epic 1 compiles and `cargo test` + the parity suite stay
  green *on its own*, before the Epic-2 registry lands (red-team #3). A minimal adapter that feeds
  the nested schema into today's selection logic; the registry (2.1) later supersedes it.
  **Executor notes (red-team pass-2, low):** keep 1.6 the thinnest adapter that compiles — no new
  abstraction 2.1 would just delete; and because 1.1 breaks the ~9 call sites until 1.6 rewires
  them, **land 1.1→1.6 as a single green boundary** (one commit/unit) so the change-validation FAST
  tier isn't spuriously red mid-epic.
  - depends-on: 1.1, 1.2

### Epic 2: Provider registry + `naba provider`/`naba models` + `--json` completion
- Issue 2.1: Introduce a provider **registry abstraction** (name → {default env var, builder from
  resolved creds, list_models}) replacing the ~9 hardcoded sites in `select.rs`/`mod.rs`. Preserve
  the SPEC-PROVIDER precedence + autodetect behavior for the existing two providers (parity stays
  green). **Define an explicit N-provider autodetect precedence order** (red-team #6): the
  registry's declared ordering is the tie-break when multiple providers have resolvable creds
  (the 2-bool autodetect becomes an ordered scan). Add autodetect unit tests for the new
  multi-cred combinations (incl. Bedrock-present alongside gemini/openrouter), not just the legacy
  two-provider cases.
  - depends-on: 1.1, 1.2
- Issue 2.2: `naba provider` command — list all registered providers and which have valid/resolvable
  credentials (human + `--json` envelope).
  - depends-on: 2.1
- Issue 2.3: `naba models [--provider <name>]` — list a provider's models via `Provider::list_models`
  (default provider if unspecified; human + `--json`).
  - depends-on: 2.1
- Issue 2.4: **Define a universal `--json` envelope contract** (red-team #5) — a SPEC clause
  fixing the common shape (e.g. `status`/`data`/`error`) that every subcommand's envelope conforms
  to — then close the remaining gaps: `config get/set` (1.4), `skills install/upgrade/remove/status`,
  and `version` emit that envelope (respect `globals.json` + SPEC-GLOBAL-003). Add a
  **traceability/parity test that enumerates every subcommand and asserts envelope presence +
  schema conformance**, so "universal" is enforced, not aspirational.
  - depends-on: 2.1
- Issue 2.5: Update `SPEC.md` §5 (SPEC-PROVIDER: registry, provider count no longer fixed at two,
  the **explicit N-provider precedence order**, `provider`/`models` commands) + the §Global
  `--json` envelope contract clause (2.4) to match.
  - depends-on: 2.1, 2.2, 2.3, 2.4

### Epic 3: AWS Bedrock image provider (+ unit tests)
- Issue 3.1: **[Capability gate: bedrock-transport]** Operator confirms the Bedrock transport —
  thin `reqwest` (recommended, exp-003) vs full `aws-sdk`. Blocks 3.2.
- Issue 3.2: Implement `BedrockProvider` (`Provider` trait) — `InvokeModel` over the chosen
  transport; Amazon (Nova Canvas, Titan v1/v2 — shared schema) + Stability (Core/Ultra/SD3.5)
  request/response families; base64 decode; region default `us-east-1` (configurable); register via
  the Epic-2 registry; `list_models` returns the curated set.
  - depends-on: 3.1, 2.1
- Issue 3.3: Bedrock auth — both modes: AWS **profile/SigV4** (`aws-sigv4` or SDK) and **api-key
  bearer** (`AWS_BEARER_TOKEN_BEDROCK` / config `api-key`), wired through Epic-1's uniform
  resolution plus a bedrock-specific profile/region config.
  - depends-on: 3.2
- Issue 3.4: **Unit tests** — per-family request-body serialization, response/base64 parsing,
  auth-mode selection (profile vs api-key), `list_models`, HTTP mocked with `wiremock` (naba's test
  idiom). Update SPEC/IG for the new provider.
  - depends-on: 3.2, 3.3

### Epic 4: MCP lazy-loading (skills as resources)
- Issue 4.1: Implement `resources/list` + extend `resources/read` for a `skill://naba/<rel>` scheme,
  serving the embedded skill tree (`SKILL.md`, `commands/*.md`, `README.md`) via `embed.rs`
  accessors as text/markdown — an mcp-friendly, lazily-fetched skills surface.
  - depends-on: (none intra-plan; MCP is independent)
- Issue 4.2: Keep tool schemas lean; ensure listing is cheap (paths) and detail is on-demand;
  optional compact index resource. Update `SPEC.md` MCP section + `docs/specifications/IG/mcp-server.md`.
  - depends-on: 4.1
- Issue 4.3: Tests — `resources/list` enumerates the skill files, `resources/read` returns content,
  handshake advertises resources; no regression to the 8 tools.
  - depends-on: 4.1

### Epic 5: Documentation + README sync + DRIFT-CHECK
- Issue 5.1: Config page (`web/`) — the **complete config-file schema** (nested), every key, the
  api-key resolution precedence, per-provider default model, and the provider list. **The Bedrock
  rows (Bedrock auth: profile + api-key, regions) are a separable sub-item** gated on 3.3
  (red-team #8): if Bedrock defers, its doc rows defer with it and the rest of the Config page
  still lands. The non-Bedrock config schema depends only on 1.4 + 2.2. *(Land-time, red-team
  pass-2 low: if 5.1 closes before Bedrock ships, file the Bedrock doc rows as a `discovered-from`
  follow-on off 3.3 so they are not left unowned.)*
  - depends-on: 1.4, 2.2 (Bedrock rows sub-item additionally: 3.3)
- Issue 5.2: New **Skills page** (`web/`) — enumerate skill subcommands + triggers, emphasizing
  **implicit triggering**; claude-code default surface vs generic `.agents/skills`; user- vs
  project-scope; full lifecycle (install/upgrade/status/remove/preflight).
  - depends-on: 2.4
- Issue 5.3: New **MCP page** (`web/`) — enumerate all MCP tools + params, the **lazy-loading
  resources** model, Claude Desktop config, `NABA_OUTPUT_DIR`.
  - depends-on: 4.2
- Issue 5.4: Wire the new pages into nav + home cards/cross-links; add `naba provider`/`naba models`
  usage; ensure the usage page covers the new commands.
  - depends-on: 5.1, 5.2, 5.3
- Issue 5.5: **Sync the project `README` install/setup** with the web install/config docs — new
  config surface (nested, per-provider), providers incl. Bedrock, skills lifecycle, MCP pointer.
  - depends-on: 5.1, 5.2, 5.3
- Issue 5.6: `DRIFT-CHECK.md` — add `web-skills` + `web-mcp` derived nodes and edges (from
  `cli-source`/`skill-md`/mcp source), a README-install ↔ web-install/config edge, and the new
  provider/config/mcp trigger-scope globs; re-approve §0. Keep the existing web edges consistent.
  - depends-on: 5.1, 5.2, 5.3, 5.5

> **Epic 6 (portable "agent-tools" SPEC) was SPLIT into its own plan (plan-008)** per operator
> decision (red-team pass-1 #4): the tool-agnostic SPEC — skills self-management + MCP-over-CLI +
> `--json` agent output, reconciled with `~/workspace/dixson3/yoshiko-flow`, naba as reference
> implementation — is a research-and-authoring effort of different character, and its yoshiko-flow
> inputs were not de-risked in this plan's investigation. It will be a **new plan seeded by a
> yoshiko-flow reconnaissance finding**, filed as a follow-on at this plan's intake. plan-007 keeps
> the shipping-naba work (epics 1–5); the universal `--json` (2.4), the skills lifecycle, and the
> MCP lazy-loading (Epic 4) it authors become the reference material plan-008 draws on.
> **Land-time (red-team pass-2):** the plan-008 seed must be filed as an actual `bd` follow-on bead
> at intake (per the portability rule), not left only in this prose.

## Gates

### Start Gate (mandatory)
- Type: human
- Approvers: operator
- Condition: operator confirms the **config-migration strategy** (auto-migrate flat→nested on load
  with a `.bak` backup, per the SELECTED decision in §Scoping) before Epic 1 execution begins. This
  folds the migration decision into the Start Gate rather than a separate gate (red-team pass-1 #1).

### Capability Gate: bedrock-transport (design decision)
- Type: human
- Approvers: operator
- Condition: operator has chosen the Bedrock transport — thin `reqwest` (recommended per
  `findings/exp-003-bedrock.md`: ~100 fewer crates, matches naba's reqwest providers, and the
  api-key bearer path needs hand-rolling under the SDK anyway) **vs** the full `aws-sdk`
  (first-class SigV4/SSO/role support, heavier).
- Test: operator decision recorded in the plan phase log / Issue 3.1.
- Blocks: Issue 3.2 (Bedrock implementation).
- Instructions: confirm the transport before Bedrock is implemented; the choice sets the crate
  dependencies and the auth wiring.

## Risks & Mitigations
- **Config migration data loss / comment loss.** *Mitigation:* extend the tested migration seam —
  `.bak` written with original bytes before any rewrite, idempotent re-run, malformed/empty no-op;
  comment loss on structural rewrite is accepted + backed up (SPEC-MIGRATE-003). Unit tests cover
  each path.
- **Breaking the config schema for existing users.** *Mitigation:* auto-migration on load; naba is
  pre-1.0 with no cargo-dist release (#7), so a structural change is acceptable; old flat `config
  set` keys keep working by mapping onto `default_provider`.
- **Bedrock dependency weight / api-key bearer gap.** *Mitigation:* the bedrock-transport decision
  gate; thin-reqwest recommendation keeps the footprint small and makes the api-key path first-class
  (bearer header) rather than fighting the SDK.
- **Provider-registry refactor changing precedence/autodetect.** *Mitigation:* preserve SPEC-PROVIDER
  precedence semantics; the parity suite + unit tests gate the refactor; SPEC §5 updated in lockstep.
- **`--json` envelope churn.** *Mitigation:* document each new envelope; parity/traceability checks;
  keep existing envelopes stable.
- **MCP client compatibility.** *Mitigation:* the `resources` capability is already advertised
  (exp-001), so lazy-loading adds handlers without a handshake break; tests assert list/read + the
  unchanged 8 tools.
- **Scope size.** *Mitigation:* five independently-landable epics with intra-plan dependencies; each
  is validated on its own; Bedrock can even be deferred if the gate stalls (docs/config still land).
- **Docs drifting from the new surface.** *Mitigation:* the docs epic runs last (documents shipped
  behavior) + new DRIFT-CHECK edges make future drift a flagged failure.
- **Rollback.** *Mitigation:* the only irreversible on-disk change is the config rewrite, guarded by
  `config.yaml.bak` (restore = copy `.bak` back). Everything else is code landed via the normal
  git/parity flow and revertable by commit; naba is pre-1.0 with no released binary depending on the
  old schema.

## Success Criteria
1. Config is nested per-provider: `default_provider` + per-provider default `model` + api-key
   resolved (inline → `api-key-envvar` → default env var); flat→nested auto-migration writes a
   `.bak`, is idempotent, and is unit-tested.
2. A provider **registry** replaces the hardcoded match sites; adding a provider is a single
   registration; SPEC-PROVIDER precedence/autodetect behavior is preserved (parity green).
3. **AWS Bedrock** provider works over the chosen transport with **both** auth modes (profile +
   api-key), a curated model set enumerable via `naba models`, and passing unit tests.
4. `naba provider` (providers + credential validity) and `naba models [--provider <name>]` exist,
   with human + `--json` output.
5. **Every** subcommand supports `--json` with a documented envelope — the config/skills/version
   gaps are closed; SPEC-GLOBAL-003 pipe-auto-enable retained.
6. MCP surfaces the skills as **lazily-loaded resources** (`resources/list` + `skill://` read); tool
   schemas stay lean; the 8 tools are unchanged; tests pass.
7. Docs cover the full surface + lifecycle: Config page shows the **complete config schema**;
   separate **Skills** and **MCP** pages exist; the project **README install/setup is in sync** with
   the web docs; new **DRIFT-CHECK edges** keep them in sync; `SPEC.md`/IG are updated; and `cargo
   fmt/clippy/test`, the parity suite, and `web/ make validate` all pass.
8. _(Moved to plan-008.)_ The tool-agnostic **agent-tools SPEC** — skills self-management + MCP
   interface + `--json` agent output, reconciled with yoshiko-flow, naba as reference implementation
   — is filed as a follow-on plan at this plan's intake, seeded by a yoshiko-flow reconnaissance
   finding. Not a plan-007 completion criterion.
