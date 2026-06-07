# naba-batch

Orchestrate multiple naba CLI calls to produce a coordinated SET of image assets in one
pass — icon suites across concepts, asset pipelines over a list of prompts, or any bulk
sequential generation with organized output. For the fixed icon+pattern+hero trio use the
`naba-brand-kit` skill; for a story sequence with per-frame edits use `naba-storyboard`.

## Usage

`/naba-batch <description of the set> [--style <style>] [--output <dir>]`

## Prerequisites

The `naba` CLI must be on PATH (declared in `SKILL.md` frontmatter as
`depends-on-tool: [naba]`). See the repository README for installing the naba binary;
when it is absent the skill installs but is inert.

## Install

Deployed by the repo-level `install.{sh,py}`, which auto-discovers every
`skills/*/SKILL.md` via its frontmatter. See the repository README for install scopes
and flags.
