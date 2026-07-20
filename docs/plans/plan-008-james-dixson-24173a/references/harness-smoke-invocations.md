# Live harness discovery smoke-test — reproducible invocations (Issue 4.3)

**Tier:** local-only. Gated on `command -v <harness>`; **self-skips** where a harness is
absent. CI does not have these harnesses — the portable path-assertion tests
(`cargo test resolve_dest_harness_paths`, Issue 1.3) are the CI baseline. This tier is an
additional *local* confirmation that each real harness discovers a naba skill installed to
its idiomatic path.

The runnable driver is `tests/harness_smoke.sh` in the repo. It builds naba, and for each
present harness: installs the `naba` skill to that harness's idiomatic **user-scope** path,
asserts the `SKILL.md` landed there, then prints the harness-specific discovery command for
the operator to confirm (the discovery/list step is harness-specific and may make a billable
provider call, so it is operator-run, not automated in the script).

## Per-harness install → idiomatic path (asserted by the script)

| Harness | Install command | Expected user-scope path |
|:--------|:----------------|:-------------------------|
| claude-code | `naba skills install --harness claude-code --scope user` | `~/.claude/skills/naba/SKILL.md` |
| opencode | `naba skills install --harness opencode --scope user` | `~/.config/opencode/skills/naba/SKILL.md` |
| pi | `naba skills install --harness pi --scope user` | `~/.pi/agent/skills/naba/SKILL.md` |
| codex | `naba skills install --harness codex --scope user` | `~/.agents/skills/naba/SKILL.md` |
| agents (portable) | `naba skills install --harness agents --scope user` | `~/.agents/skills/naba/SKILL.md` |

Multi-harness in one shot (dedupes `codex`+`agents` to a single `.agents/skills` write):

```bash
naba skills install --harness claude-code --harness opencode --harness pi --harness codex --scope user
```

## Discovery confirmation (operator-run — may make a billable model call)

The environment these were verified against (see the plan `context.md`): claude-code;
opencode → Bedrock (`AWS_PROFILE`/`AWS_REGION`); pi → OpenRouter (`OPENROUTER_API_KEY`);
codex → OpenRouter via `-c model_provider`. Keep any live call cheap-model + local-only.

| Harness | Discovery confirmation |
|:--------|:-----------------------|
| claude-code | Start Claude Code in a scratch dir; confirm `/naba` (the `naba` skill) is listed/available. |
| opencode | `opencode` (Bedrock creds in env) → list skills / trigger `/naba`; confirm the `naba` skill is discovered from `~/.config/opencode/skills` (also reads `~/.claude`, `~/.agents`). |
| pi | `pi` (OpenRouter) → confirm the `naba` skill under `~/.pi/agent/skills` (also `~/.agents`) is discovered. |
| codex | `codex -c model_provider=<openrouter>` → confirm the `naba` skill under `~/.agents/skills` is discovered (codex's official skills home). |

The install + path-assertion half is automated and cheap (no provider call); only the
discovery confirmation column above is the operator-run, potentially-billable step.
