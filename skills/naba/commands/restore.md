# /naba restore — restore or enhance an existing image

**Tier:** inline. Restore/enhance/upscale/denoise/color-correct an existing image. The
prompt is optional.

## Usage

```
/naba restore <file> [prompt]
```

## Workflow

1. Verify the input file exists: `ls -la "<file>"`.
2. Run (prompt optional): `naba restore "<file>" ["<optional refinement prompt>"]`
3. Present the output path; offer further enhancement.

## Positional arguments

| Position | Required | Description |
|----------|----------|-------------|
| 1 | Yes | Path to the input image file |
| 2 | No  | Refinement instructions (e.g. "increase sharpness", "fix color balance") |

## Flags

Only the global flags in SKILL.md (`--output`, `--preview`, …). No command-specific flags.

## Notes

Minimal prompting — the source image drives the output. An optional prompt refines the
enhancement ("increase sharpness", "fix color balance", "remove noise"). Omit it for general
restoration.

## Examples

```bash
naba restore old-photo.jpg
naba restore blurry.png "increase sharpness and reduce noise"
naba restore faded.jpg "fix color balance and increase contrast"
```
