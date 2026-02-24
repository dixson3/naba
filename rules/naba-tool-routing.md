# Naba CLI Command Routing

When the user requests image-related work, use this decision tree to select the correct `naba` subcommand.

## Decision Tree

1. **Does the user have an existing image to work with?**
   - Yes, and they want to **modify** it -> `naba edit <file> <prompt>`
   - Yes, and they want to **enhance/restore** it -> `naba restore <file> [prompt]`
   - No -> continue to step 2

2. **What type of image do they need?**
   - App icon or logo -> `naba icon <prompt>`
   - Seamless pattern or texture -> `naba pattern <prompt>`
   - Sequential image series or storyboard -> `naba story <prompt>`
   - Technical or architectural diagram -> `naba diagram <prompt>`
   - General image -> `naba generate <prompt>`

3. **Do they need a coordinated set of brand assets?**
   - Yes (icon + pattern + hero image) -> use the `brand_kit` skill (runs icon + pattern + generate in sequence)

4. **Do they need a storyboard with per-frame edits?**
   - Yes -> use the `storyboard` skill (runs story, then edit on individual frames)

## Global Flags (Available on All Commands)

| Flag | Short | Purpose |
|------|-------|---------|
| `--output` | `-o` | Output file path or directory |
| `--json` | | Structured JSON output (auto-enabled when piped) |
| `--quiet` | `-q` | Suppress progress output |
| `--model` | `-m` | Override Gemini model |
| `--preview` | | Open result in system viewer |

## Command-Specific Flags

### generate
| Flag | Short | Default | Values |
|------|-------|---------|--------|
| `--style` | `-s` | (none) | photorealistic, watercolor, oil-painting, sketch, pixel-art, anime, vintage, modern, abstract, minimalist |
| `--count` | `-n` | 1 | 1-8 |
| `--seed` | | 0 | Any integer |
| `--format` | | separate | grid, separate |
| `--variation` | `-v` | (none) | lighting, angle, color-palette, composition, mood, season, time-of-day |

### edit
Positional args: `<file> <prompt>`. No command-specific flags beyond `--preview`.

### restore
Positional args: `<file> [prompt]`. Prompt is optional. No command-specific flags beyond `--preview`.

### icon
| Flag | Default | Values |
|------|---------|--------|
| `--style` | modern | flat, skeuomorphic, minimal, modern |
| `--size` | 256 | Any integer (repeatable for multiple sizes) |
| `--format` | png | png, jpeg |
| `--background` | transparent | transparent, white, black, or color name |
| `--corners` | rounded | rounded, sharp |

### pattern
| Flag | Default | Values |
|------|---------|--------|
| `--style` | abstract | geometric, organic, abstract, floral, tech |
| `--colors` | colorful | mono, duotone, colorful |
| `--density` | medium | sparse, medium, dense |
| `--tile-size` | 256x256 | Any dimension string |
| `--repeat` | tile | tile, mirror |

### story
| Flag | Default | Values |
|------|---------|--------|
| `--steps` | 4 | 2-8 |
| `--style` | consistent | consistent, evolving |
| `--transition` | smooth | smooth, dramatic, fade |
| `--layout` | separate | separate, grid, comic |

### diagram
| Flag | Default | Values |
|------|---------|--------|
| `--type` | flowchart | flowchart, architecture, network, database, wireframe, mindmap, sequence |
| `--style` | professional | professional, clean, hand-drawn, technical |
| `--layout` | hierarchical | horizontal, vertical, hierarchical, circular |
| `--complexity` | detailed | simple, detailed, comprehensive |
| `--colors` | accent | mono, accent, categorical |
