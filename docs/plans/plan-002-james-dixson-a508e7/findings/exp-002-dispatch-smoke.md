# Finding: Install + dispatch smoke test (Issue 3.1)

End-to-end validation of the consolidated skill on a throwaway `--target` install. Resolves
red-team C5 (composite runs a real `naba` call, not review-only) and N1 (child-side
file-writing grant). Source: live runs (`naba` on PATH, `GEMINI_API_KEY` set).

## Install

`./install.sh --target <tmp>/skills naba` landed `skills/naba/` with `SKILL.md`,
`README.md`, and all 10 `commands/<sub>.md` (the `commands/` subdir travels with the dir
copy). The absolute `commands/generate.md` path resolved under the target — the mechanism
the router uses (`${CLAUDE_SKILL_DIR}/commands/<sub>.md`) is sound.

## Inline tier (generate)

One real `naba generate "… red apple …" -o <tmp>/out/inline-apple.png` produced a 1.0 MB
PNG. The single-call inline path works end-to-end.

## Composite tier (batch) — subagent dispatch

A dispatched `general-purpose` subagent, handed the **absolute** installed
`commands/batch.md` path, ran the asset-pipeline pattern: two sequential
`naba generate … -o <tmp>/out/batch-0N.png` calls produced 729 KB and 787 KB PNGs **in the
child context**. This proves: subagent spawn (`Agent`), child-side `Bash`, and `naba` writing
files all work end-to-end through a composite dispatch. (The first run died on a transient API
socket error *after* generating both images — an infra hiccup, not a permission gap.)

## N1 refinement (discovered)

A follow-up subagent confirmed the child has a working **`Write`** grant (wrote
`manifest.txt`) but **no `Glob`** grant in this harness — it fell back to shell `ls`. The
composite does **not** depend on `Glob`: `batch.md` writes each item to an explicit
`-o "<dir>/<name>.png"`, so paths are known without globbing. Updated `SKILL.md` (Composite
dispatch) and `IG/skills.md` §5 to drop the over-promising "child covers Bash/Write/Glob"
claim — the composite needs only child `Bash` (+ optional `Write`), and lists with `Bash`.

## Verdict

Dispatch contract validated end-to-end. Inline and composite tiers both execute real `naba`
calls and write files. Parent `allowed-tools: [Bash, Read, Agent]` is sufficient.
