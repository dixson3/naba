# /naba storyboard — story sequence + per-frame refinement

**Tier:** composite (subagent). Runs `naba story` to generate a sequence, then `naba edit`
to refine individual frames from feedback. Dispatched as a subagent by the SKILL.md router;
this file is the subagent's workflow.

## Usage

```
/naba storyboard <narrative prompt> [--steps <n>] [--style <style>]
```

## Workflow

1. Generate the initial sequence:
   `naba story "<narrative prompt>" --steps <n> --style <style>`
2. Present all frame paths in order.
3. Collect per-frame feedback — which frames need edits, and what changes.
4. Edit each flagged frame: `naba edit "<frame-file>" "<edit instructions>"`
5. Present the updated sequence (original + edited frames).
6. Repeat 3–5 until satisfied. Return a compact summary (final frame paths) to the parent.

## Prompt notes

- **story step** — write the **narrative arc**, not individual frames; naba splits it into
  `--steps` frames. Use `--transition` to control how frames relate.
- **edit step** — describe the **desired change** surgically; the source frame supplies the
  rest of the context.

## Flags

`story`: `--steps` (default 4, 2–8), `--style` (consistent | evolving), `--transition`
(smooth | dramatic | fade), `--layout` (separate | grid | comic). `edit`: positional
`<file> <prompt>`, global flags only. (Plus the global flags in SKILL.md.)

## Example

```bash
naba story "a paper airplane's journey from a desk through a window and across a city skyline" --steps 5 --style consistent
naba edit naba-story-002.png "make the sky more dramatic with orange and purple sunset colors"
naba edit naba-story-004.png "add a child catching the paper airplane in a park"
```
