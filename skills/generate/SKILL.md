# Generate Image

Generate an image from a text prompt using the naba CLI.

## Usage

```
/generate <prompt> [--style <style>] [--count <n>] [--seed <int>] [--variation <type>]
```

## Workflow

1. **Validate environment**:
   ```bash
   command -v naba || echo "ERROR: naba not found on PATH"
   ```

2. **Refine the prompt**: Apply prompt engineering guidance from the naba-image-prompts rule. Structure as: subject + composition + style + lighting + details.

3. **Build and run the command**:
   ```bash
   naba generate "<refined prompt>" [--style <style>] [--count <n>] [--seed <int>] [--variation <type>]
   ```

4. **Present the result**: Show the output file path(s) to the user. Use the Read tool to display the generated image if the user wants to preview it.

5. **Offer iteration**: Ask if the user wants adjustments — different style, composition changes, or variations.

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
