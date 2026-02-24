# Restore Image

Restore or enhance an existing image using the naba CLI.

## Usage

```
/restore <file> [prompt]
```

## Workflow

1. **Validate environment**:
   ```bash
   command -v naba || echo "ERROR: naba not found on PATH"
   ```

2. **Verify the input file exists**:
   ```bash
   ls -la "<file>"
   ```

3. **Run the command**: The prompt is optional — omit it for general restoration.
   ```bash
   naba restore "<file>" ["<optional refinement prompt>"]
   ```

4. **Present the result**: Show the output file path. Use the Read tool to display the restored image.

5. **Offer iteration**: Ask if the user wants further enhancement.

## Flags

| Flag | Default | Description |
|------|---------|-------------|
| `--output` | (auto) | Output file path |
| `--preview` | false | Open result in system viewer |

## Positional Arguments

| Position | Required | Description |
|----------|----------|-------------|
| 1 | Yes | Path to the input image file |
| 2 | No | Refinement instructions (e.g., "increase sharpness", "fix color balance") |

## Examples

```bash
# General restoration
naba restore old-photo.jpg

# Targeted enhancement
naba restore blurry.png "increase sharpness and reduce noise"

# Color correction
naba restore faded.jpg "fix color balance and increase contrast"
```
