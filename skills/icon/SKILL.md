# Generate Icon

Generate app icons using the naba CLI.

## Usage

```
/icon <prompt> [--style <style>] [--size <px>] [--background <bg>] [--corners <type>]
```

## Workflow

1. **Validate environment**:
   ```bash
   command -v naba || echo "ERROR: naba not found on PATH"
   ```

2. **Refine the prompt**: Focus on the **symbol or concept**. Do not describe icon framing or dimensions — naba handles that. Apply guidance from the naba-image-prompts rule.

3. **Build and run the command**:
   ```bash
   naba icon "<refined prompt>" [--style <style>] [--size <px>] [--background <bg>] [--corners <type>]
   ```
   Use `--size` multiple times for multiple icon sizes (e.g., `--size 64 --size 128 --size 256`).

4. **Present the result**: Show the output file path(s). Use the Read tool to display the generated icon.

5. **Offer iteration**: Ask if the user wants style or size adjustments.

## Flags

| Flag | Default | Values |
|------|---------|--------|
| `--style` | modern | flat, skeuomorphic, minimal, modern |
| `--size` | 256 | Any integer in px (repeatable) |
| `--format` | png | png, jpeg |
| `--background` | transparent | transparent, white, black, or color name |
| `--corners` | rounded | rounded, sharp |
| `--output` | (auto) | Output file path |
| `--preview` | false | Open result in system viewer |

## Examples

```bash
# Simple icon
naba icon "a lightning bolt with circuit board traces"

# Multiple sizes
naba icon "a mountain peak" --size 64 --size 128 --size 256 --size 512

# Flat style with specific background
naba icon "a rocket ship" --style flat --background white --corners sharp

# Skeuomorphic iOS-style
naba icon "a camera lens" --style skeuomorphic --corners rounded
```
