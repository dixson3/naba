# /naba story — sequential image series

**Tier:** inline. Generate a sequential image series from a narrative prompt. `naba story`
is a **single** CLI invocation even though it emits multiple frames.

## Usage

```
/naba story <prompt> [--steps <n>] [--style <style>] [--transition <type>] [--layout <format>]
```

## Workflow

1. Refine the prompt: write a **narrative arc**, not individual frame descriptions. Naba
   splits the story into `--steps` frames automatically.
2. Run: `naba story "<narrative prompt>" [--steps <n>] [--style <style>] [--transition <type>] [--layout <format>]`
3. Present all output paths in sequence; offer to adjust pacing/style, or route per-frame
   edits to `/naba storyboard`.

## Flags

| Flag | Default | Values |
|------|---------|--------|
| `--steps` | 4 | 2–8 |
| `--style` | consistent | consistent, evolving |
| `--transition` | smooth | smooth, dramatic, fade |
| `--layout` | separate | separate, grid, comic |

(Plus the global flags in SKILL.md.)

## Notes

Write the **narrative arc**, not individual frames. Good: "a seed growing into a towering oak
tree through the seasons". Use `--transition` to control how frames relate visually. For
per-frame edits after generation, use `/naba storyboard`.

## Examples

```bash
naba story "a seed growing into a towering oak tree through the seasons"
naba story "sunrise to sunset over a desert canyon" --steps 3 --transition dramatic
naba story "a robot learning to paint" --steps 6 --layout comic --style evolving
```
