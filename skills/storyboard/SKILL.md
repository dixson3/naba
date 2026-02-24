# Storyboard

Generate a story sequence and then refine individual frames using the naba CLI.

## Usage

```
/storyboard <narrative prompt> [--steps <n>] [--style <style>]
```

This is a **composite skill** that runs `naba story` to generate the initial sequence, then uses `naba edit` to refine individual frames based on feedback.

## Workflow

1. **Validate environment**:
   ```bash
   command -v naba || echo "ERROR: naba not found on PATH"
   ```

2. **Generate the initial sequence**:
   ```bash
   naba story "<narrative prompt>" --steps <n> --style <style>
   ```

3. **Present all frames**: Show all output file paths in order. Use the Read tool to display each frame.

4. **Collect per-frame feedback**: Ask the user which frames need edits and what changes they want.

5. **Edit individual frames**: For each frame that needs changes:
   ```bash
   naba edit "<frame-file>" "<edit instructions>"
   ```

6. **Present the updated sequence**: Show the final set of frames (original + edited).

7. **Iterate**: Repeat steps 4-6 until the user is satisfied.

## Example Session

```bash
# Step 1: Generate initial story
naba story "a paper airplane's journey from a desk through a window and across a city skyline" --steps 5 --style consistent

# Step 2: Edit frame 3 (needs more dramatic sky)
naba edit naba-story-002.png "make the sky more dramatic with orange and purple sunset colors"

# Step 3: Edit frame 5 (add detail)
naba edit naba-story-004.png "add a child catching the paper airplane in a park"
```
