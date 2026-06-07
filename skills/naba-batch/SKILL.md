---
name: naba-batch
description: >
  Orchestrate multiple naba CLI calls to produce a coordinated SET of image assets in
  one pass. TRIGGER when: /naba-batch invoked, or the user wants several related images
  at once — an icon suite across multiple concepts, an asset pipeline over a list of
  prompts, or any bulk sequential generation with organized output. SKIP for: a single
  image (/naba-generate); the fixed brand-asset trio (/naba-brand-kit); a story sequence
  with per-frame edits (/naba-storyboard).
user-invocable: true
skill-group: naba
depends-on-tool: [naba]
allowed-tools: [Bash, Read, Glob, Write]
---

# naba Batch Processor

Run multiple `naba` commands in sequence to deliver a coordinated set of image assets:
icon suites, asset pipelines, and other multi-image workflows. For the fixed
icon+pattern+hero trio use `/naba-brand-kit`; for a story sequence with per-frame edits
use `/naba-storyboard`.

## Usage

```
/naba-batch <description of the set> [--style <style>] [--output <dir>]
```

## Workflow Patterns

### Icon suite (multiple concepts)
For each icon concept in the set:
```bash
naba icon "<concept prompt>" --style <style> --size 64 --size 128 --size 256 -o "<dir>/<name>.png"
```

### Asset pipeline (batch generation over a list)
For each asset in the list:
```bash
naba generate "<prompt>" --style <style> -o "<dir>/<name>.png"
```

### Mixed set
Sequence any combination of `naba generate|icon|pattern|story|diagram|edit|restore`,
keeping a consistent `--style` and a shared output directory for a cohesive set.

## Execution Guidelines

- Run commands **sequentially**, not in parallel, to avoid API rate limits.
- Use `--json` to capture structured output for tracking; use `-o`/`--output` to organize
  files into a meaningful directory.
- Report progress after each completed item; on failure, log the error and continue with
  the remaining items.
- When complete, present a summary of all generated assets (optionally write a manifest
  file listing paths).

## Prompt Engineering

Build each prompt as **subject + composition + style + lighting + details**, and keep
`--style` consistent across the set so the assets read as a coordinated group. See the
individual command skills (`/naba-generate`, `/naba-icon`, etc.) for per-command guidance.

### Anti-Patterns

- **Avoid negatives** ("no text"); describe what you want instead.
- **Avoid resolution specs in prompts**; use CLI flags (`--size`, `--tile-size`).
- **Keep prompts to 1-3 sentences**; details compete beyond that.

## Global Flags

| Flag | Short | Purpose |
|------|-------|---------|
| `--output` | `-o` | Output file path or directory |
| `--json` | | Structured JSON output (auto-enabled when piped) |
| `--quiet` | `-q` | Suppress progress output |
| `--model` | `-m` | Override Gemini model |
| `--preview` | | Open result in system viewer |
