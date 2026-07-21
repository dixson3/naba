# naba — MCP Surface Specification

Clause IDs (`SPEC-<AREA>-NNN`) are stable and are never renumbered; append only.

## §11 MCP surface (SPEC-MCP)

- **SPEC-MCP-001** [PINNED] Server identity `naba` + version; stdio transport; tool and
  resource capabilities registered (no list-changed notifications). The resource capability
  covers both the `file:///{path}` template (SPEC-MCP-012) and the concrete skill resources
  (SPEC-MCP-014) — both are advertised under the single already-enabled `resources`
  capability, so no handshake change is introduced by the skill surface.
- **SPEC-MCP-002** [PINNED] Exactly **8 tools**: `generate_image`, `edit_image`,
  `restore_image`, `generate_icon`, `generate_pattern`, `generate_story`,
  `generate_diagram`, `list_images`. Tool/param inventory, enums, defaults, and the **base**
  descriptions are [PINNED] verbatim per the tables below. The seven **generation** tools' served
  descriptions are the pinned base **plus** the uniform lazy-guidance pointer suffix defined in
  SPEC-MCP-016 (`list_images`, the utility tool, carries no pointer).
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

### §11.1 Skills as MCP resources — lazy loading (SPEC-MCP-014/015)

- **SPEC-MCP-014** [NEW — amended plan-011] `resources/list` enumerates the **embedded skill
  tree** as concrete MCP resources so a client discovers skills cheaply and fetches instruction
  content on demand. The tree served here is the **`mcp/` render** — the **MCP-authored** variant
  produced by the `build.rs` two-tree render (SPEC-EMBED-005): the `SKILL.md` `{% if mcp %}` guide
  body **plus** the mcp-only `skills/<name>/mcp/…` subtree, with the CLI `commands/*.md` and
  `README.md` **excluded**. It is **not** the CLI-flavored `cli/` tree that `skills install`
  deploys; this fixes the earlier behavior where the MCP surface served CLI-oriented content
  (`/naba` slash commands, `--flags`). For each embedded skill `<name>` (from the binary's `mcp/`
  skill embed) it emits: a compact index resource — URI `skill://<name>`, name `<name> skills
  index`, MIME `text/markdown` — followed by one resource per file — URI `skill://<name>/<rel>`
  for each skill-relative path `<rel>` in the `mcp/` render (sorted by slash path; e.g.
  `SKILL.md`, `mcp/*.md`), name `<name>/<rel>`, MIME by extension (`.md` → `text/markdown`, else
  `text/plain`). The listing carries **URIs/metadata only — never file bodies** (the lazy-loading
  contract).
- **SPEC-MCP-015** [NEW] `resources/read` resolves the `skill://` scheme: `skill://<name>/<rel>`
  returns the embedded file content as `TextResourceContents` (`text`, MIME by extension —
  SPEC-MCP-014), served from the **`mcp/` render** embed (SPEC-EMBED-005), i.e. the MCP-authored
  variant rather than the `cli/` tree deployed on disk; `skill://<name>`
  returns a generated markdown index listing every `skill://<name>/<rel>` URI. An unknown
  skill or file → `resource not found: <uri>`. `file://` reads (SPEC-MCP-012) are unchanged;
  the eight tools (SPEC-MCP-002) are unaffected.
- **SPEC-MCP-016** [NEW — plan-011] **Tool-description lazy-guidance pointer.** Each of the seven
  **generation** tools (`generate_image`, `edit_image`, `restore_image`, `generate_icon`,
  `generate_pattern`, `generate_story`, `generate_diagram`) carries, appended to its SPEC-MCP-004…010
  pinned base description, a single **uniform** pointer line directing the client to fetch the
  `skill://naba` MCP resource for prompt-engineering and usage guidance. The suffix is
  identical across all seven tools and is the only mutation to the pinned descriptions; the tool
  `name` and `inputSchema` are unchanged. `list_images` (SPEC-MCP-011, the utility tool) carries
  **no** pointer. This keeps always-loaded tool context minimal — the detailed guidance is fetched
  on demand via `resources/read` (SPEC-MCP-015), not baked into the tool schemas.
