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

2. **Refine the prompt**: Describe the **motif and feel**, not tiling mechanics. Apply guidance from the naba-image-prompts rule.

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
