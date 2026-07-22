---
type: Plan
okf_spec: OKF-PLAN
id: plan-012-james-dixson-a3cd13
author: james-dixson
created: '2026-07-21'
status: reconciling
deliverable_class: standard
fingerprint: cd94cb77e969bf08964baf77a2cbd31bf94530c62386441cac52856aac687d02
epic: naba-mol-c30
---
# Plan: Address issues #16 (VOICE.md doc sweep), #17 (curated CHANGELOG.md), #18 (whole-skill garbage collection)

**ID:** plan-012-james-dixson-a3cd13
**Author:** james-dixson
**Created:** 2026-07-21
**Status:** reconciling
**Deliverable-class:** standard
**Epic:** naba-mol-c30
**Fingerprint:** cd94cb77e969bf08964baf77a2cbd31bf94530c62386441cac52856aac687d02
**Phase log:**
- 2026-07-21 scoping: three independent upstream issues bundled into one plan; all three `include`
- 2026-07-21 investigating: #18 code paths read (skills.rs, skills_install.rs, embed.rs, self_cmd/update.rs); #16/#17 doc targets enumerated
- 2026-07-21 drafting: plan v1 presented
- 2026-07-21 drafting: pass-1 REVISE resolved — Epic 3 GC ordering/plumbing/report pinned, Epic 3 test gate added, 2.3 verification sharpened

## Objective
Close three open GitHub issues that share nothing but the repo:

- **#16** — apply the `VOICE.md` writing-voice rules to the remaining user-facing docs (the sweep
  started on `README.md` and the web pages that had violations; finish the rest).
- **#17** — add a hand-curated `CHANGELOG.md` so `cargo-dist` generates readable release notes
  instead of raw commit summaries, and fold it into the release lockstep.
- **#18** — add whole-skill garbage collection to `naba skills upgrade` so a skill dropped from a
  future binary is swept from disk instead of lingering forever.

## Motivation
Three loose ends were filed as upstream issues after the v0.7.0 release and the VOICE.md work:

- **#16:** `VOICE.md` was authored and applied only where violations were already visible
  (`README.md` + a few web pages). The remaining reader-facing prose (`CONTRIBUTING.md`, the full
  `web/content/pages/*` and `web/content/cards/*` set) was never swept, so the voice is applied
  inconsistently — a reader hits polished prose on one page and terse reference on the next.
- **#17:** `cargo-dist` currently derives each GitHub Release body from commit summaries because no
  `CHANGELOG.md` is present. Release notes are therefore an unedited commit dump rather than an
  intentional, readable summary. Whoever reads a release to decide whether to upgrade is affected.
- **#18:** naba's skill-lifecycle story (self-update → `skills upgrade` → tree-hash detect →
  stale-file prune) is clean *except* for one edge: `prune_stale` only deletes stale files *within*
  a still-shipped skill dir, and `run_one_target` only iterates skills the binary *still* ships.
  Nothing enumerates a *previously-deployed* skill that has since been removed from the binary, so a
  deprecated skill's directory lingers indefinitely on every target it was installed to. Surfaced
  while documenting the lifecycle model for an essay — the one missing edge to make "the tool removes
  deprecated instructions automatically" fully true.

## Upstream Issues
| Issue | Title | Disposition | Notes | Resolved By |
|:------|:------|:------------|:------|:------------|
| [#16](https://github.com/dixson3/naba/issues/16) | Apply VOICE.md across remaining user-facing docs | include | Sweep CONTRIBUTING.md + web/content prose | Epic 1 |
| [#17](https://github.com/dixson3/naba/issues/17) | Add a curated CHANGELOG.md for release notes | include | Keep a Changelog; backfill v0.6.x + v0.7.0 | Epic 2 |
| [#18](https://github.com/dixson3/naba/issues/18) | skills lifecycle: whole-skill garbage collection | include | Manifest authority + marker safety check | Epic 3 |

## Investigation Findings

### #18 — current skill lifecycle code (read 2026-07-21)
- `prune_stale` (`src/skills.rs:471`) deletes on-disk files not in `embed::skill_files(name)` — but
  only *within* a skill dir it is actively deploying. No whole-dir removal.
- `run_one_target` (`src/skills.rs:234`) loops `embed::skill_names()` — only skills that **still**
  ship. Nothing enumerates a previously-deployed, now-removed skill.
- `skills_install::Target` (`src/skills_install.rs:29`) records `{ harness, scope, path }`
  **destinations** only — no per-skill deployed manifest to diff against the current embedded set.
  `Registry` has `upsert`/`remove`/`load_or_migrate`/`synthesize_from_legacy`; atomic save; serde
  `#[serde(default)]` tolerance for schema growth.
- Ownership signal (a): the `<!-- naba-skills: v=… tree=… -->` marker (`src/embed.rs:47`,
  `MARKER_PREFIX`) is injected into every deployed `SKILL.md` — proof-of-naba-ownership.
- The post-`self update` refresh runs an unqualified `skills upgrade` over every recorded target
  (`src/self_cmd/update.rs:344`), so GC wired into `upgrade` covers the self-update path for free.
- Existing stale-file test lives at `src/skills.rs:681` — the new GC test parallels it.

### #16 — doc targets enumerated
- `VOICE.md` scope: `README.md`, `CONTRIBUTING.md`, `web/content/**`. Three rules: (1) verbose,
  human-friendly exposition; (2) precedence as explicit ordered lists, never `A > B > C`; (3) name
  the tool as `naba` (never bare) in prose.
- Not-yet-swept targets: `CONTRIBUTING.md`, `web/content/pages/{usage,mcp,install,config,skills}.md`,
  `web/content/cards/*.md`, `web/content/home/hero.md`.

### #17 — release integration
- Releases are cut by `cargo-dist` (`AGENTS.md:142`); the "Releasing (lockstep rule)" at
  `AGENTS.md:151` is the ordered release checklist the CHANGELOG step folds into.

## Approach

Three **independent** epics, no cross-dependencies (operator confirmed: any order). Each lands and
validates on its own.

- **#16 (docs):** prose-only sweep. Apply the three VOICE rules file-by-file, lint each with the
  `yf-markdown-lint` authoring subset, rebuild the web site to confirm it still generates.
- **#17 (CHANGELOG):** add `CHANGELOG.md` in Keep a Changelog format, backfill v0.6.x + v0.7.0
  (operator-confirmed depth), verify cargo-dist reads the matching version section, and add the
  CHANGELOG-update step to the AGENTS.md release lockstep.
- **#18 (GC):** operator-confirmed **manifest-authority + marker-safety** design. The `record_target`
  write fires *after* deploy in `run()` (`src/skills.rs:277`), so the GC diff must read the
  **previously-recorded** skill set **before** that write clobbers it. Extend `skills_install::Target`
  with a `skills: Vec<String>` field (deployed skill names), `#[serde(default)]` for back-compat with
  v1 rows that lack it (empty vec ⇒ "GC nothing" — the first post-update upgrade over a legacy/empty
  row is a no-op that merely repopulates `skills`; only a *later* binary that drops a skill triggers a
  removal). Per-target GC ordering, explicit to avoid the empty-diff trap:
  1. **Load** the target row's previously-recorded `skills` (before any write).
  2. **Deploy** the current embedded set (`run_one_target`).
  3. **Compute** the GC set = recorded − `embed::skill_names()`.
  4. **Remove** each GC-set dir on that target — but only after the **safety gate**: the dir's
     `SKILL.md` must carry the naba marker (`MARKER_PREFIX`, `src/embed.rs:47`). A dir that lost its
     marker is *reported, not removed*. `remove_dir_all` is irreversible; the marker veto + empty
     default are the complete safety story.
  5. **Upsert** the new embedded set back to **every** registry row that dedups to the removed path
     (mirror the read-union with a write fan-out — otherwise a co-resolving row keeps the phantom
     name and reports it `skipped_no_marker` on every future upgrade), dropping a GC'd name **only
     after** its removal succeeded — a partial `remove_dir_all` failure leaves the name recorded so
     the next upgrade retries rather than silently orphaning.
  6. **Plumbing:** `targets_for` (`src/skills.rs:216`) currently reduces the registry to
     `(harness, PathBuf)` and drops the `skills` field; the recorded skills must be surfaced per
     resolved path (union the sets when two rows dedup to one path). Preferred fix: also surface each
     row's **own** scope through `targets_for` and pass it to `record_target`, keeping the single
     existing `(harness, scope, path)` upsert key. (Keying on `(harness, path)` for the upgrade case
     is a fallback, but would need a new upsert variant since the current key includes scope.)
  7. **Report:** a GC'd skill is absent from `embed::skill_names()`, so it can never surface via the
     existing `SkillActionItem` loop — add a per-target `gc: Vec<{name, path, outcome}>` collection
     (`outcome ∈ removed | skipped_no_marker`) to the report so `--dry-run` and `--json` can render
     removals. Reuse the existing dedup/scope resolution so a shared `.agents/skills` root is GC'd once.

## Epics

### Epic 1: #16 — VOICE.md doc sweep
- Issue 1.1: Sweep `CONTRIBUTING.md` for the three VOICE rules.
- Issue 1.2: Full pass over `web/content/pages/*.md` (usage, mcp, install, config, skills, 404).
- Issue 1.3: Pass over `web/content/cards/*.md` and `web/content/home/hero.md`.
- Issue 1.4: Lint every changed `.md` (yf-markdown-lint authoring subset) and rebuild the web site
  to confirm it generates.
  - depends-on: 1.1, 1.2, 1.3
  - resolves-upstream: #16 (include)

### Epic 2: #17 — curated CHANGELOG.md
- Issue 2.1: Create `CHANGELOG.md` (Keep a Changelog format) with an `[Unreleased]` section.
- Issue 2.2: Backfill notable entries for v0.6.x and v0.7.0 from git history.
  - depends-on: 2.1
- Issue 2.3: Verify cargo-dist reads the matching version section for the release body. Concrete
  check: `parse-changelog CHANGELOG.md 0.7.0` extracts the expected body, and the heading format
  (`## [0.7.0]`) matches the `v0.7.0` release tag (cargo-dist matches tag→heading at announce time —
  the `v` prefix / date suffix is where it silently fails). A dry-run announce inspection is the
  belt-and-suspenders check where available.
  - depends-on: 2.2
- Issue 2.4: Fold the CHANGELOG-update step into the AGENTS.md "Releasing (lockstep rule)".
  - depends-on: 2.1
  - resolves-upstream: #17 (include)

### Epic 3: #18 — whole-skill garbage collection
- Issue 3.1: Add `skills: Vec<String>` to `skills_install::Target` (`#[serde(default)]`), thread it
  through `Target::new`/upsert; update round-trip + migration tests.
- Issue 3.2: Populate deployed skill names in `record_target` on install + upgrade. Preferred:
  surface each registry row's **own** scope through `targets_for` and pass it to `record_target`
  (keeps the single `(harness, scope, path)` upsert key) so it updates the existing row rather than
  writing a mis-scoped duplicate that would carry the wrong `skills` list. Keying on `(harness, path)`
  is a fallback that needs a new upsert variant.
  - depends-on: 3.1
- Issue 3.3: Add a marker-check helper (dir's `SKILL.md` carries `MARKER_PREFIX`) as the
  remove-safety gate.
- Issue 3.4: Implement the GC pass on `upgrade` per the Approach ordering: (a) surface each target
  row's previously-recorded `skills` through `targets_for` (union on shared-path dedup), (b) deploy,
  (c) GC set = recorded − embedded, (d) `remove_dir_all` marker-owned dirs only, (e) upsert the new
  set back to **all** rows that dedup to the removed path, dropping only successfully-removed names.
  Add the per-target `gc: [{name, path, outcome}]` report
  field; honor `--dry-run` (report `would-remove`) + `--json`; dedup shared roots.
  - depends-on: 3.1, 3.2, 3.3
- Issue 3.5: Test parallel to the stale-file test: install skill A, simulate a binary that no longer
  ships A, upgrade, assert A's dir removed and a co-located non-naba skill untouched.
  - depends-on: 3.4
  - resolves-upstream: #18 (include)

## Gates
### Start Gate (mandatory)
- Type: human
- Approvers: operator

### Capability Gate: web site builds (Epic 1)
- Type: auto
- Condition: the Pelican web site generator runs clean after the doc sweep
- Test: `cd web && pelican content -s pelicanconf.py` exits 0 (adjust to the documented build in
  `web/` if it differs)
- Blocks: Issue 1.4
- Instructions: run the Pelican build named above; resolve any generation error before closing 1.4

### Capability Gate: Epic 3 builds & tests
- Type: auto
- Condition: the Rust crate compiles and the test suite (incl. the new GC test) is green — the only
  epic that ships code and performs an irreversible `remove_dir_all`
- Test: `cargo build && cargo test` exits 0
- Blocks: Issue 3.5 (and thereby reconcile)
- Instructions: run `cargo build && cargo test`; resolve any failure before closing 3.5

### Reconcile Gate (upstream issues incorporated)
- Type: auto (all execution beads closed)
- Blocks: reconcile step

## Risks & Mitigations
- **GC nukes a non-naba skill** (#18). Mitigation: the marker-presence safety check is a hard gate
  before `remove_dir_all`; the explicit manifest is the authority, the marker is the veto. Test 3.5
  asserts a co-located non-naba skill is untouched.
- **Irreversible deletion runs unattended** (#18). GC's `remove_dir_all` executes inside the
  post-`self update` refresh (`src/self_cmd/update.rs:344`) with no operator present. This is
  *intended* destructive-on-upgrade behavior; the marker veto + `#[serde(default)]` empty-vec
  (unknown ⇒ GC nothing) are the complete, accepted safety story. No undo — a mis-marked dir is
  reported and skipped rather than risked.
- **Empty-diff trap from write-before-read** (#18). If `record_target` overwrote the row before the
  GC diff read it, the diff would always be empty and nothing would ever be collected. Mitigation:
  the Approach pins the (a)–(e) ordering — read recorded skills first, upsert last.
- **v1 registry rows lack the `skills` field** (#18). Mitigation: `#[serde(default)]` → empty vec;
  an empty recorded set means "nothing known to GC", never a spurious removal. First upgrade
  repopulates the field.
- **VOICE sweep introduces GFM breakage** (#16). Mitigation: lint each changed file + rebuild the
  site (Issue 1.4 gate) before landing.
- **cargo-dist doesn't pick up the CHANGELOG section** (#17). Mitigation: Issue 2.3 verifies the
  section-matching behavior before the release lockstep depends on it.

## Success Criteria
- **#16:** `CONTRIBUTING.md` and all `web/content/**` prose conform to the three VOICE rules; every
  changed `.md` passes the lint subset; the web site builds clean.
- **#17:** `CHANGELOG.md` exists in Keep a Changelog format with v0.6.x + v0.7.0 backfilled and an
  `[Unreleased]` section; cargo-dist emits the matching section as the release body; the AGENTS.md
  release lockstep includes the CHANGELOG step.
- **#18:** `naba skills upgrade` removes a skill dir dropped from the binary on every recorded
  target, gated by the naba-ownership marker; a co-located non-naba skill is never touched; removals
  (and marker-skips) surface in the per-target `gc` report field under `--json`/`--dry-run`; covered
  by a test paralleling the stale-file test; `cargo build && cargo test` green; the post-`self update`
  refresh inherits the behavior.
- All three upstream issues closed via reconcile.
