# Red-Team Review — pass-4

**Plan:** plan-003-james-dixson-94015b
**Date:** 2026-06-14
**Verdict:** APPROVE

Fourth pass, scoped to the v6 delta: the skill integrity-marker mechanism (Scope #10) and its
integration into Epics 6.1/6.2, 7.1, 5.1, 4.2. No high/medium concerns; two low precision
notes (folded in). No conformance regression.

## Strengths (verified against real files)

- **Determinism correct.** `tree=<sha256>` digests the embedded FS bytes (content-derived, no
  timestamp) and is the integrity signal; `v=<naba-version>` (from `cli.Version`, ldflags) is
  informational and per-binary stable. Status keys on `tree`, not `v` — correct.
- **No marker collision.** `grep "<!--"` over `skills/naba/` is empty; no file contains a
  `naba-skills:`/`tree=` line. The token is unique in the corpus.
- **Frontmatter placement safe.** SKILL.md is `---`…`---` then `# naba`; injecting after the
  closing `---` doesn't touch the YAML, so the skill loader is unaffected.
- **Deployed copy escapes linters.** markdown-lint/drift-check fire on the repo source (manifest
  §6 glob = `skills/naba/SKILL.md`); the deployed copy is outside any glob. Repo source stays
  marker-free, so the marker never reaches a linter.
- **Whole-tree tamper detection works.** 6.2 specifies three checks (up-to-date / complete /
  unmodified); a deleted/edited `commands/*.md` with untouched SKILL.md is caught by
  complete+unmodified (recomputed hash covers the whole tree), not the marker read alone. 5.1's
  tamper case asserts this.
- **Dep graph acyclic.** 4.2 → {2,3,6,7}; 6.1→6.2→6.3; 7.1 → {1,6.2}; 5.1 → {1,2,6,7};
  4.3 → {1,6.3}. No cycle, no dangling ref.

## Concerns

- **C12 [low] Canonical-hash strip/rehash underspecified.** "Excluding the marker line" should
  be pinned: strip exactly the single anchored `^<!-- naba-skills: .* -->$` line + its
  terminator from the deployed SKILL.md (first match, scoped to SKILL.md only, not tree-wide),
  restoring the embedded original byte-for-byte; raw-byte hashing, no line-ending/trailing-
  newline normalization (files end in a single `\n` today). The marker is emitted as exactly
  one line. **Rec:** one sentence in 6.1 pinning the digest + strip definition.
- **C13 [low] Upgrade re-inject ordering unstated.** On `upgrade` the dest SKILL.md carries an
  old marker; the fresh write must rewrite SKILL.md from the (marker-free) embed and then
  inject, or strip-any-existing-then-inject (idempotent) — never append on top of an old
  marker (double-mark). **Rec:** one sentence in 6.2.

## Missing

Nothing blocking. Test coverage is complete (5.1 round-trip + tamper; 5.2(d) live install +
doctor green; SC8/SC9 marker behavior + marker-free repo source). Optional: an assertion that
`v` mismatch alone does not flip status to outdated (only `tree` does) — low value, plan text
already says status keys on `tree`.

## Gate Assessment

Unchanged, sound. Marker mechanism adds no network call/dependency → no new gate. 5.1's
round-trip is a pure stdlib test. Capability Gate's `models.list` test still covers 5.2(d).

## Upstream Assessment

Clean, unchanged. #3 deferred/out of scope. Marker is internal to `naba skills`; no upstream
surface.

## Operator Resolutions

| # | Concern | Severity | Resolution | Status |
|:-:|:--------|:---------|:-----------|:-------|
| C12 | canonical-hash strip/rehash definition | low | One sentence added to Issue 6.1 pinning the digest + anchored single-line strip + raw-byte hashing. | resolved |
| C13 | upgrade write-from-embed-then-inject ordering | low | One sentence added to Issue 6.2: upgrade rewrites SKILL.md from the embed then injects; injection is idempotent (strip-existing-then-inject). | resolved |

**Status:** APPROVE — two low precision notes folded into plan v7. Plan ready for INTAKE.
