---
name: naba
description: >
  Create or transform images with the naba CLI, invoked as `/naba <subcommand> …`.
  TRIGGER when: /naba invoked, or the user wants to — generate/create/make an image,
  picture, or artwork from a text prompt (`generate`); edit/modify/alter an existing
  image, e.g. remove background, change colors, add or remove elements (`edit`);
  restore/enhance/repair/upscale/denoise/color-correct an existing image (`restore`);
  make an app icon, logo mark, or symbol, optionally at multiple sizes (`icon`); make a
  seamless/tileable pattern, texture, or background motif (`pattern`); render a technical
  diagram IMAGE — flowchart, architecture, network, database, wireframe, mindmap, or
  sequence (`diagram`); produce a sequential image series or visual narrative (`story`);
  produce a frame sequence with per-frame edits (`storyboard`); generate a coordinated SET
  of images in one pass — icon suite, asset pipeline, bulk run (`batch`); or generate a
  brand asset set — icon + pattern + hero (`brand-kit`).
  SKIP for: editable diagram SOURCE (d2/mermaid text) — use the `diagram-authoring` or
  `mermaid` skills; `naba diagram` produces a rendered image, not editable source.
user-invocable: true
skill-group: naba
depends-on-tool: [naba]
allowed-tools: [Bash, Read, Agent]
---

# naba

One skill for the whole `naba` image toolkit. Invoked as `/naba <subcommand> [args]`.
This file is the single source of truth for the router and the shared guidance below;
each subcommand's unique detail (usage, flags, examples) lives in `commands/<sub>.md`.

## Router

Parse `$ARGUMENTS`. The **first whitespace-delimited token** is the subcommand; the
remainder is its arguments. Resolve the subcommand against the dispatch table, then:

- **Inline subcommand** → `Read` the file `${CLAUDE_SKILL_DIR}/commands/<sub>.md` and follow
  it, applying the **Shared guidance** below. These are single `naba` CLI calls; run them
  directly with `Bash`.
- **Composite subcommand** → dispatch a subagent with the `Agent` tool (do **not** run the
  loop inline — it keeps intermediate per-image output out of this context). See
  **Composite dispatch** below.
- **`help`, an empty/missing subcommand, or an unknown token** → print the dispatch table
  (subcommand + one-line purpose) and stop. Do not guess a subcommand.

### Dispatch table

| Subcommand   | Tier      | Purpose |
| :----------- | :-------- | :------ |
| `generate`   | inline    | Image from a text prompt (general-purpose). |
| `edit`       | inline    | Modify an existing image with text instructions. |
| `restore`    | inline    | Restore/enhance an existing image (prompt optional). |
| `icon`       | inline    | App icon / logo mark, optionally multi-size. |
| `pattern`    | inline    | Seamless, tileable pattern or texture. |
| `diagram`    | inline    | Rendered technical diagram image. |
| `story`      | inline    | Sequential image series (one `naba story` call emits N frames). |
| `storyboard` | composite | `story` then per-frame `edit` refinement. |
| `batch`      | composite | A coordinated set of images over a list/spec. |
| `brand-kit`  | composite | Brand asset trio: `icon` + `pattern` + hero `generate`. |

`story` is **inline**: `naba story` is one CLI invocation even though it emits multiple
frames.

### Composite dispatch

For `storyboard`, `batch`, and `brand-kit`, spawn a subagent (`Agent`,
`subagent_type: general-purpose`) so the multi-call loop and its per-image output stay out
of this context. The subagent inherits **none** of this context and, once installed, this
skill lives at its deployed path — so the dispatch prompt MUST:

1. Pass the **absolute** path `${CLAUDE_SKILL_DIR}/commands/<sub>.md` and tell the subagent
   to `Read` it for the workflow.
2. Inline the **Shared guidance** below (prompt order, anti-patterns, global flags) into the
   prompt, or tell the subagent to `Read` `${CLAUDE_SKILL_DIR}/SKILL.md` for it.
3. Pass the user's arguments (the remainder of `$ARGUMENTS`).
4. Require a **compact summary** in return: the output file paths / a manifest, not the raw
   per-image logs.

The subagent's own grant supplies its file-writing tools (`Bash` for the `naba` calls and
`Write` for any manifest); this skill's `allowed-tools` only needs `Bash`/`Read` (inline
tier) and `Agent` (to spawn). Do not rely on the child having `Glob` — write each item to an
explicit `-o "<dir>/<name>.png"` and list results with `Bash` (`ls`) so the composite works
even when the subagent lacks a `Glob` grant.

## Shared guidance

Applies to **every** subcommand. Authored once here; `commands/<sub>.md` files carry only
per-command specifics and do not repeat this.

### Prompt engineering

Build prompts in this order: **subject + composition + style + lighting + details**.

1. **Subject** — the main focus; be specific ("a tabby cat on a wooden fence", not "a cat").
2. **Composition** — angle, framing, depth of field ("close-up", "bird's eye view",
   "centered with negative space").
3. **Style** — art style or medium; maps to `--style` where the subcommand has one.
4. **Lighting** — "golden hour", "soft diffused", "dramatic side lighting", "studio".
5. **Details** — color palette, mood, texture, atmosphere ("warm earth tones", "moody").

Some subcommands narrow this: `icon` focuses on the **symbol/concept** (naba handles
framing); `pattern` describes the **motif and feel** (flags handle tiling); `diagram`
describes the **system/process** (the `--type` flag picks the format); `edit` describes the
**desired change**, not the whole image; `restore` needs minimal or no prompt; `story`
writes a **narrative arc**, not per-frame text. The `commands/<sub>.md` file restates the
relevant narrowing.

### Anti-patterns

- **Avoid negatives** ("no text", "without watermarks") — they backfire; describe what you
  *do* want.
- **Avoid resolution specs in prompt text** ("4K", "1024x1024") — use CLI flags (`--size`,
  `--tile-size`) instead.
- **Keep prompts to 1–3 sentences** — beyond that, details compete and quality drops.
- **Avoid generic prompts** ("a beautiful landscape") — add specifics ("a misty fjord at
  dawn with a lone fishing boat").

### Global flags

Available on every `naba` subcommand:

| Flag | Short | Purpose |
| :--- | :---- | :------ |
| `--output`  | `-o` | Output file path or directory. |
| `--json`    |      | Structured JSON output (auto-enabled when piped). |
| `--quiet`   | `-q` | Suppress progress output. |
| `--model`   | `-m` | Override the Gemini model. |
| `--preview` |      | Open the result in the system viewer. |

## After any subcommand

Present the output file path(s); offer to `Read` an image to preview it; offer a relevant
iteration (different style, surgical edit, more frames, etc.).

---

**Spec / drift note:** the subcommand set and dispatch model here are specified in
`docs/specifications/IG/skills.md`; `DRIFT-CHECK.md` edge `e-skill-spec` keeps the two in
sync. When subcommands change, update this dispatch table, the `commands/` dir, the README
subcommand table, and the IG guide together.
