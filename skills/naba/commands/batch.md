# /naba batch — coordinated set of image assets

**Tier:** composite (subagent). Runs multiple `naba` calls in sequence to deliver a
coordinated set: icon suites, asset pipelines, other multi-image workflows. Dispatched as a
subagent by the SKILL.md router; this file is the subagent's workflow. For the fixed
icon+pattern+hero trio use `/naba brand-kit`; for a story sequence with per-frame edits use
`/naba storyboard`.

## Usage

```
/naba batch <description of the set> [--style <style>] [--output <dir>]
```

## Workflow patterns

- **Icon suite** (multiple concepts) — for each concept:
  `naba icon "<concept prompt>" --style <style> --size 64 --size 128 --size 256 -o "<dir>/<name>.png"`
- **Asset pipeline** (list of prompts) — for each asset:
  `naba generate "<prompt>" --style <style> -o "<dir>/<name>.png"`
- **Mixed set** — any combination of `naba generate|icon|pattern|story|diagram|edit|restore`,
  keeping a consistent `--style` and a shared output directory.

## Execution guidelines

- Run commands **sequentially**, not in parallel, to avoid API rate limits.
- Use `--json` to capture structured output for tracking; use `-o`/`--output` to organize
  files into a meaningful directory (always write each item to its own `-o "<dir>/<name>.png"`).
- Report progress after each item; on failure, log the error and continue with the rest.
- On completion, return a compact summary (all generated asset paths), optionally writing a
  manifest file listing them.

## Prompt notes

Keep `--style` consistent across the set so assets read as a coordinated group. See the
individual `commands/<sub>.md` files for per-command prompt nuance.
