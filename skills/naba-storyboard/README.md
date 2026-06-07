# storyboard

Composite skill: generate a story sequence, then refine individual frames. Runs naba story, then naba edit on selected frames.

## Usage

`/naba-storyboard <narrative prompt> [--steps <n>] [--style <s>]`

Depends on the `naba-story` and `naba-edit` skills (`depends-on-skill`).

## Prerequisites

The `naba` CLI must be on PATH (declared in `SKILL.md` frontmatter as
`depends-on-tool: [naba]`). See the repository README for installing the naba binary;
when it is absent the skill installs but is inert.

## Install

Deployed by the repo-level `install.{sh,py}`, which auto-discovers every
`skills/*/SKILL.md` via its frontmatter. See the repository README for install scopes
and flags.
