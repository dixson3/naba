---
name: naba-generate
description: >
  Generate an image from a text prompt via the naba CLI — general-purpose image
  creation. TRIGGER when: /naba-generate invoked, or the user asks to create/make/generate
  an image, picture, or artwork from a text description. SKIP for: editing an existing
  image (/naba-edit), restoring/enhancing one (/naba-restore), app icons (/naba-icon), seamless
  patterns (/naba-pattern), sequential series (/naba-story), or technical diagrams (/naba-diagram).
user-invocable: true
skill-group: naba
depends-on-tool: [naba]
allowed-tools: [Bash, Read]
---
# Generate Image

Generate an image from a text prompt using the naba CLI.

## Usage

```
/naba-generate <prompt> [--style <style>] [--count <n>] [--seed <int>] [--variation <type>]
```

## Workflow

1. **Refine the prompt**: Apply the prompt engineering guidance below. Structure as: subject + composition + style + lighting + details.

2. **Build and run the command**:
   ```bash
   naba generate "<refined prompt>" [--style <style>] [--count <n>] [--seed <int>] [--variation <type>]
   ```

3. **Present the result**: Show the output file path(s) to the user. Use the Read tool to display the generated image if the user wants to preview it.

4. **Offer iteration**: Ask if the user wants adjustments — different style, composition changes, or variations.

## Flags

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--style` | `-s` | (none) | Art style: photorealistic, watercolor, oil-painting, sketch, pixel-art, anime, vintage, modern, abstract, minimalist |
| `--count` | `-n` | 1 | Number of variations (1-8) |
| `--seed` | | 0 | Seed for reproducible output |
| `--format` | | separate | Output format: grid, separate |
| `--variation` | `-v` | (none) | Variation type: lighting, angle, color-palette, composition, mood, season, time-of-day |
| `--output` | `-o` | (auto) | Output file path or directory |
| `--preview` | | false | Open result in system viewer |

## Examples

```bash
# Simple generation
naba generate "a red apple on a wooden table, soft studio lighting"

# With style
naba generate "mountain landscape at sunset" --style watercolor

# Multiple variations
naba generate "portrait of a robot" --count 3 --style anime

# Explore lighting variations
naba generate "still life with flowers" --variation lighting --count 4
```

## Prompt Engineering

Build prompts in this order: **subject + composition + style + lighting + details**.

1. **Subject**: What is the main focus? Be specific — "a tabby cat sitting on a wooden fence" not "a cat"
2. **Composition**: Camera angle, framing, depth of field — "close-up shot", "bird's eye view", "centered with negative space"
3. **Style**: Art style or medium — maps to `--style` flag values (photorealistic, watercolor, oil-painting, sketch, pixel-art, anime, vintage, modern, abstract, minimalist)
4. **Lighting**: "golden hour", "soft diffused", "dramatic side lighting", "studio lighting"
5. **Details**: Color palette, mood, texture, atmosphere — "warm earth tones", "moody and atmospheric"

General-purpose image creation. Prompts can be descriptive and open-ended. Use `--style` to anchor the visual treatment. Use `--variation` for systematic exploration of lighting, angle, color-palette, composition, mood, season, or time-of-day.

### Anti-Patterns

- **Avoid negatives**: "no text" or "without watermarks" often backfire. Instead, describe what you want.
- **Avoid resolution specs in prompts**: Use CLI flags (`--size`, `--tile-size`) instead of "4K" or "1024x1024" in the prompt text.
- **Avoid overly long prompts**: 1-3 sentences is the sweet spot. Beyond that, details compete and quality drops.
- **Avoid generic prompts**: "a beautiful landscape" produces generic results. Add specifics: "a misty fjord at dawn with a lone fishing boat".

## Global Flags

| Flag | Short | Purpose |
|------|-------|---------|
| `--output` | `-o` | Output file path or directory |
| `--json` | | Structured JSON output (auto-enabled when piped) |
| `--quiet` | `-q` | Suppress progress output |
| `--model` | `-m` | Override Gemini model |
| `--preview` | | Open result in system viewer |
