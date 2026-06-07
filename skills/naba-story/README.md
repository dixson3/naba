# story

A sequential, multi-frame image series from a narrative arc, via the naba CLI.

## Usage

`/naba-story <prompt> [--steps <n>] [--style <s>] [--layout <format>]`

## Prerequisites

The `naba` CLI must be on PATH (declared in `SKILL.md` frontmatter as
`depends-on-tool: [naba]`). See the repository README for installing the naba binary;
when it is absent the skill installs but is inert.

## Install

Deployed by the repo-level `install.{sh,py}`, which auto-discovers every
`skills/*/SKILL.md` via its frontmatter. See the repository README for install scopes
and flags.
