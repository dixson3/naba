# icon

App icons from a concept, optionally at multiple sizes, via the naba CLI.

## Usage

`/naba-icon <prompt> [--style <s>] [--size <px>] [--background <bg>]`

## Prerequisites

The `naba` CLI must be on PATH (declared in `SKILL.md` frontmatter as
`depends-on-tool: [naba]`). See the repository README for installing the naba binary;
when it is absent the skill installs but is inert.

## Install

Deployed by the repo-level `install.{sh,py}`, which auto-discovers every
`skills/*/SKILL.md` via its frontmatter. See the repository README for install scopes
and flags.
