# Generate Pattern

Generate seamless patterns and textures using the naba CLI.

## Usage

```
/pattern <prompt> [--style <style>] [--colors <scheme>] [--density <level>] [--tile-size <dim>] [--repeat <method>]
```

## Workflow

1. **Validate environment**:
   ```bash
   command -v naba || echo "ERROR: naba not found on PATH"
   ```

2. **Refine the prompt**: Describe the **motif and feel**, not tiling mechanics. Apply the prompt engineering guidance below.

3. **Build and run the command**:
   ```bash
   naba pattern "<refined prompt>" [--style <style>] [--colors <scheme>] [--density <level>]
   ```

4. **Present the result**: Show the output file path. Use the Read tool to display the generated pattern.

5. **Offer iteration**: Ask if the user wants density, color, or style adjustments.

## Flags

| Flag | Default | Values |
|------|---------|--------|
| `--style` | abstract | geometric, organic, abstract, floral, tech |
| `--colors` | colorful | mono, duotone, colorful |
| `--density` | medium | sparse, medium, dense |
| `--tile-size` | 256x256 | Any dimension string |
| `--repeat` | tile | tile, mirror |
| `--output` | (auto) | Output file path |
| `--preview` | false | Open result in system viewer |

## Examples

```bash
# Floral pattern
naba pattern "tropical leaves with monstera and palm fronds" --style floral --colors colorful

# Geometric monochrome
naba pattern "interlocking hexagons" --style geometric --colors mono --density dense

# Tech-inspired
naba pattern "circuit board traces and microchip elements" --style tech --colors duotone
```

## Prompt Engineering

Describe the **motif and feel**, not the tiling mechanics. Good: "tropical leaves with monstera and palm fronds". The `--style`, `--colors`, and `--density` flags handle the technical pattern attributes.

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
