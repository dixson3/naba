# Brand Kit

Generate a coordinated set of brand assets (icon + pattern + hero image) using the naba CLI.

## Usage

```
/brand_kit <brand description> [--style <style>]
```

This is a **composite skill** that runs `naba icon`, `naba pattern`, and `naba generate` in sequence to produce a cohesive brand asset set.

## Workflow

1. **Validate environment**:
   ```bash
   command -v naba || echo "ERROR: naba not found on PATH"
   ```

2. **Gather brand details**: Ask the user for:
   - Brand name and concept
   - Color preferences (if any)
   - Style direction (modern, minimal, bold, etc.)
   - Target use cases (app, website, print)

3. **Generate the icon**:
   ```bash
   naba icon "<icon prompt based on brand concept>" --style <style> --size 64 --size 128 --size 256 --size 512
   ```

4. **Generate the pattern**:
   ```bash
   naba pattern "<pattern prompt inspired by brand motifs>" --colors <scheme> --style <style>
   ```

5. **Generate the hero image**:
   ```bash
   naba generate "<hero image prompt capturing brand essence>" --style <style>
   ```

6. **Present all assets**: Show all output file paths grouped by type. Use the Read tool to display each asset.

7. **Offer iteration**: Ask which assets need refinement and re-run individual commands as needed.

## Example Session

```bash
# Step 1: Icon — multiple sizes
naba icon "a minimalist wave crest" --style minimal --size 64 --size 128 --size 256 --size 512 --background transparent

# Step 2: Pattern — matching motif
naba pattern "flowing ocean waves and sea foam" --style organic --colors duotone --density medium

# Step 3: Hero image — brand showcase
naba generate "abstract ocean waves at golden hour, minimal composition with deep blues and warm highlights" --style modern
```
