# Plan 03: Embed Rules into Skills and Agents (Option C)

## Objective

Eliminate the preflight hook and symlink system by embedding rule content directly into skill and agent files. Two-tier approach: agents get full rules, skills get only their command-specific content.

## Content Distribution

| File | Prompt Structure | Per-Cmd Guidance | Anti-Patterns | Decision Tree | Global Flags | Cmd Flags |
|------|:---:|:---:|:---:|:---:|:---:|:---:|
| naba_image_assistant | Full | All 7 | Full | Full | Full | All 7 |
| naba_batch_processor | Full | All 7 | Full | Full | Full | All 7 |
| generate/SKILL.md | Full | generate | Full | -- | Full | generate |
| edit/SKILL.md | -- | edit | Full | -- | Full | -- |
| restore/SKILL.md | -- | restore | Full | -- | Full | -- |
| icon/SKILL.md | -- | icon | Full | -- | Full | icon |
| pattern/SKILL.md | -- | pattern | Full | -- | Full | pattern |
| story/SKILL.md | -- | story | Full | -- | Full | story |
| diagram/SKILL.md | -- | diagram | Full | -- | Full | diagram |
| brand_kit/SKILL.md | Full | icon,pattern,generate | Full | -- | Full | icon,pattern,generate |
| storyboard/SKILL.md | -- | story,edit | Full | -- | Full | story,edit |

## Steps

### Step 1: Update 7 command skills

Append to each SKILL.md:
- `## Prompt Engineering` — command-specific guidance paragraph (from naba-image-prompts.md)
- `### Anti-Patterns` — 4 bullet points (universal)
- `## Global Flags` — 5-row table (from naba-tool-routing.md)
- `generate` also gets the full 5-point prompt structure framework
- Replace vague references ("Apply guidance from the naba-image-prompts rule") with "Apply the prompt engineering guidance below"

### Step 2: Update 2 composite skills

- `brand_kit/SKILL.md` — full prompt structure + guidance for icon/pattern/generate + anti-patterns + flags for all 3
- `storyboard/SKILL.md` — guidance for story/edit + anti-patterns + flags for both

### Step 3: Rewrite 2 agent files

- `naba_image_assistant.md` — embed full decision tree, full prompt structure, all 7 per-command guidance blocks, anti-patterns, global flags, all 7 command flag tables
- `naba_batch_processor.md` — same full reference content appended after existing workflow patterns

### Step 4: Modify plugin.json

Remove the `hooks` section entirely.

### Step 5: Delete files

1. `.claude/rules/naba/` symlinks and directory (project-level)
2. `rules/naba-image-prompts.md` and `rules/naba-tool-routing.md`
3. `rules/` directory
4. `.claude-plugin/preflight.json`
5. `scripts/plugin-preflight.sh`
6. `scripts/` directory

### Step 6: Verify

- No references to old rule file names remain
- No symlinks at `.claude/rules/naba/`
- `plugin.json` has no `hooks` key
- Each skill has Anti-Patterns and Global Flags sections
- Each agent has full Decision Tree and all flag tables

## Risks

- **Duplication**: Anti-patterns (4 lines) + global flags (5 rows) repeated across 11 files. Acceptable — content is short and stable.
- **Drift**: Flag changes require updating multiple files. Mitigated by the fact skills already had their own flag tables.
- **Context budget**: Agents grow ~120 lines each. Well within limits, only loaded on invocation.
