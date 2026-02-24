# Generate Story

Generate a sequential image series from a narrative prompt using the naba CLI.

## Usage

```
/story <prompt> [--steps <n>] [--style <style>] [--transition <type>] [--layout <format>]
```

## Workflow

1. **Validate environment**:
   ```bash
   command -v naba || echo "ERROR: naba not found on PATH"
   ```

2. **Refine the prompt**: Write a **narrative arc**, not individual frame descriptions. Naba automatically splits the story into the requested number of frames. Apply guidance from the naba-image-prompts rule.

3. **Build and run the command**:
   ```bash
   naba story "<narrative prompt>" [--steps <n>] [--style <style>] [--transition <type>] [--layout <format>]
   ```

4. **Present the result**: Show all output file paths. Use the Read tool to display the generated frames in sequence.

5. **Offer iteration**: Ask if the user wants to adjust pacing, style, or edit individual frames (suggest the `storyboard` skill for per-frame editing).

## Flags

| Flag | Default | Values |
|------|---------|--------|
| `--steps` | 4 | 2-8 |
| `--style` | consistent | consistent, evolving |
| `--transition` | smooth | smooth, dramatic, fade |
| `--layout` | separate | separate, grid, comic |
| `--output` | (auto) | Output file path |
| `--preview` | false | Open results in system viewer |

## Examples

```bash
# Basic story
naba story "a seed growing into a towering oak tree through the seasons"

# Short dramatic sequence
naba story "sunrise to sunset over a desert canyon" --steps 3 --transition dramatic

# Comic-style layout
naba story "a robot learning to paint" --steps 6 --layout comic --style evolving
```
