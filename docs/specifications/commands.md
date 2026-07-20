# naba — Commands Specification

Clause IDs (`SPEC-<AREA>-NNN`) are stable and are never renumbered; append only.

## §1 Command inventory (SPEC-INV)

- **SPEC-INV-001** [PINNED] The binary exposes exactly **15 real command groups**:
  `generate`, `edit`, `restore`, `icon`, `pattern`, `diagram`, `story`,
  `config` (subcommands `get`, `set`), `doctor`,
  `skills` (subcommands `install`, `upgrade`, `remove`, `status`, `preflight`), `provider`,
  `models`, `mcp`,
  `self` (subcommands `update`, `install`, `uninstall`; see distribution.md §17 SPEC-SELF),
  `version`. (Epic 2 added `provider` and `models`; plan-005 added `self`; the count is no
  longer fixed at 12.)
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
  `Provider: gemini or openrouter`). The help prose is [DIVERGENCE] (SPEC-DIVERGE-001/004) and
  lags the registry — the **runtime-valid** provider set is the registry (SPEC-PROVIDER-009):
  `gemini`, `openrouter`, `bedrock` (an unknown name is a usage error, exit 2). See §PROVIDER
  for resolution.
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

## §3 Command groups (SPEC-`<CMD>`)

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
- **SPEC-DOCTOR-002** [PINNED] Flags: `--scope` (string, `"user"`), `--harness` (string,
  default `claude-code`) with a deprecated hidden `--surface` alias (`claude → claude-code`,
  `agents → agents`), `--target` (string, `""`) — same semantics as `skills` (§3.11).
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
  | `--harness` | []string (repeatable) | `["claude-code"]` | `target harness (repeatable): claude-code \| opencode \| pi \| codex \| agents (default: claude-code). Give multiple times to install for several harnesses.` |
  | `--surface` | string | — | deprecated, hidden alias for `--harness` (`claude → claude-code`, `agents → agents`) |
  | `--target` | string | `""` | `override skills destination directory (takes precedence over scope/harness)` |
  | `--dry-run` | bool | `false` | `print the actions that would be taken; change nothing` |

- **SPEC-SKILLS-002** [PINNED] Subcommands (all no-args): `install` (`Install embedded
  skills to the resolved destination`); `upgrade` (`Rewrite installed skills from the
  embedded tree and prune stale files`); `remove` (`Remove installed skills from the
  destination`); `status` (`Report whether installed skills are up-to-date, complete, and
  unmodified`); `preflight` (`Fast skill-gate: validate provider key + skills/binary
  freshness`; see skills.md §18 SPEC-PREFLIGHT).
- **SPEC-SKILLS-003** [PINNED] Destination resolution `resolve_dest(scope, harness, target)`:
  non-empty `--target` wins; else the anchor (`$HOME` for scope `user`, git-root-else-cwd for
  scope `project`) is joined with the harness's scope-appropriate subpath per the
  SPEC-HARNESS-002 descriptor table (skills.md §19). A legacy/unknown harness value falls back
  to the uniform `.<value>/skills` layout (SPEC-HARNESS-004).
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
  Output (human / TTY): `naba <Version> (commit: <Commit>, built: <Date>)`. The concrete
  Version/Commit/Date **values** are [DIVERGENCE] (build-injected — §VERSION); the suite
  pins the *format*, normalizing the three fields. Under `--json` (incl. the SPEC-GLOBAL-003
  piped auto-enable) `version` emits the universal envelope (SPEC-JSON-006) instead:
  `{ "status": "ok", "data": { "version", "commit", "date", "host_triple", "line" } }`, where
  `line` is the human string above.
- **SPEC-VERSION-002** [PINNED] The `doctor` `version` check uses a *different* format:
  `naba <Version> (commit <Commit>, built <Date>)` (no colons). Preserve both formats as-is
  (do not unify).
