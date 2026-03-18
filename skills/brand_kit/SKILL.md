# Brand Kit

Generate a coordinated set of brand assets (icon + pattern + hero image) using the naba CLI.

## Usage

```
/brand_kit <brand description> [--style <style>]
```

This is a **composite skill** that runs `naba icon`, `naba pattern`, and `naba generate` in sequence to produce a cohesive brand asset set.

## Workflow

1. **Validate environment**:
   ```bash
   command -v naba || echo "ERROR: naba not found on PATH"
   ```

2. **Gather brand details**: Ask the user for:
   - Brand name and concept
   - Color preferences (if any)
   - Style direction (modern, minimal, bold, etc.)
   - Target use cases (app, website, print)

3. **Generate the icon**:
   ```bash
   naba icon "<icon prompt based on brand concept>" --style <style> --size 64 --size 128 --size 256 --size 512
   ```

4. **Generate the pattern**:
   ```bash
   naba pattern "<pattern prompt inspired by brand motifs>" --colors <scheme> --style <style>
   ```

5. **Generate the hero image**:
   ```bash
   naba generate "<hero image prompt capturing brand essence>" --style <style>
   ```

6. **Present all assets**: Show all output file paths grouped by type. Use the Read tool to display each asset.

7. **Offer iteration**: Ask which assets need refinement and re-run individual commands as needed.

## Example Session

```bash
# Step 1: Icon — multiple sizes
naba icon "a minimalist wave crest" --style minimal --size 64 --size 128 --size 256 --size 512 --background transparent

# Step 2: Pattern — matching motif
naba pattern "flowing ocean waves and sea foam" --style organic --colors duotone --density medium

# Step 3: Hero image — brand showcase
naba generate "abstract ocean waves at golden hour, minimal composition with deep blues and warm highlights" --style modern
```

## Prompt Engineering

Build prompts in this order: **subject + composition + style + lighting + details**.

1. **Subject**: What is the main focus? Be specific — "a tabby cat sitting on a wooden fence" not "a cat"
2. **Composition**: Camera angle, framing, depth of field — "close-up shot", "bird's eye view", "centered with negative space"
3. **Style**: Art style or medium
4. **Lighting**: "golden hour", "soft diffused", "dramatic side lighting", "studio lighting"
5. **Details**: Color palette, mood, texture, atmosphere — "warm earth tones", "moody and atmospheric"

### Per-Command Guidance

**icon**: Focus on the **symbol or concept**, not composition. Naba handles icon framing. Use `--style` for visual treatment (flat, skeuomorphic, minimal, modern).

**pattern**: Describe the **motif and feel**, not the tiling mechanics. The `--style`, `--colors`, and `--density` flags handle the technical pattern attributes.

**generate**: General-purpose image creation. Prompts can be descriptive and open-ended. Use `--style` to anchor the visual treatment.

### Anti-Patterns

- **Avoid negatives**: "no text" or "without watermarks" often backfire. Instead, describe what you want.
- **Avoid resolution specs in prompts**: Use CLI flags (`--size`, `--tile-size`) instead of "4K" or "1024x1024" in the prompt text.
- **Avoid overly long prompts**: 1-3 sentences is the sweet spot. Beyond that, details compete and quality drops.
- **Avoid generic prompts**: "a beautiful landscape" produces generic results. Add specifics: "a misty fjord at dawn with a lone fishing boat".

## Command Flags

### Global Flags

| Flag | Short | Purpose |
|------|-------|---------|
| `--output` | `-o` | Output file path or directory |
| `--json` | | Structured JSON output (auto-enabled when piped) |
| `--quiet` | `-q` | Suppress progress output |
| `--model` | `-m` | Override Gemini model |
| `--preview` | | Open result in system viewer |

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

### generate

| Flag | Short | Default | Values |
|------|-------|---------|--------|
| `--style` | `-s` | (none) | photorealistic, watercolor, oil-painting, sketch, pixel-art, anime, vintage, modern, abstract, minimalist |
| `--count` | `-n` | 1 | 1-8 |
| `--seed` | | 0 | Any integer |
| `--format` | | separate | grid, separate |
| `--variation` | `-v` | (none) | lighting, angle, color-palette, composition, mood, season, time-of-day |
