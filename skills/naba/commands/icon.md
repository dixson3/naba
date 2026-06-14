# /naba icon — app icon / logo mark

**Tier:** inline. Generate app icons, optionally at multiple sizes.

## Usage

```
/naba icon <prompt> [--style <style>] [--size <px>] [--background <bg>] [--corners <type>]
```

## Workflow

1. Refine the prompt: focus on the **symbol or concept** — do not describe framing or
   dimensions; naba handles those.
2. Run: `naba icon "<refined prompt>" [--style <style>] [--size <px>] [--background <bg>] [--corners <type>]`
   Repeat `--size` for multiple sizes (e.g. `--size 64 --size 128 --size 256`).
3. Present the output path(s); offer style or size adjustments.

## Flags

| Flag | Default | Values |
|------|---------|--------|
| `--style` | modern | flat, skeuomorphic, minimal, modern |
| `--size` | 256 | Any integer in px (repeatable) |
| `--format` | png | png, jpeg |
| `--background` | transparent | transparent, white, black, or color name |
| `--corners` | rounded | rounded, sharp |

(Plus the global flags in SKILL.md.)

## Notes

Focus on the **symbol or concept**, not composition (naba handles icon framing). Good: "a
lightning bolt with circuit traces". Bad: "a 256x256 icon centered on a white background of a
lightning bolt". Use `--style` for the visual treatment.

## Examples

```bash
naba icon "a lightning bolt with circuit board traces"
naba icon "a mountain peak" --size 64 --size 128 --size 256 --size 512
naba icon "a rocket ship" --style flat --background white --corners sharp
naba icon "a camera lens" --style skeuomorphic --corners rounded
```
