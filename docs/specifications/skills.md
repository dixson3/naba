# naba — Skills Specification

Clause IDs (`SPEC-<AREA>-NNN`) are stable and are never renumbered; append only.

## §12 Skill-embed (SPEC-EMBED)

- **SPEC-EMBED-001** [PINNED] The binary embeds the `skills/` tree. Marker prefix `<!--
  naba-skills:`; marker format `<!-- naba-skills: v=<version> tree=<hash> -->` injected into
  each `SKILL.md` after its YAML frontmatter (else prepended); injection is idempotent.
- **SPEC-EMBED-002** [PINNED] Tree hash `hashTree`: sha256 over, per file sorted by
  skill-relative slash path, `write(relpath bytes) then write(file bytes)`; **no newline
  normalization**; `SKILL.md`'s marker line is stripped before hashing so embedded
  (marker-free) and deployed (marked) trees hash identically.
- **SPEC-EMBED-003** [PINNED] `status`/`doctor` compare: **UpToDate** = marker's `tree=`
  hash == `EmbeddedTreeHash(name)`; **Complete** = every embedded file present on disk;
  **Unmodified** = recomputed `DeployedTreeHash(destDir)` == `EmbeddedTreeHash(name)`;
  **Installed** = `SKILL.md` present.
- **SPEC-EMBED-004** [DIVERGENCE — Concern 4 / M4] The Rust port may **reproduce** Go's
  tree-hash byte-for-byte (so existing installs keep matching), **or** consciously adopt a
  different hash format and require a one-time post-cutover `naba skills upgrade` (Issue
  5.3). Either is acceptable; the choice is recorded in Issue 4.0. The parity suite pins the
  status **semantics** (up-to-date/complete/unmodified flags behave correctly against a
  freshly-installed tree), not the hash literal.

---

## §18 Skills preflight (SPEC-PREFLIGHT) [NEW — plan-005, Rust-only]

- **SPEC-PREFLIGHT-001** [PINNED] `naba skills preflight [--json]` is a fast skill-gate emitting an
  envelope `{command:"skills preflight", status, axes:{auth, skills, binary}}` with three axes.
  It shares scope/surface/target resolution with `skills`/`doctor` and provider resolution with
  `doctor` (`resolve_provider`/`provider_api_key`/`provider_key_name`, promoted to `pub(crate)`).
- **SPEC-PREFLIGHT-002** [PINNED] **auth** axis: offline provider **key-present** (no network on
  the hot path) — resolves the effective provider and checks its key
  (`GEMINI_API_KEY`/`OPENROUTER_API_KEY`, env or config). This is naba's deliberate divergence
  from `yf preflight`, which validates no API keys.
- **SPEC-PREFLIGHT-003** [PINNED] **skills** axis: every embedded skill is installed + up-to-date
  + complete + unmodified (`embed::skill_status` against the resolved dest); the first failing
  flag drives the `detail` remediation.
- **SPEC-PREFLIGHT-004** [PINNED] **binary** axis is a **tri-state**
  (`up_to_date | update_available | unknown`) read from the update-check cache. An **absent or
  stale cache yields `unknown`, which is NON-BLOCKING** — a fresh install has no cache yet, so the
  overall status stays `ok`. Overall `status` is `auth_missing` (auth fails), else
  `skills_outdated` (skills fails), else `ok`; the binary axis never blocks. Exit code is non-zero
  on any non-`ok` status. `skills/naba/SKILL.md` invokes the gate at trigger time and branches on
  the status.
