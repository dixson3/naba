---
type: Reference
okf_spec: OKF-PLAN
---
# Upstream #18: skills lifecycle: add whole-skill garbage collection to install/upgrade

- **Number:** 18
- **Title:** skills lifecycle: add whole-skill garbage collection to install/upgrade
- **URL:** 
- **State:** OPEN
- **Labels:** 

## Body

## Summary

The install/upgrade skill lifecycle prunes **stale files within a still-shipped skill** but has **no whole-skill garbage collection**. If a skill is dropped from a future binary, its previously-deployed directory is never swept from disk. Add whole-skill GC so `naba skills upgrade` (and the post-`self update` refresh) removes skills the binary no longer ships.

## Current behavior

- `prune_stale` (`src/skills.rs:471-486`) deletes on-disk files not in the embedded set — but only *within* a skill dir it is actively deploying.
- `run_one_target` iterates `embed::skill_names()` (`src/skills.rs:234-244`) — i.e. only skills that **still** ship. Nothing enumerates a *previously-deployed* skill that has since been removed from the binary.
- `skills-install.json` (`src/skills_install.rs:28-36`) records `Target { harness, scope, path }` **destinations**, not a per-skill/per-file deployed manifest — so there is no record of *which skills* were written to a target to diff against the current embedded set.

Net: a deprecated skill's directory lingers indefinitely on every target it was installed to.

## Proposed behavior

On `skills upgrade` (and the post-`self update` refresh, which runs an unqualified upgrade over every recorded target — `src/self_cmd/update.rs:344-357`):

1. Determine the set of skills the current binary ships (`embed::skill_names()`).
2. For each recorded target, determine which naba-managed skill dirs are present on disk.
3. Remove skill dirs present on disk but absent from the embedded set (naba-authored only — must not touch user/other-tool skills sharing the same `.../skills` root).

## Design considerations

- **Ownership boundary.** The `.../skills` roots (e.g. `~/.claude/skills`, `.agents/skills`) are shared with other tools and hand-authored skills. GC must only remove skills naba deployed. Two viable signals: (a) the hidden `<!-- naba-skills: v=… tree=… -->` marker already injected into each deployed `SKILL.md` (`src/embed.rs:47,192`) — treat marker presence as proof of naba ownership; or (b) extend `skills-install.json` to record deployed skill *names* per target, giving an explicit manifest to diff.
- Prefer (b) as the authority (explicit, no filesystem heuristics), with (a) as a safety check before `remove_dir_all` so a hand-edited dir that lost its marker isn't nuked.
- Dedup/scope semantics already handled by the existing target resolution (`src/skills.rs:171-185`) should be reused so a codex/agents shared `.agents/skills` dir is GC'd once.
- Add a test parallel to the existing stale-file test (`src/skills.rs:681-698`): install skill A, simulate a binary that no longer ships A, upgrade, assert A's dir is removed and a co-located non-naba skill is untouched.

## Context

Surfaced while documenting naba's skill-lifecycle model for an essay on embedding skills in tools. The lifecycle story is otherwise clean (self-update → `skills upgrade` → tree-hash detection → stale-file prune); whole-skill GC is the one missing edge to make "the tool removes deprecated instructions automatically" fully true.

