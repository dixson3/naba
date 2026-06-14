# /naba edit — modify an existing image

**Tier:** inline. Edit an existing image file with text instructions.

## Usage

```
/naba edit <file> <prompt>
```

## Workflow

1. Verify the input file exists: `ls -la "<file>"`.
2. Craft a **surgical** edit prompt — describe the desired change, not the whole image.
3. Run: `naba edit "<file>" "<edit prompt>"`
4. Present the output path; offer further edits on the result.

## Positional arguments

| Position | Required | Description |
|----------|----------|-------------|
| 1 | Yes | Path to the input image file |
| 2 | Yes | Edit instructions |

## Flags

Only the global flags in SKILL.md (`--output`, `--preview`, …). No command-specific flags.

## Notes

Describe the **desired change**, not the full image. Be surgical ("remove the background and
replace with a sunset sky", "change the shirt color to blue"). The source image supplies the
rest of the context.

## Examples

```bash
naba edit photo.png "remove the background and replace with a gradient sky"
naba edit logo.png "change the primary color from blue to green"
naba edit scene.png "add a flock of birds in the sky"
```
