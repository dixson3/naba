# Edit Image

Edit an existing image with text instructions using the naba CLI.

## Usage

```
/edit <file> <prompt>
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

3. **Craft the edit prompt**: Describe the **desired change**, not the full image. Be specific and surgical.

4. **Run the command**:
   ```bash
   naba edit "<file>" "<edit prompt>"
   ```

5. **Present the result**: Show the output file path. Use the Read tool to display the edited image.

6. **Offer iteration**: Ask if the user wants further edits on the result.

## Flags

| Flag | Default | Description |
|------|---------|-------------|
| `--output` | (auto) | Output file path |
| `--preview` | false | Open result in system viewer |

## Positional Arguments

| Position | Required | Description |
|----------|----------|-------------|
| 1 | Yes | Path to the input image file |
| 2 | Yes | Edit instructions |

## Examples

```bash
# Remove background
naba edit photo.png "remove the background and replace with a gradient sky"

# Change colors
naba edit logo.png "change the primary color from blue to green"

# Add elements
naba edit scene.png "add a flock of birds in the sky"
```
