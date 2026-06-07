# edit

Surgical edits to an existing image via text instructions, using the naba CLI.

## Usage

`/naba-edit <file> <prompt>`

## Prerequisites

The `naba` CLI must be on PATH (declared in `SKILL.md` frontmatter as
`depends-on-tool: [naba]`). See the repository README for installing the naba binary;
when it is absent the skill installs but is inert.

## Install

Deployed by the repo-level `install.{sh,py}`, which auto-discovers every
`skills/*/SKILL.md` via its frontmatter. See the repository README for install scopes
and flags.
