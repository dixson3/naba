# Red-Team Review — pass-3

**Plan:** plan-003-james-dixson-94015b
**Date:** 2026-06-14
**Verdict:** REVISE

Third pass, scoped to the v4 new scope (Epics 6/7 + 4.3 installer expansion). Epics 1–4
were verified in pass-1/pass-2 and not re-litigated. **New scope is fundamentally sound**
(go:embed feasibility, install.py port, doctor seams all check out); four new gaps
(C8–C11), all medium/low and mechanical.

## Strengths (verified)

- **go:embed feasible.** `skills/` is at repo root; module `github.com/dixson3/naba`; a root
  `package naba` with `//go:embed skills` is importable by `cmd/naba`. No existing root `.go`,
  no import cycle, no dotfiles under `skills/naba/` (plain `//go:embed skills` works). 6.1
  isolating it first is correct.
- **No build/CI break from embedding or removing install.{sh,py}.** Skill files reference
  `install.{sh,py}` only as doc prose; `.goreleaser.yaml`/`Makefile` build only `cmd/naba`.
- **install.py port well-scoped.** Its group/depends-on/rules machinery is present-but-unused
  for naba (one skill, no rules) — the Go port legitimately drops the dependency engine.
- **doctor seams exist.** `config.ResolveAPIKey()`, `config.Load()`, `cli.Version/Commit/Date`
  all present; `skills status` gives the installed check.

## Concerns

- **C8 [medium] 4.3 retargets the DRIFT-CHECK `installer` node + `e-installer-frontmatter`
  edge but misses §4 referencers, §5 required-sections, and referencer prose that also
  hardcode `install.{sh,py}`.** Stale after supersession: §4 (`installer` "referenced by the
  README Claude Code Skills section"), §5 rows (`Install → install.sh/install.py reference`,
  `Skill install instructions → install.sh actual flags`), plus `skills/naba/README.md:41-43`
  and `AGENTS.md:59` prose. **Rec:** expand 4.3(b) to enumerate the §4/§5 retarget (point at
  `naba skills install`) and add `skills/naba/README.md` + `AGENTS.md` to the 4.1/6.3 rewrite.
- **C9 [medium] README's plan-002 breaking-change migration blockquote (`README.md:152-176`,
  `./install.sh --uninstall` before updating) becomes a dangling command, and `naba skills`
  cannot remove the legacy `/naba-*` skills it never embedded.** The migration path genuinely
  changes, not just the command name. **Rec:** 6.3/4.1 must rewrite that blockquote with a
  documented manual removal path for pre-plan-002 installs (don't silently break it).
- **C10 [medium] doctor "configured model reachable" via `models.list` risks false-green
  (key valid but model absent — the exact bug class this plan fixes) / false-red.** No
  `models.list` call exists today (net-new). **Rec:** 7.1 must specify the check is
  "configured model present in `models.list` results" (intersection, with name normalization
  per E1), and define the absent-model policy (warn vs fail) — not ambiguous "key valid".
- **C11 [low] doctor "skills installed" across scopes/surfaces under-specified** —
  `resolve_dests()` yields user/project × claude/agents + `--target`; doctor calling `skills
  status` once checks one location. **Rec:** state doctor checks the default user/claude (or
  reports per-location).

## Missing

- install.py flag parity in the Go port not enumerated — state what 6.2 carries
  (`--scope`/`--surface`/`--target`/`--dry-run`) vs intentionally drops
  (`--group`/`--list-groups`/`--strict`/dep-closure — single skill, no deps), so SC8 is
  verifiable.
- `rsync --delete` prune-on-upgrade parity: 6.2 "write the embedded tree" should state
  `upgrade` prunes dest files absent from the embed (else orphaned files persist).
- No test/SC asserts `install.{sh,py}` references are gone from docs — covered by 5.3's
  drift-check only if C8's referencer/required-section updates land in 4.3.

## Gate Assessment

Sound, unchanged. Capability Gate covers 5.2(d). Low-priority: add a `models.list` 200 to the
gate test so it proves doctor's path too (cheap, no image cost), not just generation.

## Upstream Assessment

Clean. #3 correctly filed as deferred, out of scope, building on Epic 6. Dependency direction
(#3 → this plan) correct.

## Sequencing note

7.1 `depends-on Epic 1, 6.2` correct (doctor needs the bumped model + `skills status`). 4.3
`depends-on Epic 1, Epic 6` — tighten the installer-node part to **6.3** specifically (the
edge retarget needs install.{sh,py} actually removed). Two-theme split remains clean if wanted.

## Operator Resolutions

| # | Concern | Severity | Resolution | Status |
|:-:|:--------|:---------|:-----------|:-------|
| C8 | 4.3 misses §4/§5/referencer-prose install.{sh,py} retarget | medium | Issue 4.3(b) expanded to enumerate the §4 referencer, both §5 required-section rows (→ `naba skills install`), and `skills/naba/README.md`/`AGENTS.md` prose; 6.3 owns the prose rewrite. | resolved |
| C9 | README plan-002 migration blockquote dangles; legacy /naba-* removal | medium | Issue 6.3 now rewrites `README.md:152-176` with an explicit manual removal command for pre-plan-002 installs (naba skills can't remove legacy /naba-*). SC9a added. | resolved |
| C10 | doctor model-reachable must be models.list intersection + absent policy | medium | Issue 7.1: reachability = `models.list` intersection (with E1 normalization); absent-but-key-valid = **`fail`**. SC9 updated. | resolved |
| C11 | doctor skills-installed scope/surface unspecified | low | Issue 7.1: doctor checks default user/claude unless `--scope`/`--surface` given. | resolved |
| M | install.py flag parity + rsync --delete prune + docs-clean test | — | Issue 6.2 enumerates carried flags (`--scope/--surface/--target/--dry-run`) vs dropped (`--group/--list-groups/--strict`/dep-closure/rules) + `upgrade` prune parity; SC9a + 5.3 assert docs-clean. | resolved |
| seq | tighten 4.3 installer dep to 6.3; optional models.list in gate test | — | 4.3 `depends-on` tightened to `6.3`; Capability Gate adds a `models.list` 200 test (doctor path). | resolved |

**Status:** resolved (all C8–C11 + missing addressed in plan v5; awaiting operator approval)
