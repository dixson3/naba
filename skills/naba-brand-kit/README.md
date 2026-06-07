# brand_kit

Composite skill: a coordinated brand asset set (icon + pattern + hero image) in one pass. Runs naba icon, pattern, and generate in sequence.

## Usage

`/naba-brand-kit <brand description> [--style <s>]`

Depends on the `naba-icon`, `naba-pattern`, and `naba-generate` skills (`depends-on-skill`).

## Prerequisites

The `naba` CLI must be on PATH (declared in `SKILL.md` frontmatter as
`depends-on-tool: [naba]`). See the repository README for installing the naba binary;
when it is absent the skill installs but is inert.

## Install

Deployed by the repo-level `install.{sh,py}`, which auto-discovers every
`skills/*/SKILL.md` via its frontmatter. See the repository README for install scopes
and flags.
