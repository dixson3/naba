---
name: naba-storyboard
description: >
  Generate a story sequence then refine individual frames via the naba CLI. TRIGGER
  when: /naba-storyboard invoked, or the user wants a multi-frame sequence with per-frame
  edits/feedback. SKIP for: a sequence with no per-frame editing (/naba-story); editing a
  single image (/naba-edit).
user-invocable: true
skill-group: naba
depends-on-tool: [naba]
depends-on-skill: [naba-story, naba-edit]
allowed-tools: [Bash, Read]
---
# Storyboard

Generate a story sequence and then refine individual frames using the naba CLI.

## Usage

```
/naba-storyboard <narrative prompt> [--steps <n>] [--style <style>]
```

This is a **composite skill** that runs `naba story` to generate the initial sequence, then uses `naba edit` to refine individual frames based on feedback.

## Workflow

1. **Generate the initial sequence**:
   ```bash
   naba story "<narrative prompt>" --steps <n> --style <style>
   ```

2. **Present all frames**: Show all output file paths in order. Use the Read tool to display each frame.

3. **Collect per-frame feedback**: Ask the user which frames need edits and what changes they want.

4. **Edit individual frames**: For each frame that needs changes:
   ```bash
   naba edit "<frame-file>" "<edit instructions>"
   ```

5. **Present the updated sequence**: Show the final set of frames (original + edited).

6. **Iterate**: Repeat steps 4-6 until the user is satisfied.

## Example Session

```bash
# Step 1: Generate initial story
naba story "a paper airplane's journey from a desk through a window and across a city skyline" --steps 5 --style consistent

# Step 2: Edit frame 3 (needs more dramatic sky)
naba edit naba-story-002.png "make the sky more dramatic with orange and purple sunset colors"

# Step 3: Edit frame 5 (add detail)
naba edit naba-story-004.png "add a child catching the paper airplane in a park"
```

## Prompt Engineering

### story
Write the **narrative arc**, not individual frames. Good: "a seed growing into a towering oak tree through the seasons". Naba splits this into `--steps` frames automatically. Use `--transition` to control how frames relate visually.

### edit
Prompts should describe the **desired change**, not the full image. Be surgical: "remove the background and replace with a sunset sky" or "change the shirt color to blue". The source image provides context.

### Anti-Patterns

- **Avoid negatives**: "no text" or "without watermarks" often backfire. Instead, describe what you want.
- **Avoid resolution specs in prompts**: Use CLI flags (`--size`, `--tile-size`) instead of "4K" or "1024x1024" in the prompt text.
- **Avoid overly long prompts**: 1-3 sentences is the sweet spot. Beyond that, details compete and quality drops.
- **Avoid generic prompts**: "a beautiful landscape" produces generic results. Add specifics: "a misty fjord at dawn with a lone fishing boat".

## Command Flags

### Global Flags

| Flag | Short | Purpose |
|------|-------|---------|
| `--output` | `-o` | Output file path or directory |
| `--json` | | Structured JSON output (auto-enabled when piped) |
| `--quiet` | `-q` | Suppress progress output |
| `--model` | `-m` | Override Gemini model |
| `--preview` | | Open result in system viewer |

### story

| Flag | Default | Values |
|------|---------|--------|
| `--steps` | 4 | 2-8 |
| `--style` | consistent | consistent, evolving |
| `--transition` | smooth | smooth, dramatic, fade |
| `--layout` | separate | separate, grid, comic |

### edit

Positional args: `<file> <prompt>`. No command-specific flags beyond global flags.
