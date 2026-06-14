# /naba pattern — seamless pattern or texture

**Tier:** inline. Generate seamless, tileable patterns and textures.

## Usage

```
/naba pattern <prompt> [--style <style>] [--colors <scheme>] [--density <level>] [--tile-size <dim>] [--repeat <method>]
```

## Workflow

1. Refine the prompt: describe the **motif and feel**, not the tiling mechanics.
2. Run: `naba pattern "<refined prompt>" [--style <style>] [--colors <scheme>] [--density <level>]`
3. Present the output path; offer density, color, or style adjustments.

## Flags

| Flag | Default | Values |
|------|---------|--------|
| `--style` | abstract | geometric, organic, abstract, floral, tech |
| `--colors` | colorful | mono, duotone, colorful |
| `--density` | medium | sparse, medium, dense |
| `--tile-size` | 256x256 | Any dimension string |
| `--repeat` | tile | tile, mirror |

(Plus the global flags in SKILL.md.)

## Notes

Describe the **motif and feel**, not the tiling mechanics. Good: "tropical leaves with
monstera and palm fronds". The `--style`, `--colors`, and `--density` flags handle the
technical pattern attributes.

## Examples

```bash
naba pattern "tropical leaves with monstera and palm fronds" --style floral --colors colorful
naba pattern "interlocking hexagons" --style geometric --colors mono --density dense
naba pattern "circuit board traces and microchip elements" --style tech --colors duotone
```
