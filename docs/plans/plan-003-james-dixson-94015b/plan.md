# Plan: Modernize naba Gemini model usage: replace dead default gemini-2.0-flash-exp-image-generation with GA gemini-3.1-flash-image, wire imageConfig (aspectRatio + imageSize), expose gemini-3-pro-image quality tier, reconcile docs/spec

**ID:** plan-003-james-dixson-94015b
**Author:** james-dixson
**Created:** 2026-06-14
**Status:** complete
**Epic:** naba-mol-mdw
**Phase log:**
- 2026-06-14 scoping: initial scope captured
- 2026-06-14 investigating: scope captured; 2 experiments (E1 imageConfig schema, E2 pro parity)
- 2026-06-14 drafting: E1/E2 done; synthesizing plan
- 2026-06-14 drafting: plan v1 synthesized
- 2026-06-14 review: red-team pass-1 REVISE presented
- 2026-06-14 review: red-team pass-2 REVISE presented (C1-C7 verified resolved; N1-N4 new)
- 2026-06-14 review: red-team pass-3 REVISE presented (Epics 6/7 sound; C8-C11 + missing)
- 2026-06-14 review: red-team pass-4 APPROVE (marker sound; 2 low notes folded)
- 2026-06-14 approved: operator approved (4 red-team passes; pass-4 APPROVE)
- 2026-06-14 intake: epic naba-mol-mdw poured
- 2026-06-14 executing: start gate resolved
- 2026-06-14 reconciling: all execution beads closed
- 2026-06-14 complete: plan complete

## Objective
Modernize naba on two axes: (1) **Gemini model/API** — replace the dead default
gemini-2.0-flash-exp-image-generation with GA gemini-3.1-flash-image, wire imageConfig
(aspectRatio + imageSize), expose gemini-3-pro-image, reconcile docs/spec; and (2) **CLI
distribution** — add a `naba skills` lifecycle command (binary-embedded skills installed to
user scope) that supersedes `install.{sh,py}`, plus a `naba doctor` health command. (The
Homebrew-tap consolidation is deferred to upstream issue #3.)

## Motivation

naba's image generation client (`internal/gemini/client.go:19`) hardcodes
`defaultModel = "gemini-2.0-flash-exp-image-generation"`, a model Google **shut down
November 14, 2025**. Any fresh user who runs naba without a `model` config override calls a
retired model and the request fails outright. Existing installs only work if they set
`model` in config (as the maintainer's does, to `gemini-2.5-flash-image`).

Meanwhile Google released GA image models on May 28, 2026: `gemini-3.1-flash-image`
(Nano Banana 2) and `gemini-3-pro-image` (Nano Banana Pro), which also expose an
`imageConfig` block (aspect ratio + resolution) that naba does not use — its `--size`/
`--tile-size` flags are prompt-text only. Documentation (`README.md:99`,
`docs/specifications/IG/configuration.md`, `docs/specifications/EDD/CORE.md:161`) still
references the dead/old models.

Affected: every fresh naba install (broken default) and all users (missing current models
and resolution/aspect control). Triggered by the maintainer's request to audit the
API/model calls for newer/more capable Google image models.

## Upstream Issues

None incorporated. The open-issue search returned 0 at scoping time; this plan later *filed*
GitHub issue **#3** (homebrew-tap consolidation) as **deferred follow-up work** — it is
explicitly out of scope here (see Scope Decisions), not an issue this plan resolves.

## Scope Decisions (operator-confirmed 2026-06-14)

| # | Decision | Choice |
|:-:|:---------|:-------|
| 1 | Default model | Replace dead `gemini-2.0-flash-exp-image-generation` → **`gemini-3.1-flash-image`** (current GA, cost/latency-optimal default). |
| 2 | imageConfig UX | Add general **`--aspect`** (1:1, 16:9, 9:16, 21:9, …) and **`--resolution`** (512, 1K, 2K, 4K) flags → `generationConfig.imageConfig{aspectRatio, imageSize}` on **all generative commands** (generate, edit, restore, pattern, diagram, story). `icon --size` (px) stays a separate concept (canvas pixels, not imageConfig `imageSize`). |
| 6 | Output extension | API returns **JPEG**. When `-o <path>` extension disagrees with the response mimeType, **correct the extension** on disk (e.g. `.png`→`.jpg`), **warn** that it changed from the directive, and **surface requested-vs-actual format in JSON output** so the operator/LLM can decide if post-generation conversion is needed. |
| 3 | Pro tier | Selectable via raw **`--model gemini-3-pro-image`** (documented) **and** via a **`--quality {fast,high}`** alias (fast→flash, high→pro) for parity with the config `quality` key (#5). `--model` is the highest-precedence raw override. |
| 4 | MCP parity | Full — add aspectRatio / imageSize / quality(model) params to the MCP image tool definitions (`internal/mcp`), not just inherit the default-model fix. |
| 5 | Config defaults | Extend `~/.config/naba/config.yaml` with default **`model`** (exists), **`aspect`**, **`resolution`**, and **`quality`** (alias over model). Per-call flags override config; precedence: flag > config > built-in default. |
| 7 | `naba skills` command | New cobra command group with lifecycle verbs: **`install`**, **`upgrade`**, **`remove`**, **`status`** (operator phrased as `--install`/`--upgrade`/`--remove`; implemented as verbs). Installs to **user scope** by default with `--scope`/`--surface`/`--target` mirroring `install.py`. Skill files are **embedded in the binary via `go:embed`** (offline, version-matched). |
| 8 | `naba doctor` | New command validating environment health: skills installed (per #7), `GEMINI_API_KEY`/config present, **and a live key check** (a cheap `models.list`/`countTokens` call confirming the key works and the configured model is reachable — no image cost), config parseable, binary version. |
| 9 | Installer supersession | `naba skills` becomes the canonical installer. `install.{sh,py}` is **removed (or reduced to a thin shim that calls `naba skills install`)**. This touches the DRIFT-CHECK `installer` node + `e-installer-frontmatter` edge (manifest update, on top of #C7's model-id edge). |
| 10 | Skill integrity marker | On `install`/`upgrade`, `naba skills` injects a single **hidden HTML-comment marker** into the deployed `SKILL.md`: `<!-- naba-skills: v=<naba-version> tree=<sha256> -->`. `<sha256>` digests the **canonical embedded tree** (sorted relative paths + bytes), computed **excluding** the marker line. The binary hashes its own `embed.FS` at runtime (deterministic, no build step). The **repo source `skills/naba/SKILL.md` stays marker-free** (injected only on deploy). `skills status` and `doctor` read the marker, recompute the install's tree hash (marker stripped) + confirm every embedded file is present → **up-to-date / complete / unmodified**. |

**Cost/access context (informs #1, #3):** all three image models require a **paid (billing-enabled) tier** — none are free-tier. Pro costs ~2–3.5× flash per image
(flash ~$0.067/1K, pro ~$0.134/1K). Flash is the default for cost/latency; Pro is opt-in
for final/hero assets. This is a cost trade-off, not an availability gate.

**`go:embed` constraint (informs #7):** `//go:embed` cannot reference parent directories, so
the embed directive must live in a **repo-root package** embedding `skills/` (e.g. a root
`naba.go`), imported by `cmd/naba` — not inside `cmd/naba/`. Confirm at implementation.

Out of scope: no changes to the post-generation pipeline (resize/preview/output writing);
no new image *operations* (still generate/edit/restore/icon/pattern/diagram/story); no
provider beyond Google Gemini. **Homebrew-tap consolidation is deferred to upstream issue #3**
(it builds on `naba skills` from this plan).

## Investigation Findings

Full detail in `findings/exp-001-model-schema.md`. Live verification (2026-06-14, real key):

- **E1 ✓** `gemini-3.1-flash-image` returns HTTP 200 through naba's existing request shape,
  and with `imageConfig`. `models.list` confirms it (and `gemini-3-pro-image`,
  `gemini-2.5-flash-image`) are available; the repo default
  `gemini-2.0-flash-exp-image-generation` is **absent** (retired).
- **E2 ✓** `gemini-3-pro-image` works with the identical shape + `imageConfig` → Pro via raw
  `--model` needs no client change.
- **Schema (load-bearing):** `generationConfig.imageConfig{aspectRatio, imageSize}`
  (Schema A). The docs' `responseFormat.image` paraphrase is **wrong** (live-disproven).
  `aspectRatio` ∈ {1:1,…,16:9,21:9}; `imageSize` ∈ {512,1K,2K,4K} (uppercase K).
- **NEW — API is permissive:** an invalid `imageSize:"1k"` returned 200 (silently ignored),
  so **client-side enum validation is required**.
- **NEW — JPEG:** responses are `image/jpeg`; output/extension handling must be
  mimeType-driven, not PNG-hardcoded.

## Approach

A focused Go change across the gemini client, CLI, MCP, plus docs/spec/tests. The request
schema and model ids are now verified, so there are no remaining unknowns.

1. **Client (`internal/gemini`).** Bump `defaultModel` → `gemini-3.1-flash-image`. Add an
   `ImageConfig` struct and an `*ImageConfig` field (`omitempty`) on `GenerationConfig` so
   bare requests stay byte-identical. Add `Generate`/`GenerateWithImage` variants (or an
   options arg) that accept aspect/resolution. Validate enums client-side (API won't).
2. **CLI (`internal/cli`).** Add `--aspect` + `--resolution` to all generative commands
   (generate, edit, restore, pattern, diagram, story; icon px-only) and `--quality
   {fast,high}` (alias: fast→flash, high→pro). Model precedence `--model` > `--quality` >
   config > default, using cobra `Changed()` (not empty-string sentinels). Reconcile the
   output extension to the response mimeType with a warning + JSON requested-vs-actual.
3. **Config (`internal/config`).** Add `aspect`, `resolution`, `quality` keys (Get/Set/
   ValidKeys); per-call flags override config; config overrides built-in default; intra-config
   tiebreak config `model` > config `quality`.
4. **MCP (`internal/mcp`).** Signature refactor: thread model + imageConfig through
   `resolveClient`/`generateAndReturn`/`generateWithImageAndReturn`, add the params to the
   image tool definitions, and fix the three pre-built `image/png` output paths
   (`server.go:187,259,325`) to defer the extension to the actual mimeType.
5. **Docs/spec.** Reconcile model ids and the new behavior across README + the five touched
   specs (configuration, EDD/CORE, image-generation, **output-handling**, **mcp-server**);
   document the new flags, the `gemini-3-pro-image` option, the JPEG output/extension
   behavior, **and add a pricing / not-free + model↔quality callout to the README**
   (operator request 2026-06-14).
6. **`naba skills` command (`internal/cli` + embed).** New cobra group with `install`/
   `upgrade`/`remove`/`status` verbs. Embed `skills/` in the binary via a repo-root
   `go:embed` package + a canonical tree-hash. `install`/`upgrade` write the embedded tree to
   the resolved scope (`--scope`/`--surface`/`--target`, default user/claude) and **inject the
   integrity marker** (scope #10) into the deployed `SKILL.md`; `remove` deletes it; `status`
   uses the marker to report up-to-date / complete / unmodified. Port the install/uninstall
   logic from `install.py` (now single-skill) into Go, then **remove `install.{sh,py}`** (or
   leave a thin shim).
7. **`naba doctor` command.** Validate: skills installed **and matching the embedded binary**
   (marker hash == embedded tree hash + completeness), key present (`GEMINI_API_KEY`/config),
   **live key check** (cheap `models.list`), configured model reachable (intersection),
   config parseable, version. Structured pass/warn/fail output + non-zero exit on fail.
8. **Tests/validate.** Constant-assertion test for `defaultModel` (the bug class that
   slipped before — see `docs/plans/plan-02.md`), imageConfig marshaling, enum validation,
   model-precedence resolution, embedded-skills integrity + `skills install/remove` to a temp
   `--target`, `doctor` check logic; live end-to-end smoke (flash + pro, jpeg output, bad-enum
   rejection); drift-check + markdown-lint.

## Epics

### Epic 1: Gemini client — model + imageConfig
- Issue 1.1: Replace dead `defaultModel` → `gemini-3.1-flash-image` in
  `internal/gemini/client.go`. (Critical: unbreaks fresh installs.)
- Issue 1.2: Add `ImageConfig` struct + `*ImageConfig`(`omitempty`) field on
  `GenerationConfig` (`types.go`); thread aspect/resolution into `Generate`/
  `GenerateWithImage`. Bare-call request stays byte-identical.
  - depends-on: 1.1
- Issue 1.3: Client-side enum validation for `aspectRatio`/`imageSize` (reject invalid →
  `ExitUsage`), since the API silently ignores bad values (E1 finding).
  - depends-on: 1.2
- Issue 1.4: **Output extension reconciliation (resolves C2/C3; N2/N4).** Responses are
  JPEG. Make the on-disk extension match the response mimeType: when a user `-o <path>`
  extension disagrees with the returned mimeType, **write the corrected extension**
  (`hero.png` → `hero.jpg`) and **warn on stderr** stating requested-vs-actual format.
  Surface scope (N2): the change spans
  (a) `internal/output/writer.go` (`OutputPath`/`WriteImage` — extension currently honored
  literally only when `outputPath==""`);
  (b) the **CLI JSON carrier** `internal/output/Result`/`json.go` — add `requested_format` /
  `actual_format` (or `Params`) fields, populated in the per-command handlers so the JSON
  reports the mismatch;
  (c) the **icon CLI path** `internal/cli/icon.go` + `output.ExtForFormat` (icon builds its
  path outside `WriteImage`, so an icon JPEG also mis-lands `.png` — same correction+warning;
  icon's imageConfig stays out of scope, this is extension-only);
  (d) the **three MCP call sites** `server.go:187,259,325` (pre-built `image/png` path) →
  defer extension to actual mimeType. The MCP carrier for format (N2): the MCP
  `imageResult()` currently returns only path + `ResourceLink`; add the corrected path + a
  format note to the MCP tool result text (MCP does not emit the CLI `Result` JSON).
  depends-on: 1.2

### Epic 2: CLI flags + config defaults
- Issue 2.1: Add `--aspect` and `--resolution` flags to **all generative commands**
  (`generate`, `edit`, `restore`, `pattern`, `diagram`, `story`) mapping to `imageConfig`;
  `icon` keeps its px `--size` (canvas pixels — a separate concept, **not** imageConfig
  `imageSize`; docs must call this out). depends-on: Epic 1
- Issue 2.2: Add `--quality {fast,high}` alias (fast→`gemini-3.1-flash-image`,
  high→`gemini-3-pro-image`) + model-resolution precedence `--model` > `--quality` >
  config > default. **Use cobra `cmd.Flags().Changed("model")/Changed("quality")`** to
  detect explicit flags rather than empty-string sentinels (resolves C6). depends-on: Epic 1
- Issue 2.3: Extend config (`internal/config`) with `aspect`/`resolution`/`quality` keys
  (Get/Set/ValidKeys). Precedence: flag > config > built-in default; **intra-config
  tiebreak: config `model` beats config `quality`** (mirrors flag precedence, resolves C4).
  depends-on: 2.1, 2.2

### Epic 3: MCP parity
- Issue 3.1: **Full MCP parity — signature refactor (resolves C1).** `resolveClient()`
  (`server.go:59`) takes no model override and the `generateAndReturn`/
  `generateWithImageAndReturn` helpers take no imageConfig — so this is a signature change,
  not param-passing: thread model + imageConfig (aspect/resolution/quality) through
  `resolveClient` and both helpers, then add aspectRatio/imageSize/quality params to the
  image tool definitions in `internal/mcp/tools.go`. State that with `count>1` all N calls
  reuse the same imageConfig. (The MCP `image/png` hardcode fix lives in Issue 1.4.)
  depends-on: Epic 1

### Epic 4: Docs + spec reconcile
- Issue 4.1: README — update model refs (default + examples), document `--aspect`/
  `--resolution`/`--quality` and `gemini-3-pro-image`, the JPEG output/extension behavior
  (1.4), that `icon --size` is canvas px (not `imageSize`), **and add a pricing / not-free +
  model↔quality dynamic callout** (all image models require a paid tier; flash vs pro cost
  trade-off). Note existing configs (`model: gemini-2.5-flash-image`) keep working via
  precedence. depends-on: Epic 2
- Issue 4.2: Reconcile the specs the change touches (N1):
  `docs/specifications/IG/configuration.md` (default model + new config keys + tiebreak),
  `docs/specifications/EDD/CORE.md:161` (default model),
  `docs/specifications/IG/image-generation.md` (imageConfig + model lineup),
  `docs/specifications/IG/output-handling.md` (mimeType/extension reconciliation + the
  `Result` JSON format fields from 1.4), `docs/specifications/IG/mcp-server.md`
  (MCP write flow + format note), and `docs/specifications/IG/skills.md` (the `naba skills`
  lifecycle, the integrity-marker format + hashing, `naba doctor`'s skill-match check —
  scope #7/#8/#10). depends-on: Epic 2, Epic 3, Epic 6, Epic 7
- Issue 4.3: **DRIFT-CHECK manifest update (resolves C7; N3 scoped; + installer
  supersession).** (a) Model-id edge: the current manifest has only `README.md` as a node
  (`project-readme`); `IG/configuration.md` and `EDD/CORE.md` are not nodes. Add **three
  nodes** — `gemini-source` (`internal/gemini/client.go`, `fixed`), `ig-configuration`,
  `edd-core` — `value-equal` edges from the `defaultModel` constant to the model ids in
  those docs, §6 globs, and a §7 clause naming `gemini-source` a second `fixed` authority.
  (b) **Installer supersession (scope #9; C8).** When `install.{sh,py}` is removed/shimmed,
  retarget every manifest clause that hardcodes it — not just the node + edge:
  the `installer` **node** (`install.py`) and `e-installer-frontmatter` **edge**; the §4
  **referencer** for `installer`; the two §5 **required-section** rows
  (`Install → install.sh/install.py`, `Skill install instructions → install.sh flags`) →
  point them at `naba skills install`; and the **referencer prose** in
  `skills/naba/README.md:41-43` and `AGENTS.md:59` ("deployed via `./install.sh`"). If
  `naba skills` install owns discovery, point the edge at the embedded-skills source. Drop the
  manifest to `approved: no` and obtain operator re-approval. Sequence before Epic 4 closes.
  depends-on: Epic 1, 6.3

### Epic 6: `naba skills` lifecycle command
- Issue 6.1: Add a repo-root `go:embed` package embedding `skills/` (per the embed
  constraint), exposing the embedded skill tree to the binary, plus a **canonical tree-hash**
  helper used by install/status/doctor (C12): digest = sha256 over, for each file sorted by
  relative path, the relative-path bytes then the file bytes; **raw bytes, no line-ending or
  trailing-newline normalization** (files end in a single `\n` today). To hash a *deployed*
  tree, first remove the single anchored `^<!-- naba-skills: .* -->$` line **and its
  terminator** from the deployed `SKILL.md` only (first match), restoring the embedded original
  byte-for-byte. The marker is emitted as exactly one line (no embedded newlines).
  depends-on: (none)
- Issue 6.2: Add the `naba skills` cobra group with `install`/`upgrade`/`remove`/`status`
  verbs and `--scope`/`--surface`/`--target`/`--dry-run` flags (default user/claude), porting
  the install/uninstall + `resolve_dests` logic from `install.py` into Go over the embedded
  tree. **Intentionally dropped** (single skill, no deps, no rules):
  `--group`/`--list-groups`/`--strict`/dependency-closure/companion-rules. On `install`/
  `upgrade`, **inject the integrity marker** (scope #10) into the deployed `SKILL.md` (after
  the YAML frontmatter, so it does not break the frontmatter parse). `upgrade` **rewrites each
  dest file from the (marker-free) embed before injecting** a fresh marker — injection is
  idempotent (strip any existing marker, then inject) so it never double-marks (C13) — and
  **prunes dest files absent from the embed** (`rsync --delete` parity) so stale files don't
  persist.
  `status` reads the installed marker and reports **up-to-date** (marker `tree` == embedded
  hash), **complete** (all embedded files present), and **unmodified** (recomputed install
  hash, marker stripped, == embedded hash) — or which of those failed. depends-on: 6.1
- Issue 6.3: Supersede `install.{sh,py}` (scope #9) — remove them (or reduce `install.sh` to
  a thin shim calling `naba skills install`); update **all** references incl.
  `skills/naba/README.md` and `AGENTS.md` prose. **Rewrite the plan-002 breaking-change
  migration note** (`README.md:152-176`): `./install.sh --uninstall` no longer exists, and
  `naba skills` cannot remove the legacy `/naba-*` skills it never embedded — provide an
  explicit **manual removal command** for pre-plan-002 installs instead of a dangling
  instruction (C9). depends-on: 6.2

### Epic 7: `naba doctor` command
- Issue 7.1: Add `naba doctor` — checks: **skills installed AND match the embedded binary**
  (reuse `skills status` against the **default user/claude** dest unless `--scope`/`--surface`
  given — C11; `fail` if the marker is missing, the `tree` hash ≠ the binary's embedded hash
  (outdated), or the install is incomplete/modified — scope #10),
  `GEMINI_API_KEY`/config key present, **live key validation** (cheap `models.list`, net-new —
  no call exists today), and **configured model reachability as a `models.list` intersection**
  (the resolved model must appear in the returned list, with E1's name normalization;
  absent-but-key-valid = **`fail`**, the dead-default bug class — C10). Also: config parseable,
  binary version. Structured pass/warn/fail; non-zero exit on any fail. depends-on: Epic 1, 6.2

### Epic 5: Validate
- Issue 5.1: Go stdlib tests — `defaultModel` constant assertion (regression guard for the
  dead-default class), `imageConfig` JSON marshaling (incl. `omitempty` bare-call
  invariance), enum validation, model-precedence resolution (flag/config tiebreaks),
  output-extension reconciliation logic (1.4), **embedded-skills integrity + `skills
  install`/`remove` to a temp `--target`**, the **integrity-marker round-trip** (install →
  strip marker → re-hash == embedded hash) **and a tamper case** (modify an installed file →
  `status`/`doctor` report modified/outdated — scope #10), and **`doctor` check logic**
  (mock/seam the live call). depends-on: Epic 1, Epic 2, Epic 6, Epic 7
- Issue 5.2: Live end-to-end smoke — build naba; `generate --aspect 16:9 --resolution 1K` on
  flash and `--model gemini-3-pro-image`; assert (a) the **CLI** path writes a corrected
  `.jpg` (+ warning) when `-o *.png` is given, (b) an MCP image-tool call also writes a
  correctly-extensioned file (the `server.go` png fix), (c) an invalid `--resolution`
  is rejected client-side with `ExitUsage`, **(d) `naba skills install --target <tmp>` lands
  the skill tree and `naba doctor` reports green against the live key**. depends-on: Epic 1, Epic 2, Epic 3, Epic 6, Epic 7
- Issue 5.3: `markdown-lint` changed docs + `drift-check` against the **re-approved**
  manifest (incl. the new `gemini-source` edge and the installer-node change from 4.3).
  depends-on: Epic 4

## Gates

### Start Gate (mandatory)
- Type: human
- Approvers: operator

### Capability Gate: live Gemini API access
- Type: human
- Approvers: operator
- Condition: a paid-tier `GEMINI_API_KEY` can call `gemini-3.1-flash-image` and
  `gemini-3-pro-image` (both are paid-only; E1/E2 already confirmed this key works).
- Test (generation path): `curl -sS -o /dev/null -w '%{http_code}' -X POST "https://generativelanguage.googleapis.com/v1beta/models/gemini-3.1-flash-image:generateContent" -H "x-goog-api-key: $GEMINI_API_KEY" -H 'Content-Type: application/json' -d '{"contents":[{"parts":[{"text":"x"}]}],"generationConfig":{"responseModalities":["TEXT","IMAGE"],"imageConfig":{"aspectRatio":"16:9","imageSize":"512"}}}'` → expect `200`.
- Test (doctor path): `curl -sS -o /dev/null -w '%{http_code}' "https://generativelanguage.googleapis.com/v1beta/models?key=$GEMINI_API_KEY"` → expect `200` (proves the `models.list` path `naba doctor` depends on; no image cost).
- Blocks: Issue 5.2 (live smoke incl. `naba doctor` green).
- Instructions: ensure billing-enabled key in `GEMINI_API_KEY`; image models have no free tier.

## Risks & Mitigations

| Risk | Mitigation |
|:-----|:-----------|
| **Changing the default model alters output for users who relied on 2.0 behavior.** | The 2.0 model is already dead (404), so there is no working baseline to preserve — the change can only fix, not regress. `--model`/config override remains for anyone who wants a specific model. |
| **API silently ignores invalid aspect/resolution** (E1 finding) → users get wrong sizes thinking it worked. | Client-side enum validation (Issue 1.3) rejects bad values before the call. |
| **JPEG response written to a `.png` name** — concrete today in MCP (`server.go:187,259,325` hardcode `image/png`) and reachable on the CLI via `-o foo.png`. | Issue 1.4 corrects the extension to the response mimeType (warn + JSON requested-vs-actual) and fixes the three MCP call sites; Issue 5.2 asserts both CLI and MCP paths. |
| **`--quality` vs `--model` precedence ambiguity** (operator picked document-only Pro, then a quality alias). | Fixed precedence `--model` > `--quality` > config > default via cobra `Changed()`; intra-config `model` > `quality` (Issue 2.2/2.3). |
| **Pricing/tier surprise for users** (image models are paid-only). | README pricing/not-free callout (Issue 4.1); keep flash (cheaper) as default. |
| **Model-id duplication re-drifts** (`client.go` ↔ README/IG/EDD) — the exact class that caused this bug. | Issue 4.3 adds a `gemini-source` DRIFT-CHECK node + `value-equal` edges (manifest re-approval); Issue 5.3 runs drift-check against it. |
| **Existing users with `model: gemini-2.5-flash-image`** could fear a forced upgrade. | Backward-compatible: precedence preserves any config `model`; only the empty/dead-default case changes. Documented (Issue 4.1). |
| **`go:embed` cannot reach repo-root `skills/` from `cmd/naba`** → build break. | Embed package lives at repo root (scope note); Issue 6.1 isolates and verifies this first, before the command logic (6.2). |
| **Removing `install.{sh,py}` strands users/docs that reference it** (README install section, plan-001/002 work). | Issue 6.3 keeps a thin `install.sh` shim option and updates all references; README install section rewritten to `naba skills install` (Issue 4.1). |
| **`naba doctor`'s live key check makes a network call** (could fail offline / on a bad key). | The check is cheap (`models.list`, no image cost) and reports `warn`/`fail` gracefully; doctor degrades to presence-only when offline rather than erroring hard. |
| **Embedded skills drift from `skills/` source** (binary ships stale skills). | `skills status` compares installed-vs-embedded; embed is rebuilt every `go build`; Issue 5.1 asserts embedded-tree integrity against `skills/`. |
| **Integrity marker breaks the SKILL.md frontmatter parse or the hash round-trip** (marker counted in its own hash → never matches). | Marker injected **after** the YAML frontmatter (HTML comment, valid GFM, ignored by skill loaders); hash computed over the canonical tree **excluding** the marker line; Issue 5.1 tests the install→strip→re-hash round-trip and a tamper case (modify a file → `status`/`doctor` report modified). |
| **Plan scope is now two themes (model + distribution)** — larger review/blast radius. | Epics are independent (Epic 6/7 don't depend on Epic 1's API work except doctor's live check); operator may split at approval if preferred. |

## Success Criteria

1. `internal/gemini/client.go` default is `gemini-3.1-flash-image`; a fresh install with no
   config generates successfully (no 404).
2. `--aspect` and `--resolution` on all generative commands (generate/edit/restore/pattern/
   diagram/story) produce the requested `imageConfig` and are validated client-side (invalid
   values rejected with `ExitUsage`); `icon --size` (px) is unchanged.
3. `--quality {fast,high}` and config `quality`/`aspect`/`resolution` work with the
   documented precedence (`--model` > `--quality` > config > default; intra-config
   `model` > `quality`), implemented with cobra `Changed()`.
4. `gemini-3-pro-image` is selectable via `--model` and documented.
5. MCP image tools expose aspect/resolution/quality (via the refactored
   `resolveClient`/helpers) and write correctly-extensioned files.
6. JPEG responses land on disk with a matching extension across the CLI (generate/edit/
   restore/pattern/diagram/story **and** icon), and MCP: a mismatched `-o`/format extension
   is corrected and a warning emitted. The CLI JSON `Result` reports `requested_format`/
   `actual_format`; the MCP tool result reports the corrected path + actual format.
7. README documents the new flags, the pro model, the output-extension behavior, the
   `icon --size` vs `imageSize` distinction, a **pricing / not-free + model↔quality**
   callout, **and the `naba skills` / `naba doctor` commands** (replacing the `install.sh`
   instructions); `IG/configuration.md`, `IG/image-generation.md`, `IG/output-handling.md`,
   `IG/mcp-server.md`, and `EDD/CORE.md` reflect the new default, config keys, and behavior;
   existing `model:` configs keep working.
8. **`naba skills install`** writes the binary-embedded skill tree to user scope (and to a
   `--target`/`--scope`/`--surface`) and injects the integrity marker (scope #10) into the
   deployed `SKILL.md`; `upgrade`/`remove` work and `upgrade` prunes stale files; `status`
   reports up-to-date / complete / unmodified from the marker + tree hash; `install.{sh,py}`
   is superseded (removed or a thin shim). The skill files come from `go:embed` (offline).
   The repo source `skills/naba/SKILL.md` carries no marker.
9. **`naba doctor`** reports pass/warn/fail for: skills installed **and matching the embedded
   binary** (integrity marker present, `tree` hash == embedded hash, complete + unmodified —
   scope #10), key present, key live-valid (`models.list`), configured model reachable
   (**`models.list` intersection; `fail` if the model is absent**), config parseable —
   non-zero exit on fail.
9a. No `install.{sh,py}` references remain in `README.md`, `AGENTS.md`, `skills/naba/README.md`,
   or the DRIFT-CHECK manifest (§4/§5/referencers); the plan-002 migration note gives a working
   manual removal path for legacy `/naba-*` installs. Verified by `drift-check` (5.3) + grep.
10. Tests cover the default-model constant, imageConfig marshaling, enum validation, model
    precedence, extension reconciliation, embedded-skills integrity, and `skills`/`doctor`
    logic; live smoke passes on flash + pro (CLI and MCP) with correct jpeg output and a
    green `naba doctor`; `markdown-lint` clean and `drift-check` passes against the
    re-approved manifest (model-id + installer-node changes).
