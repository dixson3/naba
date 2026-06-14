# Finding: Dispatch contract spike (Issue 0.1)

Verifies the `/naba <subcommand>` invocation model is real before authoring 10 command
files. Resolves red-team concerns C1, C2, C6 and N1. Source: authoritative Claude Code
skill/subagent docs (via `claude-code-guide`) + direct read of `internal/cli/story.go`.

## Q1 — ARGUMENTS contract (C1)

**Confirmed.** A user-invocable skill receives trailing tokens via the `$ARGUMENTS`
placeholder; if `$ARGUMENTS` is absent from the body they are appended as
`ARGUMENTS: <value>`. Individual tokens are addressable as `$ARGUMENTS[N]` / `$N`. The
markdown-body router splits `$ARGUMENTS` on whitespace and branches on the first token as
the subcommand. This is the same pattern `/bdplan` uses for `init`/`continue`/`execute`.
→ No design pivot. The router model is valid.

## Q2 — Self base-dir resolution (C2)

**Confirmed, with a better mechanism than the bash `find` the plan assumed.** The canonical
way is the `${CLAUDE_SKILL_DIR}` environment variable — "the directory containing the
skill's `SKILL.md`", valid across every deployment scope (`~/.claude/skills/`,
project `.claude/skills/`, plugin skills, `--target`/`--add-dir` installs). The router
hands a spawned subagent an **absolute** path `${CLAUDE_SKILL_DIR}/commands/<sub>.md`.
→ Resolves C2 in favor of **option (a)** (absolute path), via `${CLAUDE_SKILL_DIR}`
(no fragile `find` needed).

## Q3 — `naba story` is a single CLI invocation (C6)

**Confirmed.** `internal/cli/story.go` defines one cobra command (`Use: "story <prompt>"`,
`RunE: runStory`). `runStory` loops internally (`for step := 1; step <= storySteps`) to
emit N frames, but the skill issues **one** `naba story` call. → `story` belongs in the
**inline** tier, not the subagent tier.

## Q4 — Composite tool grants (N1)

**Confirmed.** Subagent tool permissions are independent of the parent skill. A spawned
subagent's grant comes from the agent definition (e.g. `general-purpose` has all tools),
not from the parent skill's `allowed-tools`. The parent therefore needs only
`[Bash, Read, Agent]` — `Bash`/`Read` for the inline tier and reading `commands/<sub>.md`,
`Agent` to spawn composites. The composite's `Bash`/`Write`/`Glob` file-writing grant
lives in the **child**. → Parent frontmatter does **not** need `Write`/`Glob`.

## Q-bonus — Spawn tool name (Risks table)

The subagent-spawn tool is **`Agent`** in shipped Claude Code (renamed from `Task` in
2.1.63; `Task` still works as a legacy alias). Use `Agent` in `allowed-tools`.

## Design decisions carried into Issue 1.1

- Router parses `$ARGUMENTS`; first token = subcommand; `help`/unknown/empty → print the
  subcommand list.
- Composite dispatch passes `${CLAUDE_SKILL_DIR}/commands/<sub>.md` (absolute) to the
  subagent — **option (a)**.
- Parent `allowed-tools: [Bash, Read, Agent]` (refines the plan's
  `[Bash, Read, Glob, Write, Task]`: drop `Write`/`Glob` — they live in the composite
  child; `Task` → `Agent`).
- Inline tier: generate, edit, restore, icon, pattern, diagram, **story**.
  Composite tier: storyboard, batch, brand-kit.
