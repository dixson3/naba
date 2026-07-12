# naba — UX Contract Specification

**Status:** authoritative source of truth for the Go→Rust port (plan-004).
**Captured from:** Go `naba` at the plan-004 execute base (`main`).
**Clause IDs:** `SPEC-<AREA>-NNN`. IDs are stable — never renumber; append only. The
regression suite (`tests/parity/`) references these IDs; a CI traceability check asserts
every clause maps to at least one test case.

Legend for divergence markers:

- **[PINNED]** — the Rust port must reproduce this behavior byte-/semantics-identically;
  a parity test pins it.
- **[DIVERGENCE]** — a sanctioned intentional difference (see §DIVERGE). The suite pins
  the *inventory/semantics*, not a byte-identical snapshot.
- **[NEW]** — behavior introduced by the port (multi-provider); no Go counterpart.

---

## §1 Command inventory (SPEC-INV)

- **SPEC-INV-001** [PINNED] The binary exposes exactly **12 real command groups**:
  `generate`, `edit`, `restore`, `icon`, `pattern`, `diagram`, `story`,
  `config` (subcommands `get`, `set`), `doctor`,
  `skills` (subcommands `install`, `upgrade`, `remove`, `status`), `mcp`, `version`.
- **SPEC-INV-002** [PINNED] `storyboard`, `batch`, and `brand-kit` are **NOT** binary
  subcommands. They are skill-layer composites (the `/naba` skill orchestrates multiple
  real CLI calls). They are out of the binary parity surface and are protected only
  *transitively* — via the primitive-command goldens the composites are built from (M4).
  No CLI parity test targets them directly.
- **SPEC-INV-003** [PINNED] Root command: `Use: "naba"`, `Short: "Nanobanana image
  generation CLI"`, `Long: "Generate, edit, and transform images using Google Gemini
  AI."` The `Long` string is [DIVERGENCE] — help prose may be reworded to mention
  multi-provider (see SPEC-DIVERGE-001).
- **SPEC-INV-004** [PINNED] Root sets `SilenceUsage: true` and `SilenceErrors: true`;
  errors print to stderr via the top-level handler (see §EXIT), not cobra/clap's default
  usage dump.

---

## §2 Global flags & TTY autodetect (SPEC-GLOBAL)

- **SPEC-GLOBAL-001** [PINNED] Persistent (global) flags, exact names/shorthands/defaults:

  | Flag | Short | Type | Default | Help (verbatim) |
  |:--|:--|:--|:--|:--|
  | `--json` | — | bool | `false` | `Output structured JSON` |
  | `--output` | `-o` | string | `""` | `Output file path or directory` |
  | `--quiet` | `-q` | bool | `false` | `Suppress progress output` |
  | `--model` | `-m` | string | `""` | `Override Gemini model` |
  | `--no-input` | — | bool | `false` | `Disable interactive prompts` |

  The `--model` help string is [DIVERGENCE] (reworded to drop "Gemini", per multi-provider).
- **SPEC-GLOBAL-002** [NEW] Add a global `--provider` flag (string, default `""`, help
  `Provider: gemini or openrouter`). See §PROVIDER for resolution.
- **SPEC-GLOBAL-003** [PINNED] TTY autodetect at startup (root `PersistentPreRun`
  equivalent): if **stdout** is not a character device, force `--json` true. If **stdin**
  is not a character device, force `--no-input` true. Detection is on the stream mode
  (`os.ModeCharDevice`); the Rust port uses `IsTerminal` on stdout/stdin — semantically
  equivalent (a parity test pipes stdout and asserts JSON is emitted).
- **SPEC-GLOBAL-004** [PINNED] `--no-input` is auto-set from TTY but **never consumed** —
  no interactive-prompt code path exists. The port preserves this: the flag exists and is
  accepted, but drives no behavior. (Do not add interactive prompts.)
- **SPEC-GLOBAL-005** [PINNED] `--preview` is **not** a global flag; each image command
  declares its own `--preview` bool (default `false`, help `Open result in system viewer`,
  or `Open results in system viewer` for `story`). See per-command clauses.

---

## §3 Command groups (SPEC-<CMD>)

Prompt builders join their fragments with `". "` (period-space) unless stated otherwise.
All fragment strings below are **[PINNED] verbatim** — the mocked provider records the
outgoing prompt and the suite asserts it exactly.

### §3.1 generate (SPEC-GEN)

- **SPEC-GEN-001** [PINNED] `Use: "generate <prompt>"`, `Short: "Generate an image from a
  text prompt"`, exactly one positional arg.
- **SPEC-GEN-002** [PINNED] Flags:

  | Flag | Short | Type | Default | Help (verbatim) |
  |:--|:--|:--|:--|:--|
  | `--style` | `-s` | string | `""` | `Art style (photorealistic, watercolor, oil-painting, sketch, pixel-art, anime, vintage, modern, abstract, minimalist)` |
  | `--count` | `-n` | int | `1` | `Number of variations (1-8)` |
  | `--seed` | — | int | `0` | `Seed for reproducible output` |
  | `--format` | — | string | `"separate"` | `Output format (grid, separate)` |
  | `--variation` | `-v` | []string | `nil` | `Variation types (lighting, angle, color-palette, composition, mood, season, time-of-day)` |
  | `--preview` | — | bool | `false` | `Open result in system viewer` |

  plus the imageConfig flags `--aspect`/`--resolution`/`--quality` (§4).
- **SPEC-GEN-003** [PINNED] `--count` is **NOT range-validated in the CLI** (unlike MCP).
  Any int is accepted; the command loops `count` times. Preserve this asymmetry.
- **SPEC-GEN-004** [PINNED] `--seed` and `--format` are collected but **unused** — they do
  not affect the prompt, the request, or the output. Preserve (accept, ignore).
- **SPEC-GEN-005** [PINNED] Prompt builder `EnrichGeneratePrompt(prompt, style, variations)`:
  fragments = `prompt`; then `Style: <style>` iff style non-empty; then `Vary the <v>` for
  each variation `v`.
- **SPEC-GEN-006** [PINNED] Progress (stderr, unless `--quiet`): `Generating image %d/%d...`
  when count>1, else `Generating image...`.
- **SPEC-GEN-007** [PINNED] `Result.Params` carries imageConfig params (§4), `style`,
  `variations`, and (when count>1) `index`/`count`.

### §3.2 edit (SPEC-EDIT)

- **SPEC-EDIT-001** [PINNED] `Use: "edit <file> <prompt>"`, `Short: "Edit an existing image
  with instructions"`, exactly two positional args.
- **SPEC-EDIT-002** [PINNED] Flags: `--preview` + `--aspect`/`--resolution`/`--quality`.
- **SPEC-EDIT-003** [PINNED] Input file is `os.Stat`-checked; missing →
  `exitError(ExitFileIO, "input file not found: %s")` (exit 10).
- **SPEC-EDIT-004** [PINNED] Prompt: `EnrichEditPrompt(prompt)` = `"Edit this image: " + prompt`.
- **SPEC-EDIT-005** [PINNED] Progress: `Editing image...`. `Result.Params` = `{"input":
  <path>}` + imageConfig params. Routes through the image-input provider path.

### §3.3 restore (SPEC-RESTORE)

- **SPEC-RESTORE-001** [PINNED] `Use: "restore <file> [prompt]"`, `Short: "Restore or
  enhance an existing image"`, 1–2 positional args (prompt optional).
- **SPEC-RESTORE-002** [PINNED] Flags: `--preview` + `--aspect`/`--resolution`/`--quality`.
- **SPEC-RESTORE-003** [PINNED] Missing input → `ExitFileIO` `"input file not found: %s"`.
- **SPEC-RESTORE-004** [PINNED] Prompt `EnrichRestorePrompt(prompt)`: empty prompt →
  `"Restore and enhance this image. Improve quality, fix artifacts, and sharpen details."`;
  non-empty → `"Restore and enhance this image: " + prompt`.
- **SPEC-RESTORE-005** [PINNED] Progress: `Restoring image...`. Image-input provider path.

### §3.4 icon (SPEC-ICON)

- **SPEC-ICON-001** [PINNED] `Use: "icon <prompt>"`, `Short: "Generate app icons"`, one arg.
- **SPEC-ICON-002** [PINNED] Flags:

  | Flag | Type | Default | Help (verbatim) |
  |:--|:--|:--|:--|
  | `--style` | string | `"modern"` | `Visual style (flat, skeuomorphic, minimal, modern)` |
  | `--size` | []int | `[256]` | `Icon sizes in px (repeatable)` |
  | `--format` | string | `"png"` | `Output format (png, jpeg)` |
  | `--background` | string | `"transparent"` | `Background (transparent, white, black, or color name)` |
  | `--corners` | string | `"rounded"` | `Corner style (rounded, sharp)` |
  | `--preview` | bool | `false` | `Open result in system viewer` |

  plus `--quality` **only** — icon does **not** take `--aspect`/`--resolution` because its
  `--size` is canvas pixels, not `imageConfig.imageSize`.
- **SPEC-ICON-003** [PINNED] icon uses the **plain generate** provider path (no imageConfig
  is sent), looping once per `--size`.
- **SPEC-ICON-004** [PINNED] Output naming: `-o` empty → `icon-<size><ext>`; `-o` set with
  multiple sizes → `<base>-<size><ext>`, where `<ext>` = `ExtForFormat(--format)`
  (jpeg/jpg→`.jpg`, else `.png`). (The API still returns JPEG; §OUTPUT reconciles.)
- **SPEC-ICON-005** [PINNED] Progress: `Generating %dx%d icon...`. Params: `size, style,
  format, background, corners`.
- **SPEC-ICON-006** [PINNED] Prompt `EnrichIconPrompt(prompt, style, size, background,
  corners)`, fragments: `Generate an app icon: <prompt>`; `Style: <style>`;
  `Size: <size>x<size> pixels`; background `!= "transparent"` → `Background: <background>`
  else `Background: transparent`; corners `"rounded"` → `Rounded corners suitable for app
  icons` else `Sharp corners`; trailing `Clean, centered design suitable for use as an
  application icon`.

### §3.5 pattern (SPEC-PATTERN)

- **SPEC-PATTERN-001** [PINNED] `Use: "pattern <prompt>"`, `Short: "Generate seamless
  patterns and textures"`, one arg.
- **SPEC-PATTERN-002** [PINNED] Flags:

  | Flag | Type | Default | Help (verbatim) |
  |:--|:--|:--|:--|
  | `--style` | string | `"abstract"` | `Pattern style (geometric, organic, abstract, floral, tech)` |
  | `--colors` | string | `"colorful"` | `Color scheme (mono, duotone, colorful)` |
  | `--density` | string | `"medium"` | `Element density (sparse, medium, dense)` |
  | `--tile-size` | string | `"256x256"` | `Pattern tile size` |
  | `--repeat` | string | `"tile"` | `Tiling method (tile, mirror)` |
  | `--preview` | bool | `false` | `Open result in system viewer` |

  plus `--aspect`/`--resolution`/`--quality`.
- **SPEC-PATTERN-003** [PINNED] Progress: `Generating pattern...`. Params: `style, colors,
  density, tile_size, repeat` + imageConfig.
- **SPEC-PATTERN-004** [PINNED] Prompt `EnrichPatternPrompt(...)`, fragments: `Generate a
  seamless <style> pattern: <prompt>`; `Color scheme: <colors>`; `Element density:
  <density>`; `Tile size: <tileSize>`; repeat `"mirror"` → `Use mirror tiling for seamless
  repetition` else `Design for seamless tile repetition`.

### §3.6 diagram (SPEC-DIAGRAM)

- **SPEC-DIAGRAM-001** [PINNED] `Use: "diagram <prompt>"`, `Short: "Generate technical
  diagrams"`, one arg.
- **SPEC-DIAGRAM-002** [PINNED] Flags:

  | Flag | Type | Default | Help (verbatim) |
  |:--|:--|:--|:--|
  | `--type` | string | `"flowchart"` | `Diagram type (flowchart, architecture, network, database, wireframe, mindmap, sequence)` |
  | `--style` | string | `"professional"` | `Visual style (professional, clean, hand-drawn, technical)` |
  | `--layout` | string | `"hierarchical"` | `Layout (horizontal, vertical, hierarchical, circular)` |
  | `--complexity` | string | `"detailed"` | `Detail level (simple, detailed, comprehensive)` |
  | `--colors` | string | `"accent"` | `Color scheme (mono, accent, categorical)` |
  | `--preview` | bool | `false` | `Open result in system viewer` |

  plus `--aspect`/`--resolution`/`--quality`.
- **SPEC-DIAGRAM-003** [PINNED] Progress: `Generating diagram...`. Params: `type, style,
  layout, complexity, colors` + imageConfig.
- **SPEC-DIAGRAM-004** [PINNED] Prompt `EnrichDiagramPrompt(...)`, fragments: `Generate a
  <type> diagram: <prompt>`; `Visual style: <style>`; `Layout: <layout>`; `Level of detail:
  <complexity>`; `Color scheme: <colors>`; `Include clear labels and annotations`;
  `Professional quality suitable for documentation or presentations`.

### §3.7 story (SPEC-STORY)

- **SPEC-STORY-001** [PINNED] `Use: "story <prompt>"`, `Short: "Generate a sequential image
  series"`, one arg.
- **SPEC-STORY-002** [PINNED] Flags:

  | Flag | Type | Default | Help (verbatim) |
  |:--|:--|:--|:--|
  | `--steps` | int | `4` | `Number of frames (2-8)` |
  | `--style` | string | `"consistent"` | `Visual consistency (consistent, evolving)` |
  | `--transition` | string | `"smooth"` | `Transition style (smooth, dramatic, fade)` |
  | `--layout` | string | `"separate"` | `Output layout (separate, grid, comic)` |
  | `--preview` | bool | `false` | `Open results in system viewer` |

  plus `--aspect`/`--resolution`/`--quality`.
- **SPEC-STORY-003** [PINNED] `--steps` **is** validated: `< 2 || > 8` →
  `exitError(ExitUsage, "steps must be between 2 and 8")` (exit 2).
- **SPEC-STORY-004** [PINNED] `--layout` is collected but **unused** in the prompt (recorded
  in Params only). Preserve.
- **SPEC-STORY-005** [PINNED] Progress: `Generating frame %d/%d...`. Params: `step, total,
  style, transition, layout` + imageConfig.
- **SPEC-STORY-006** [PINNED] JSON output is **always** the array form (`PrintJSONMulti`),
  even for a single frame. See SPEC-JSON-003.
- **SPEC-STORY-007** [PINNED] Prompt `EnrichStoryPrompt(prompt, step, total, style,
  transition)`, fragments: `Generate frame <step> of <total> for a visual story: <prompt>`;
  style `"consistent"` → `Maintain consistent visual style, characters, and setting across
  all frames` else `Allow the visual style to evolve naturally across frames`; transition
  `"dramatic"` → `Use dramatic transitions between scenes`, `"fade"` → `Use subtle, fading
  transitions between scenes`, else `Use smooth, natural transitions between scenes`; step
  `1` → `This is the opening scene — establish the setting and characters`, step `== total`
  → `This is the final scene — bring the story to a conclusion`, else `This is scene <step>
  — continue developing the narrative`.

### §3.8 config (SPEC-CONFIG)

- **SPEC-CONFIG-001** [PINNED] `config` parent `Short: "Manage configuration"`; `Long` =
  `"Manage naba configuration.\n\nConfig file: <ConfigPath>\nValid keys: <joined ValidKeys>"`.
  The valid-keys list is [DIVERGENCE] — it gains `provider` (§6).
- **SPEC-CONFIG-002** [PINNED] `config get <key>` (one arg): load error →
  `ExitGeneral` `"load config: %v"`; unset key → `ExitGeneral` `"key %q is not set\n\nValid
  keys: %s"`; else prints the value.
- **SPEC-CONFIG-003** [PINNED] `config set <key> <value>` (two args): load error →
  `ExitGeneral` `"load config: %v"`; unknown key → `ExitUsage` `"unknown key %q\n\nValid
  keys: %s"`; save error → `ExitFileIO` `"save config: %v"`; success (unless `--quiet`)
  prints `Set %s = %s`.

### §3.9 doctor (SPEC-DOCTOR)

- **SPEC-DOCTOR-001** [PINNED] `Use: "doctor"`, no args, `Short: "Check naba's environment
  health (skills, API key, model, config)"`.
- **SPEC-DOCTOR-002** [PINNED] Flags: `--scope` (string, `"user"`), `--surface` (string,
  `"claude"`), `--target` (string, `""`) — same semantics as `skills` (§3.11).
- **SPEC-DOCTOR-003** [PINNED] Check statuses: `pass` / `warn` / `fail`. Each check is
  `{name, status, detail}`.
- **SPEC-DOCTOR-004** [PINNED] Checks, in order: `version`; `config`; `api_key`;
  `model_config` (only when model resolution errors); `api_live` (only when key present);
  `model_reachable`; `skills:<name>` per embedded skill. Detail strings are [PINNED]
  verbatim (see §ERR / provider notes); the `api_live`/`model_reachable` checks are
  provider-aware [DIVERGENCE] (they validate the *selected* provider — see SPEC-DOCTOR-006).
- **SPEC-DOCTOR-005** [PINNED] JSON envelope: `{"ok": bool, "failed": int, "checks":
  [{name,status,detail}]}`. Human output line: `[<symbol>] <name>: <detail>` with symbol
  `✓` (pass) / `!` (warn) / `✗` (fail); footer `All checks passed.` or `%d check(s)
  failed.`. Any fail → exit **1** (`ExitGeneral`, message `doctor: %d check(s) failed`).
- **SPEC-DOCTOR-006** [DIVERGENCE] The `api_key`/`api_live`/`model_reachable` checks become
  provider-aware: they report the key/liveness/model of the *resolved* provider (Gemini or
  OpenRouter). Detail wording may change; the suite pins `status` + envelope shape, not the
  exact detail string, for these three checks.

### §3.10 mcp (SPEC-MCP-CLI)

- **SPEC-MCP-CLI-001** [PINNED] `Use: "mcp"`, no args, `Short: "Start MCP server for AI tool
  integration"`, `Long: "Start a stdio-based Model Context Protocol server that exposes all
  image generation capabilities as MCP tools for AI assistants."`. Runs the stdio server
  (§MCP).

### §3.11 skills (SPEC-SKILLS)

- **SPEC-SKILLS-001** [PINNED] Parent `Short: "Install, upgrade, remove, or check naba's
  binary-embedded skills"`. Persistent flags:

  | Flag | Type | Default | Help (verbatim) |
  |:--|:--|:--|:--|
  | `--scope` | string | `"user"` | `user → $HOME; project → git root (else cwd)` |
  | `--surface` | string | `"claude"` | `claude → <root>/.claude/skills; agents → <root>/.agents/skills` |
  | `--target` | string | `""` | `override skills destination directory (takes precedence over scope/surface)` |
  | `--dry-run` | bool | `false` | `print the actions that would be taken; change nothing` |

- **SPEC-SKILLS-002** [PINNED] Subcommands (all no-args): `install` (`Install embedded
  skills to the resolved destination`); `upgrade` (`Rewrite installed skills from the
  embedded tree and prune stale files`); `remove` (`Remove installed skills from the
  destination`); `status` (`Report whether installed skills are up-to-date, complete, and
  unmodified`).
- **SPEC-SKILLS-003** [PINNED] Destination resolution `resolveDest(scope, surface, target)`:
  non-empty `--target` wins; else anchor = `$HOME` (scope `user`) or git-root-else-cwd
  (scope `project`), joined with `.<surface>/skills`.
- **SPEC-SKILLS-004** [PINNED] `install`/`upgrade` write each embedded file (dirs `0o755`,
  files `0o644`) and inject the skill marker into `SKILL.md` (§EMBED). `upgrade` also prunes
  dest files absent from the embed (`  pruned stale: %s`). Output: `OK: %s -> %s (%d
  files)`; dry-run: `(dry run) would write %d file(s) of %q -> %s`; success footer
  `Destination: %s`.
- **SPEC-SKILLS-005** [PINNED] `remove`: absent → `absent: %s`; dry-run → `(dry run) would
  remove %s`; else recursive delete + `removed: %s`.
- **SPEC-SKILLS-006** [PINNED] `status` line per skill: not installed → `<name>: not
  installed (<path>)`; else `<name>: <flags> (<path>)` where flags = `✓/✗up-to-date`,
  `✓/✗complete`, `✓/✗unmodified`. The **hash** underlying up-to-date/unmodified is
  [DIVERGENCE] (§EMBED / §DIVERGE) — the Rust embed hash may differ from Go's; the suite
  pins the *flag semantics* and *line shape*, not the hash value.

### §3.12 version (SPEC-VERSION)

- **SPEC-VERSION-001** [PINNED] `Use: "version"`, `Short: "Show version information"`.
  Output: `naba <Version> (commit: <Commit>, built: <Date>)`. The concrete
  Version/Commit/Date **values** are [DIVERGENCE] (build-injected — §VERSION); the suite
  pins the *format*, normalizing the three fields.
- **SPEC-VERSION-002** [PINNED] The `doctor` `version` check uses a *different* format:
  `naba <Version> (commit <Commit>, built <Date>)` (no colons). Preserve both formats as-is
  (do not unify).

---

## §4 Validation enums & imageConfig (SPEC-IMG)

- **SPEC-IMG-001** [PINNED] `ValidAspectRatios` (verbatim, order-preserving for help/enum):
  `1:1, 1:4, 1:8, 2:3, 3:2, 3:4, 4:1, 4:3, 4:5, 5:4, 8:1, 9:16, 16:9, 21:9`.
- **SPEC-IMG-002** [PINNED] `ValidImageSizes`: `512, 1K, 2K, 4K` (uppercase `K`; lowercase
  is rejected).
- **SPEC-IMG-003** [PINNED] imageConfig flags: `--aspect` (string, `""`, help `Aspect ratio
  for the generated image (e.g. 1:1, 16:9, 9:16, 21:9)`), `--resolution` (string, `""`, help
  `Image resolution (512, 1K, 2K, 4K)`).
- **SPEC-IMG-004** [PINNED] `--quality` flag: string, default `""`, help `Quality tier: fast
  (flash) or high (pro). Overridden by --model`. Help text is [DIVERGENCE] under
  multi-provider (see SPEC-PROVIDER-005).
- **SPEC-IMG-005** [PINNED] Both aspect and resolution empty → **no** `imageConfig` is sent
  (byte-identical bare request). Invalid aspect → `ExitUsage` `"invalid aspect ratio
  %q\n\nValid values: <joined>"`; invalid resolution → `ExitUsage` `"invalid resolution
  %q\n\nValid values: <joined>"`.
- **SPEC-IMG-006** [PINNED] `imageConfig` resolution precedence: flag (set) > config
  (`aspect`/`resolution`) > unset.
- **SPEC-IMG-007** [PINNED] naba-a3a carry-forward: `512` is **model-dependent** — image-size
  validation must be **provider/model-aware**, not a single global list. A model that does
  not support `512` must be rejected with a provider/model-specific message rather than
  passing the global `ValidImageSizes` gate and failing at the API. (Fixes naba-a3a; §5.)

---

## §5 Provider layer (SPEC-PROVIDER)

- **SPEC-PROVIDER-001** [NEW] naba supports two providers: **gemini** (current) and
  **openrouter** (new). Every image path (`generate`, `edit`, `restore`, and the composite
  commands) routes through the selected provider.
- **SPEC-PROVIDER-002** [PINNED] **Gemini** provider (port of the Go client): base URL
  `https://generativelanguage.googleapis.com/v1beta` (override via `GEMINI_BASE_URL`);
  endpoint `{base}/models/{model}:generateContent`, POST, headers `Content-Type:
  application/json` + `x-goog-api-key: <key>`; `generationConfig.responseModalities =
  ["TEXT","IMAGE"]` always; `imageConfig` under `generationConfig.imageConfig` with
  `aspectRatio`/`imageSize` (omitempty). Default model `gemini-3.1-flash-image`.
- **SPEC-PROVIDER-003** [PINNED] Gemini model constants: `DefaultModel = FlashModel =
  "gemini-3.1-flash-image"`, `ProModel = "gemini-3-pro-image"`.
- **SPEC-PROVIDER-004** [NEW] **OpenRouter** provider: bespoke client against the dedicated
  **`POST /api/v1/images`** Unified Image API. Bearer auth (`Authorization: Bearer <key>`).
  Base URL `https://openrouter.ai/api/v1` with a **`OPENROUTER_BASE_URL`** override
  (mirroring `GEMINI_BASE_URL`, for mockable tests). Request fields map from naba's
  imageConfig: `aspect_ratio` ← aspect, `resolution` ← imageSize (`512`/`1K`/`2K`/`4K`),
  native `quality` ← quality, and `input_references[]` for edit/restore image input.
  Response images are base64 in `data[].b64_json`. **CONFIRMED by the Issue 2.6 live-key
  smoke (2026-07-12):** the success envelope is `{ "created", "data": [{ "b64_json",
  "media_type" }], "usage": {...} }` (observed `media_type: "image/png"`); the error envelope
  is `{ "error": { "message", "code" } }`; `openrouter/auto` returns HTTP 404 `No endpoint
  found for model "openrouter/auto"` (SPEC-PROVIDER-006 validated — naba's early exit-2 guard
  means that call is never made). The `input_references[]` shape, the moderation-403 metadata,
  and the image-model discovery endpoint remain mock-validated (not exercised by the minimal
  smoke). Default image model slug `google/gemini-3.1-flash-image-preview`.
- **SPEC-PROVIDER-005** [PINNED/RESOLVED] **Per-provider `quality` semantics.** The trait
  carries the raw `--quality` value; each provider resolves it:
  - **Gemini**: quality → **model tier**. `fast` → `FlashModel`, `high` → `ProModel`. Any
    other value → `ExitUsage` `"invalid quality %q\n\nValid values: fast, high"`. `--quality`
    is overridden by an explicit `--model`.
  - **OpenRouter**: quality → the **native `quality` request parameter** on `/api/v1/images`;
    it does **NOT** swap the model. The model slug is selected independently (`--model` /
    config `model` / the default slug). Therefore `--provider openrouter --quality high`
    means: keep the resolved OpenRouter model slug, and pass `quality: high` to the API.
    (The `fast`/`high` vocabulary is preserved as the cross-provider surface; OpenRouter
    maps it onto its native quality parameter.)
- **SPEC-PROVIDER-006** [NEW] `openrouter/auto` **cannot generate images** (text-only
  router) and must **never** be selected for an image path — not as a default, not via
  autodetect. It is reserved for a possible future text path only. A request that would
  route image generation through `auto` is rejected.
- **SPEC-PROVIDER-007** [NEW] **Provider/model resolution precedence** (the selector
  factory): **CLI flags > config (`provider`/`model`) > env-key autodetect > built-in
  fallback**. Rules:
  - Env autodetect: only `GEMINI_API_KEY` present → **gemini** (+ Gemini default model);
    only `OPENROUTER_API_KEY` present → **openrouter** (+ default slug).
  - **Multiple keys + no config default → openrouter** with
    `google/gemini-3.1-flash-image-preview` (never `auto` for images).
  - **`--model` on the CLI requires `--provider`** — a model name alone is ambiguous across
    providers; `--model` without `--provider` is a usage error (`ExitUsage`, 2).
- **SPEC-PROVIDER-008** [NEW/INTENTIONAL — Concern 6] The multi-key default is an
  **intentional precedence outcome**: a user who already has `GEMINI_API_KEY` set and then
  adds `OPENROUTER_API_KEY` (with no `provider` in config) is **rerouted to OpenRouter**.
  This is documented, not a bug. The mitigation for a user who wants to stay on Gemini is to
  pin `provider: gemini` in config (config beats autodetect). SPEC and the `naba` skill/docs
  must call this out explicitly (Issue 5.2).

---

## §6 Config schema & precedence (SPEC-CFGSCHEMA)

- **SPEC-CFGSCHEMA-001** [PINNED+NEW] Config file `config.yaml` at `NABA_CONFIG_DIR` (if set)
  else `<home>/.config/naba`. Keys (YAML, all omitempty):
  `api_key`, `model`, `default_output_dir`, `aspect`, `resolution`, `quality`, **`provider`
  [NEW]**. `ValidKeys()` returns exactly this set (order: api_key, model, provider,
  default_output_dir, aspect, resolution, quality — `provider` is [NEW], placement is
  [DIVERGENCE]).
- **SPEC-CFGSCHEMA-002** [PINNED] Missing config file → zero-value config, no error. `Save()`
  mkdir `0o755`, file `0o644`.
- **SPEC-CFGSCHEMA-003** [PINNED] API-key precedence: `GEMINI_API_KEY` env > config
  `api_key`. [NEW] The OpenRouter key is `OPENROUTER_API_KEY` env (there is no config
  `openrouter_api_key` key in this port — keys stay env-only for OpenRouter unless a later
  plan adds one).
- **SPEC-CFGSCHEMA-004** [PINNED] Output-dir precedence: `NABA_OUTPUT_DIR` env > config
  `default_output_dir` > XDG default `<home>/.local/share/naba/images`.
- **SPEC-CFGSCHEMA-005** [PINNED] **CLI-vs-MCP output-dir asymmetry.** The **CLI** image
  commands do **NOT** consult `NABA_OUTPUT_DIR`/`default_output_dir`/the XDG default — they
  write to `-o` (file or dir) or auto-name in **CWD**. `NABA_OUTPUT_DIR` and the XDG default
  are consumed **only by the MCP server**. Preserve this asymmetry exactly.
- **SPEC-CFGSCHEMA-006** [PINNED] Model precedence in config (`ResolveModel`): `model` key >
  `quality`→model > unset. Invalid config `quality` → `"invalid quality %q in config (valid:
  fast, high)"`. Full CLI model precedence: `--model` (set, non-empty) > `--quality` (set) >
  config `ResolveModel` > provider default.

---

## §7 Exit-code matrix (SPEC-EXIT)

- **SPEC-EXIT-001** [PINNED] Exit codes: `1` General, `2` Usage, `3` Auth, `4` RateLimit,
  `5` API, `10` FileIO.
- **SPEC-EXIT-002** [PINNED] Dispatch: on error the top-level handler prints the error to
  stderr and exits with the error's `ExitCode()` if it implements one, else **1**.
- **SPEC-EXIT-003** [PINNED] **Cobra/clap parse errors exit 1**, not 2. With
  `SilenceErrors/SilenceUsage`, a flag/arg parse error has no `ExitCode()` and falls to the
  default 1. Only explicit in-code usage errors exit 2 (e.g. `steps must be between 2 and
  8`, `unknown key`, invalid aspect/resolution/quality, `--model` without `--provider`).
  The Rust port must replicate: argument-parse failures exit **1**, not clap's default 2.
- **SPEC-EXIT-004** [PINNED] HTTP→exit mapping (Gemini): 401/403 → 3 (Auth); 429 → 4
  (RateLimit); ≥500 → 5 (API, message rewritten); other non-2xx → 5. Prompt-block / no-image
  → 5. Input-image read failure → 10 (FileIO). OpenRouter maps analogously: 401/403 → 3,
  429 → 4 (honoring `Retry-After`), moderation-403/content-policy → 3-or-5 per §ERR, ≥500 →
  5.
- **SPEC-EXIT-005** [PINNED] `doctor` with any failing check exits **1** (not 2), message
  `doctor: %d check(s) failed`.

---

## §8 JSON output shapes (SPEC-JSON)

- **SPEC-JSON-001** [PINNED] `Result` object (2-space-indented):

  ```json
  {
    "path": "string",
    "command": "string",
    "prompt": "string",
    "elapsed_ms": 0,
    "params": { },
    "requested_format": "string",
    "actual_format": "string"
  }
  ```

  `params` is omitempty; `requested_format`/`actual_format` omitempty.
- **SPEC-JSON-002** [PINNED] Single-image commands emit a **single object** when there is
  one result, a **JSON array** when there is more than one.
- **SPEC-JSON-003** [PINNED] `story` **always** emits a JSON array, even for one frame.
- **SPEC-JSON-004** [PINNED] `doctor` JSON envelope: `{"ok": bool, "failed": int, "checks":
  [{"name","status","detail"}]}`.
- **SPEC-JSON-005** [PINNED] Nondeterministic fields the suite **normalizes** before
  comparison: `elapsed_ms`, timestamped auto-names/paths, version/commit/date. The parity
  harness has a normalizer (Issue 1.2) that canonicalizes these.

---

## §9 Verbatim error strings (SPEC-ERR)

All [PINNED] unless the wording is provider-dependent (marked [DIVERGENCE]).

- **SPEC-ERR-001** API key unset (CLI image cmds): `GEMINI_API_KEY not set.\n\nSet it with:
  export GEMINI_API_KEY=<your-key>\nOr run: naba config set api_key <your-key>` → exit 3.
  [DIVERGENCE] under multi-provider the message names the selected provider's key
  (`OPENROUTER_API_KEY` when the provider is openrouter). The suite pins exit 3 + the
  "not set" shape, not the exact key name for the openrouter case.
- **SPEC-ERR-002** Input file missing (edit/restore): `input file not found: %s` → exit 10.
- **SPEC-ERR-003** story steps: `steps must be between 2 and 8` → exit 2.
- **SPEC-ERR-004** invalid aspect: `invalid aspect ratio %q\n\nValid values: <list>` → 2.
- **SPEC-ERR-005** invalid resolution: `invalid resolution %q\n\nValid values: <list>` → 2.
- **SPEC-ERR-006** invalid quality (flag/MCP): `invalid quality %q\n\nValid values: fast,
  high` → 2.
- **SPEC-ERR-007** invalid quality (config): `invalid quality %q in config (valid: fast,
  high)`.
- **SPEC-ERR-008** config get unset: `key %q is not set\n\nValid keys: <list>` → 1.
- **SPEC-ERR-009** config set unknown key: `unknown key %q\n\nValid keys: <list>` → 2.
- **SPEC-ERR-010** Gemini auth (401/403): `authentication failed: %s\n\nSet GEMINI_API_KEY
  or run: naba config set api_key <your-key>` → 3.
- **SPEC-ERR-011** rate limit (429): `rate limit exceeded: %s\n\nWait a moment and try
  again.` → 4.
- **SPEC-ERR-012** server (≥500): `Gemini server error: %s\n\nThis is a temporary issue. Try
  again shortly.` → 5. [DIVERGENCE] OpenRouter uses an analogous provider-named string.
- **SPEC-ERR-013** prompt blocked: `prompt blocked: %s` → 5.
- **SPEC-ERR-014** no images: `no images in response` → 5.
- **SPEC-ERR-015** read image file: `read image file %q: %v` → 10.
- **SPEC-ERR-016** [NEW] `--model` without `--provider`: usage error → exit 2.
- **SPEC-ERR-017** [NEW] OpenRouter moderation/content-policy (403): a content-policy error
  string → exit 3 (auth-class) or 5 per the live-key smoke; `Retry-After` honored on 429.

---

## §10 Config migration (SPEC-MIGRATE)

- **SPEC-MIGRATE-001** [NEW/RESOLVED — Concern 5] The schema change is
  **additive-optional**: `provider` (and `model`, already present) are optional keys. The
  **default migration is zero-rewrite** — absent keys resolve to defaults on read, and the
  user's hand-edited `config.yaml` (**including its comments**) is left untouched.
- **SPEC-MIGRATE-002** [NEW] A file **rewrite** is performed **only** if a genuine structural
  migration is later required. When it is, migration does a serde round-trip, writes a
  `.bak` backup first, is **idempotent**, and is graceful on
  empty/missing/malformed/already-new inputs (no data loss, no crash).
- **SPEC-MIGRATE-003** [NEW/ACCEPTED — Concern 5] A structural rewrite (if ever triggered)
  **loses YAML comments** (serde round-trip does not preserve them). This is an accepted
  loss, mitigated by the `.bak` backup and the zero-rewrite default. Documented here so it
  is not a surprise.
- **SPEC-MIGRATE-004** [NEW] YAML crate: use `serde_norway`/`yaml_serde`. **`serde_yml` is
  forbidden** (RUSTSEC-2025-0068).

---

## §11 MCP surface (SPEC-MCP)

- **SPEC-MCP-001** [PINNED] Server identity `naba` + version; stdio transport; tool and
  resource capabilities registered (no list-changed notifications).
- **SPEC-MCP-002** [PINNED] Exactly **8 tools**: `generate_image`, `edit_image`,
  `restore_image`, `generate_icon`, `generate_pattern`, `generate_story`,
  `generate_diagram`, `list_images`. Tool/param inventory, enums, defaults, and descriptions
  are [PINNED] verbatim per the tables below.
- **SPEC-MCP-003** [PINNED] Shared imageConfig options on the image tools: `aspect` (enum =
  `ValidAspectRatios`, desc `Aspect ratio (generationConfig.imageConfig.aspectRatio)`),
  `resolution` (enum = `ValidImageSizes`, desc `Image resolution
  (generationConfig.imageConfig.imageSize)`), `quality` (enum `fast`, `high`, desc `Quality
  tier: fast (gemini-3.1-flash-image) or high (gemini-3-pro-image)`). The `quality`
  description is [DIVERGENCE] under multi-provider.
- **SPEC-MCP-004** [PINNED] `generate_image`: desc `Generate an image from a text prompt`;
  `prompt` (required); `style` enum `photorealistic, watercolor, oil-painting, sketch,
  pixel-art, anime, vintage, modern, abstract, minimalist`; `variations` array enum
  `lighting, angle, color-palette, composition, mood, season, time-of-day`; `count` number
  default `1`, min `1`, max `8`; `seed` number; + aspect/resolution/quality. Validates
  `count 1..8` → `count must be between 1 and 8`; missing prompt → `missing required
  parameter: prompt`.
- **SPEC-MCP-005** [PINNED] `edit_image`: desc `Edit an existing image based on a text
  prompt`; `prompt` (required), `file` (required); + aspect/resolution/quality. Missing file
  → `missing required parameter: file`; file absent → `file not found: %s`.
- **SPEC-MCP-006** [PINNED] `restore_image`: desc `Restore or enhance an existing image`;
  `file` (required), `prompt` (optional); + aspect/resolution/quality.
- **SPEC-MCP-007** [PINNED] `generate_icon`: desc `Generate app icons in multiple sizes`;
  `prompt` (required); `sizes` array of numbers (item min `16`, max `1024`, default `[256]`);
  `style` default `modern` enum `flat, skeuomorphic, minimal, modern`; `background` default
  `transparent` (no enum); `corners` default `rounded` enum `rounded, sharp`; `format`
  default `png` enum `png, jpeg`; `quality` only (no aspect/resolution).
- **SPEC-MCP-008** [PINNED] `generate_pattern`: desc `Generate seamless patterns and
  textures`; `prompt` (required); `style` default `abstract` enum `geometric, organic,
  abstract, floral, tech`; `colors` default `colorful` enum `mono, duotone, colorful`;
  `density` default `medium` enum `sparse, medium, dense`; `size` default `256x256` (no
  enum); `repeat` default `tile` enum `tile, mirror`; + aspect/resolution/quality.
- **SPEC-MCP-009** [PINNED] `generate_story`: desc `Generate a sequence of images that tell a
  visual story`; `prompt` (required); `steps` number default `4` min `2` max `8`; `style`
  default `consistent` enum `consistent, evolving`; `transition` default `smooth` enum
  `smooth, dramatic, fade`; `layout` default `separate` enum `separate, grid, comic`; +
  aspect/resolution/quality. Validates `steps 2..8` → `steps must be between 2 and 8`.
- **SPEC-MCP-010** [PINNED] `generate_diagram`: desc `Generate technical diagrams and
  flowcharts`; `prompt` (required); `type` default `flowchart` enum `flowchart,
  architecture, network, database, wireframe, mindmap, sequence`; `style` default
  `professional` enum `professional, clean, hand-drawn, technical`; `layout` default
  `hierarchical` enum `horizontal, vertical, hierarchical, circular`; `complexity` default
  `detailed` enum `simple, detailed, comprehensive`; `colors` default `accent` enum `mono,
  accent, categorical`; + aspect/resolution/quality.
- **SPEC-MCP-011** [PINNED] `list_images` (**MCP-only**, no CLI counterpart — M1): desc `List
  recently generated images in the output directory`; `limit` number default `20`. Behavior:
  outDir empty → `no output directory configured`; limit<1 → treated as 20; dir missing →
  `No images found (directory does not exist)`; else filter `naba-*` with ext
  png/jpg/jpeg/gif/webp, newest-first by modtime, cap at limit; empty → `No images found`;
  one text content per path.
- **SPEC-MCP-012** [PINNED] Resource template: URI `file:///{path}`, name `Generated image
  file`, desc `Access a generated image by its file path`, MIME `image/*`. Reader strips
  `file://` and returns `BlobResourceContents` (base64); MIME by extension
  (png/jpg/jpeg/gif/webp else `application/octet-stream`).
- **SPEC-MCP-013** [PINNED] MCP tools write via the **MCP output-dir** resolution
  (`NABA_OUTPUT_DIR`/config/XDG default — SPEC-CFGSCHEMA-005), return a text path + `Format:
  <mime>` text + a `file://` resource link; multi-image tools return one entry per image.
  MCP errors use tool-level error results (not process exit). MCP missing-key message:
  `GEMINI_API_KEY not set. Set it with: export GEMINI_API_KEY=<your-key> or run: naba config
  set api_key <your-key>` [DIVERGENCE for the openrouter provider].

---

## §12 Skill-embed (SPEC-EMBED)

- **SPEC-EMBED-001** [PINNED] The binary embeds the `skills/` tree. Marker prefix `<!--
  naba-skills:`; marker format `<!-- naba-skills: v=<version> tree=<hash> -->` injected into
  each `SKILL.md` after its YAML frontmatter (else prepended); injection is idempotent.
- **SPEC-EMBED-002** [PINNED] Tree hash `hashTree`: sha256 over, per file sorted by
  skill-relative slash path, `write(relpath bytes) then write(file bytes)`; **no newline
  normalization**; `SKILL.md`'s marker line is stripped before hashing so embedded
  (marker-free) and deployed (marked) trees hash identically.
- **SPEC-EMBED-003** [PINNED] `status`/`doctor` compare: **UpToDate** = marker's `tree=`
  hash == `EmbeddedTreeHash(name)`; **Complete** = every embedded file present on disk;
  **Unmodified** = recomputed `DeployedTreeHash(destDir)` == `EmbeddedTreeHash(name)`;
  **Installed** = `SKILL.md` present.
- **SPEC-EMBED-004** [DIVERGENCE — Concern 4 / M4] The Rust port may **reproduce** Go's
  tree-hash byte-for-byte (so existing installs keep matching), **or** consciously adopt a
  different hash format and require a one-time post-cutover `naba skills upgrade` (Issue
  5.3). Either is acceptable; the choice is recorded in Issue 4.0. The parity suite pins the
  status **semantics** (up-to-date/complete/unmodified flags behave correctly against a
  freshly-installed tree), not the hash literal.

---

## §13 Version injection (SPEC-VERSION-BUILD)

- **SPEC-VERSION-BUILD-001** [DIVERGENCE] Go injects `Version`/`Commit`/`Date` via ldflags
  (`git describe --tags --always --dirty`, `git rev-parse --short HEAD`, UTC date). The Rust
  port injects the same three via `build.rs`/compile-time env (replacing ldflags — M3). The
  values are build-dependent; the suite normalizes them (SPEC-JSON-005) and pins only the
  output *format* (§VERSION).

---

## §14 Sanctioned divergence zones (SPEC-DIVERGE)

The port is a drop-in replacement **except** for the enumerated zones below. Every
divergence is captured by a SPEC clause and covered by a semantics-level (not
byte-snapshot) test.

- **SPEC-DIVERGE-001** Help text: cobra→clap rendering differs (usage layout, flag ordering,
  auto-generated sections). Root/`--model`/`--quality`/config-keys prose may be reworded for
  multi-provider. Tests assert flag *inventory* and enum membership, not full help snapshots.
- **SPEC-DIVERGE-002** Skill integrity hashes: Go embed → Rust embed (SPEC-EMBED-004).
- **SPEC-DIVERGE-003** Version strings: build-injected values (SPEC-VERSION-BUILD-001);
  normalized in tests.
- **SPEC-DIVERGE-004** Multi-provider additions: the `--provider` flag, the `provider`
  config key, provider-aware doctor checks, and provider-named error/help strings are [NEW]
  and have no Go counterpart — they are additive, not regressions.
- **SPEC-DIVERGE-005** The multi-key → OpenRouter reroute (SPEC-PROVIDER-008) is an
  intentional precedence outcome, documented, not a divergence-as-defect.
- **SPEC-DIVERGE-006** Everything **not** enumerated in §14 is [PINNED]: any observable
  difference outside these zones is a port defect, not a sanctioned divergence.
