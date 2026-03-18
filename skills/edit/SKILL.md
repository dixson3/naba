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

## Prompt Engineering

Prompts should describe the **desired change**, not the full image. Be surgical: "remove the background and replace with a sunset sky" or "change the shirt color to blue". The source image provides context.

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
