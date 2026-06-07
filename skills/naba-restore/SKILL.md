---
name: naba-restore
description: >
  Restore or enhance an existing image via the naba CLI. TRIGGER when: /naba-restore
  invoked, or the user wants to enhance/repair/upscale/denoise/color-correct an
  existing image (prompt optional). SKIP for: directed content changes (/naba-edit),
  creating a new image (/naba-generate).
user-invocable: true
skill-group: naba
depends-on-tool: [naba]
allowed-tools: [Bash, Read]
---
# Restore Image

Restore or enhance an existing image using the naba CLI.

## Usage

```
/naba-restore <file> [prompt]
```

## Workflow

1. **Verify the input file exists**:
   ```bash
   ls -la "<file>"
   ```

2. **Run the command**: The prompt is optional — omit it for general restoration.
   ```bash
   naba restore "<file>" ["<optional refinement prompt>"]
   ```

3. **Present the result**: Show the output file path. Use the Read tool to display the restored image.

4. **Offer iteration**: Ask if the user wants further enhancement.

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

## Prompt Engineering

Minimal prompting — the source image drives the output. Optional prompt refines the enhancement: "increase sharpness", "fix color balance", "remove noise". Omit the prompt for general restoration.

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
