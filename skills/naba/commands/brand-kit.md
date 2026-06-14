# /naba brand-kit — coordinated brand asset set

**Tier:** composite (subagent). Runs `naba icon`, `naba pattern`, and `naba generate` in
sequence to produce a cohesive brand asset trio (icon + pattern + hero image). Dispatched as
a subagent by the SKILL.md router; this file is the subagent's workflow. For a single asset,
use `/naba icon`, `/naba pattern`, or `/naba generate` directly.

## Usage

```
/naba brand-kit <brand description> [--style <style>]
```

## Workflow

1. Gather brand details: name/concept, color preferences, style direction (modern, minimal,
   bold…), target use cases (app, website, print).
2. Generate the icon:
   `naba icon "<icon prompt>" --style <style> --size 64 --size 128 --size 256 --size 512`
3. Generate the pattern:
   `naba pattern "<pattern prompt>" --colors <scheme> --style <style>`
4. Generate the hero image:
   `naba generate "<hero prompt capturing brand essence>" --style <style>`
5. Present all asset paths grouped by type; return a compact summary to the parent.
6. Offer to re-run individual assets that need refinement.

## Per-command prompt notes

- **icon** — focus on the **symbol/concept**, not framing; `--style`: flat, skeuomorphic,
  minimal, modern.
- **pattern** — describe the **motif and feel**; `--style`/`--colors`/`--density` handle the
  technical attributes.
- **generate (hero)** — descriptive, open-ended; `--style` anchors the visual treatment.

## Example

```bash
naba icon "a minimalist wave crest" --style minimal --size 64 --size 128 --size 256 --size 512 --background transparent
naba pattern "flowing ocean waves and sea foam" --style organic --colors duotone --density medium
naba generate "abstract ocean waves at golden hour, minimal composition with deep blues and warm highlights" --style modern
```
