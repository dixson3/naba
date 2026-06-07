---
name: naba-icon
description: >
  Generate app icons via the naba CLI. TRIGGER when: /naba-icon invoked, or the user wants
  an app icon, logo mark, or symbol, optionally at multiple sizes. SKIP for: general
  images (/naba-generate), seamless patterns (/naba-pattern), or full brand asset sets
  (/naba-brand-kit).
user-invocable: true
skill-group: naba
depends-on-tool: [naba]
allowed-tools: [Bash, Read]
---
# Generate Icon

Generate app icons using the naba CLI.

## Usage

```
/naba-icon <prompt> [--style <style>] [--size <px>] [--background <bg>] [--corners <type>]
```

## Workflow

1. **Refine the prompt**: Focus on the **symbol or concept**. Do not describe icon framing or dimensions — naba handles that. Apply the prompt engineering guidance below.

2. **Build and run the command**:
   ```bash
   naba icon "<refined prompt>" [--style <style>] [--size <px>] [--background <bg>] [--corners <type>]
   ```
   Use `--size` multiple times for multiple icon sizes (e.g., `--size 64 --size 128 --size 256`).

3. **Present the result**: Show the output file path(s). Use the Read tool to display the generated icon.

4. **Offer iteration**: Ask if the user wants style or size adjustments.

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

## Prompt Engineering

Prompts should focus on the **symbol or concept**, not composition (naba handles icon framing). Good: "a lightning bolt with circuit traces". Bad: "a 256x256 icon centered on a white background of a lightning bolt". Use `--style` for visual treatment (flat, skeuomorphic, minimal, modern).

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
