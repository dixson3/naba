# /naba generate — image from a text prompt

**Tier:** inline. General-purpose image creation from a text description.

## Usage

```
/naba generate <prompt> [--style <style>] [--count <n>] [--seed <int>] [--variation <type>]
```

## Workflow

1. Refine the prompt per the SKILL.md prompt-engineering order (subject + composition +
   style + lighting + details).
2. Run: `naba generate "<refined prompt>" [--style <style>] [--count <n>] [--seed <int>] [--variation <type>]`
3. Present the output path(s); offer iteration.

## Flags

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--style` | `-s` | (none) | photorealistic, watercolor, oil-painting, sketch, pixel-art, anime, vintage, modern, abstract, minimalist |
| `--count` | `-n` | 1 | Number of variations (1–8) |
| `--seed` | | 0 | Seed for reproducible output |
| `--format` | | separate | grid, separate |
| `--variation` | `-v` | (none) | lighting, angle, color-palette, composition, mood, season, time-of-day |

(Plus the global flags in SKILL.md.)

## Notes

Prompts can be descriptive and open-ended. Use `--style` to anchor the visual treatment and
`--variation` for systematic exploration of lighting, angle, color-palette, composition,
mood, season, or time-of-day.

## Examples

```bash
naba generate "a red apple on a wooden table, soft studio lighting"
naba generate "mountain landscape at sunset" --style watercolor
naba generate "portrait of a robot" --count 3 --style anime
naba generate "still life with flowers" --variation lighting --count 4
```
