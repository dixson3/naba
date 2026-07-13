# Plan: Port yoshiko-flow self-update + vendor install and skills preflight into naba

**ID:** plan-005-james-dixson-55765d
**Author:** james-dixson
**Created:** 2026-07-12
**Status:** reconciling
**Epic:** naba-mol-ueh
**Fingerprint:** ceed9aef45bf34bd15a32c328502929b91b2cc3d203eeb66e49833ac49ba0c04
**Phase log:**
- 2026-07-12 scoping: initial scope captured
- 2026-07-12 drafting: scope locked via 4 answers (cargo-dist, offline key-present preflight, full self parity, dedicated skills preflight); yf reference captured
- 2026-07-12 review: plan v1 presented — red-team REVISE (pass-1); 6 concerns resolved
- 2026-07-12 review: plan v2 re-reviewed — red-team APPROVE (pass-2)
- 2026-07-12 ready-for-approval: ready-check green — last red-team APPROVE (pass-2) + audit pass
- 2026-07-12 approved: operator approved
- 2026-07-12 intake: epic naba-mol-ueh poured
- 2026-07-12 executing: start gate resolved
- 2026-07-12 reconciling: post-execution reconciliation; DAG drained

## Objective

Bring naba to feature parity with yoshiko-flow (`yf`) on two subsystems:

1. **Self-update + vendor install** — a `naba self update|install|uninstall` command group
   and a cargo-dist-generated `curl|sh` vendor installer, while **Homebrew remains the
   documented default** install path.
2. **Skills preflight** — a dedicated `naba skills preflight --json` skill-gate that validates
   API/auth (provider key present) and that the installed skills **and** the binary are up to
   date, invoked by `skills/naba/SKILL.md` at trigger time.

The canonical reference for the yf implementation this ports from is
[`references/yf-reference-report.md`](references/yf-reference-report.md).

## Motivation

naba was rewritten in Rust (plan-004) and now ships as a standalone binary distributed via a
hand-rolled `release.yml` (cross-compile + Homebrew tap). Two capabilities that yoshiko-flow
already has are missing:

- **No in-place upgrade path.** A user who installed naba outside Homebrew has no `naba self
  update`; they must re-download manually. yf solved this with a vendor install (`~/.local/bin`
  + a receipt) plus a source-aware `self update` that refuses on Homebrew installs and points
  to `brew upgrade`.
- **No skill-facing preflight.** naba's embedded skill (`skills/naba`) has no gate that a skill
  invocation can call to confirm the environment is ready — that the provider API key is set,
  and that the on-disk skill files and the binary are current. `naba doctor` does a full sweep
  but is not a lightweight per-invocation gate. yf's `yf preflight <skill> --json` is exactly
  this gate (though yf deliberately omits API/auth — naba adds it, per operator decision).

Bringing these over makes naba self-maintaining and gives the naba skill a fast readiness
check, matching the operational model already proven in yoshiko-flow.

## Scope Decisions (operator-confirmed 2026-07-12)

| Decision | Choice | Consequence |
|:---|:---|:---|
| Release tooling | **Adopt cargo-dist** | Replace hand-rolled `release.yml`; cargo-dist generates the `curl\|sh` installer (writes the receipt), the Homebrew formula, and `dist-manifest.json` that `self update` reads. |
| Preflight auth depth | **Key-present, offline** | Preflight resolves the effective provider and confirms a key is set — no network call in the hot path. Live probe stays in `naba doctor`. |
| `self` surface | **Full parity** | `self update` + `self install --from-build` + `self uninstall`, path-primary source classification, Homebrew-refuse. |
| Preflight CLI | **Dedicated `naba skills preflight`** | New `--json` skill-gate envelope; `doctor` stays the full env sweep (mirrors yf's doctor/preflight split). |

## Upstream Issues

| Issue | Title | Disposition | Notes | Resolved By |
|:---|:---|:---|:---|:---|
| #5 | Retire Go source once Rust parity is trusted | exclude | Unrelated to self-update/preflight; independent follow-on. | — |

No upstream issue tracks this work; a single coarse tracking issue is filed at intake per the
project convention. No reconcile gate is required (nothing incorporated). One **follow-on** is
created by this plan — the first cargo-dist release that activates self-update (see
[Follow-on Work](#follow-on-work)) — filed as a deferred bead and hoisted upstream at
land-the-plane.

## Investigation Findings

No separate investigation phase was run: the yf implementation **is** the investigation, and
its structure, module tree, receipt schema, source-classification precedence, dist-manifest
version-discovery, and preflight JSON contract are captured verbatim in
[`references/yf-reference-report.md`](references/yf-reference-report.md). Key facts driving the
approach:

- naba is a **single-package** crate (no `[workspace]` table). cargo-dist requires a
  `[workspace]` (or `[package.metadata.dist]`) surface — the port adds a `[workspace]` table to
  naba's `Cargo.toml` so the package is its own workspace root, then `[workspace.metadata.dist]`
  + `[profile.dist]`.
- naba is **async** (tokio + reqwest) whereas yf is sync (ureq). The port **reuses reqwest**
  for the manifest/artifact download (via `tokio::task::spawn_blocking` or the existing async
  runtime) rather than adding `ureq`. `sha2` is already a naba dependency; `flate2` + `tar` +
  `self-replace` are new.
- naba **already has** skills embedding (`src/embed.rs`) and `naba skills
  install|upgrade|remove|status` (`src/skills.rs`), plus a provider-aware `naba doctor`
  (`src/doctor.rs`) that already does `api_key`/`api_live`/`model_reachable` and per-skill
  status. Feature 2 therefore **reuses** these primitives — it is mostly the new preflight
  composition, not new embedding.
- naba's SPEC uses `SPEC-<AREA>-<NNN>` IDs (e.g. `SPEC-SKILLS-001`, `SPEC-EMBED-004`,
  `SPEC-DOCTOR-006`). New areas: `SPEC-SELF`, `SPEC-DIST`, `SPEC-DIRS`, `SPEC-PREFLIGHT`.

## Approach

Four epics, executed roughly in order (A → B → C → D), with D (docs/spec/tests) trailing each
feature. Feature 1 = Epics A + B; Feature 2 = Epic C. Epic D reconciles docs/SPEC/tests/parity
across both.

**Epic A — Distribution & XDG dirs (cargo-dist foundation).** Add the `[workspace]` +
`[workspace.metadata.dist]` + `[profile.dist]` config to `Cargo.toml` (hand-authored per
cargo-dist's schema, as yf did when `dist` was unavailable; regenerated with `dist
init`/`dist generate` once `dist` is installed — a capability gate). Generate the cargo-dist
`release.yml` and **retire the hand-rolled one**, preserving the Homebrew-tap publish
(`dixson3/homebrew-tap`, `HOMEBREW_TAP_TOKEN`, no runtime `depends_on`). Add `src/dirs.rs`
(XDG resolution, `APP = "naba"`: `~/.config/naba`, `~/.cache/naba`, `~/.local/share/naba`,
`~/.local/bin`; honors `XDG_*`).

**Epic B — `naba self` command group.** Port the `self_cmd` module tree: `source.rs`
(classification Homebrew > FromBuild > Vendor > Unknown, path-primary), `receipt.rs` (read the
cargo-dist receipt `~/.config/naba/naba-receipt.json`; write/read naba's own
`naba-from-build.json` marker), `update.rs` (fetch `dist-manifest.json` via reqwest, compare
semver, sha256-verify the artifact, `self_replace` swap, post-update `naba skills upgrade`
refresh), `archive.rs` (pure-Rust flate2+tar `.tar.gz`), `install.rs`/`uninstall.rs`
(`--from-build`), `update_check.rs` (cache at `~/.cache/naba/update-check.json`), `nag.rs`
(throttled nudge from `version`/`doctor`). CLI: `naba self update|install|uninstall` with
`--check/--force/--binary-only/--json`. Homebrew-refuse with `brew upgrade` guidance.

**Epic C — `naba skills preflight`.** New `naba skills preflight --json` emitting a skill-gate
JSON envelope with three axes: (1) **auth** — provider-aware key-present (offline; reuse the
`resolve_provider` + `provider_api_key` helpers, promoted to a `pub(crate)` shared surface both
`doctor` and `preflight` import); (2) **skills up-to-date** — the embed marker axes
(`installed`/`up_to_date`/`complete`/`unmodified`) from `embed::skill_status` against the
resolved dest; (3) **binary up-to-date** — **tri-state** `up_to_date | update_available |
unknown`, read from the `update_check` cache (`~/.cache/naba/update-check.json`). The cache is
**absent by default** on every fresh install until a cargo-dist release exists and has been
fetched once, so `unknown` (cache absent or stale) is **non-blocking** — the overall preflight
status stays `ok`, and only an affirmative `update_available` is surfaced (never blocks). No
network in the hot path. Status enum + exit codes mirror the doctor/preflight convention. Wire
`skills/naba/SKILL.md` to call it at trigger time.

**Epic D — SPEC, docs, tests, parity.** New SPEC sections (`SPEC-SELF-*`, `SPEC-DIST-*`,
`SPEC-DIRS-*`, `SPEC-PREFLIGHT-*`); README (keep Homebrew default, add self-update + vendor
`curl|sh` sections, document `skills preflight`); update AGENTS.md and `DRIFT-CHECK.md`
manifest; unit tests (source classification, receipt parse, sha256, archive extract, preflight
axes) and parity-suite entries (new commands are **Rust-only** — no Go analog, so exempt from
the Go-captured goldens; recorded explicitly in the parity suite).

## Epics

### Epic A: Distribution & XDG dirs (cargo-dist foundation)

- Issue A.1: Add `[workspace]` + `[workspace.metadata.dist]` + `[profile.dist]` to `Cargo.toml`
  (installers `shell`+`homebrew`, tap `dixson3/homebrew-tap`, `install-path = ~/.local/bin`,
  `unix-archive = .tar.gz`, 4-target matrix, `checksum = sha256`, **no** homebrew
  `depends_on`). Hand-author per cargo-dist schema.
- Issue A.2: Add `src/dirs.rs` — XDG dir resolution (`APP = "naba"`, config/cache/data/bin,
  honoring `XDG_CACHE_HOME/XDG_DATA_HOME/XDG_BIN_HOME`). **Config-dir resolution defers to the
  existing `config.rs` resolver** (`NABA_CONFIG_DIR` → `XDG_CONFIG_HOME` → `~/.config/naba`) as
  the single source of truth, so `self`/`preflight` never diverge from `config`. Document that
  the **receipt lookup path must match the cargo-dist installer's fixed `~/.config/naba`**, and
  add a precedence test covering `NABA_CONFIG_DIR`/`XDG_CONFIG_HOME` set vs unset. Also promote
  `resolve_provider` + `provider_api_key` from `doctor.rs` (currently private) to a `pub(crate)`
  shared surface for Epic C reuse.
  - depends-on: A.1
- Issue A.3: Generate the cargo-dist `release.yml` (via `dist init`/`dist generate`) and
  **retire** the hand-rolled `release.yml`; verify the Homebrew-tap publish job is preserved.
  Diff the generated formula **and the release-asset names and tag-trigger glob** against the
  current workflow (`v*` → `**[0-9]+.[0-9]+.[0-9]+*`; `naba_darwin_arm64` →
  `naba-aarch64-apple-darwin.tar.gz`); note the tag-format change in README/AGENTS release docs.
  - depends-on: A.1
  - gated-by: Capability Gate: cargo-dist available

### Epic B: `naba self` command group (update / install / uninstall)

- Issue B.1: Add dependencies (`flate2`, `tar`, `self-replace`) and scaffold the `src/self_cmd/`
  module (`mod.rs` dispatch) + wire `Commands::SelfCmd` (`#[command(name = "self")]`) into
  `cli.rs`.
  - depends-on: A.2
- Issue B.2: `source.rs` — install-source classification (`Vendor`/`Homebrew`/`FromBuild`/
  `Unknown`, precedence Homebrew > FromBuild > Vendor > Unknown, path-primary via canonicalized
  `current_exe()`), `auto_updatable()`/`nag_eligible()`, `refusal_guidance()`. Unit tests for
  each classification branch.
  - depends-on: B.1
- Issue B.3: `receipt.rs` — read the cargo-dist receipt (`~/.config/naba/naba-receipt.json`,
  tolerate unknown keys, `canonical_install_prefix()`); write/read naba's own
  `naba-from-build.json` marker (atomic temp+rename). Unit tests incl. a real-shape receipt
  fixture.
  - depends-on: B.1, A.2
- Issue B.4: `archive.rs` — pure-Rust `.tar.gz` extraction (flate2+tar), tolerating an
  enclosing `naba-<triple>/naba` dir; `parse_sha256_file` + `sha256_hex`. Unit tests.
  - depends-on: B.1
- Issue B.5: `update.rs` — the update pipeline (fetch `dist-manifest.json` via reqwest, select
  host artifact, compare semver, download + sha256-verify, `self_replace` swap), behind a
  `Fetcher` seam for testing; `--check` short-circuit; source-gate refusal; Homebrew-refuse.
  - depends-on: B.2, B.3, B.4
- Issue B.6: Post-update skills refresh (REQ parity) — after swap, exec the **swap-destination**
  binary (captured before the swap) → `naba skills upgrade --scope user --surface <surface>`
  per present surface; fail-soft. `--binary-only` skips it.
  - depends-on: B.5
- Issue B.7: `install.rs` / `uninstall.rs` (`self install --from-build`, `self uninstall`) +
  `update_check.rs` cache (`~/.cache/naba/update-check.json`) + `nag.rs` throttled nudge wired
  into `version`/`doctor` (honoring `NABA_NO_UPDATE_CHECK`/`CI`).
  - depends-on: B.5, A.2

### Epic C: `naba skills preflight` (skill-gate)

- Issue C.1: Add `naba skills preflight --json` to the `skills` subcommand surface (share
  `scope`/`surface`/`target` resolution with `skills`/`doctor`).
  - depends-on: B.3
  - note: depends only on the update-check cache **path** (from `dirs.rs`, A.2) plus the
    absent-cache → `unknown` tri-state contract — **not** the full B.7 `update_check.rs` writer /
    nag wiring — so a self-update slip does not block preflight; the binary axis reads `unknown`
    until B.7 populates the cache.
- Issue C.2: Implement the three preflight axes — auth (offline provider key-present, reusing
  the promoted `resolve_provider`/`provider_api_key` helpers), skills-up-to-date
  (`embed::skill_status`), binary-up-to-date (**tri-state** from the `update_check` cache;
  `unknown` on absent/stale cache is non-blocking) — and the JSON status envelope + exit codes.
  Unit-test the **absent-cache path** (must yield `unknown`, overall `ok`).
  - depends-on: C.1
- Issue C.3: Wire `skills/naba/SKILL.md` to invoke `naba skills preflight --json` at trigger
  time and branch on its status (mirrors the yf skill-preflight convention). Update the embedded
  skill content.
  - depends-on: C.2

### Epic D: SPEC, docs, tests, parity

- Issue D.1: SPEC.md — add `SPEC-DIST-*`, `SPEC-DIRS-*`, `SPEC-SELF-*`, `SPEC-PREFLIGHT-*`
  sections describing the shipped behavior; keep IDs traceable to tests. Must pin: the
  **binary-axis tri-state contract** (`unknown` non-blocking), the **receipt-path precedence**
  (`config.rs` resolver as single source of truth; installer's fixed `~/.config/naba`), and that
  **end-to-end `self update` is deferred to post-first-release** (unit-tested via the `Fetcher`
  seam at execution; live verification is the follow-on bead below).
  - depends-on: A.3, B.7, C.3
- Issue D.2: README + AGENTS.md — keep Homebrew as documented default, add a self-update
  section, add the vendor `curl|sh` install section, document `naba skills preflight`; update
  AGENTS.md architecture/env-var tables (`NABA_NO_UPDATE_CHECK`, XDG dirs).
  - depends-on: A.3, B.7, C.3
- Issue D.3: Parity + DRIFT-CHECK — record the new `self`/`skills preflight` commands as
  **Rust-only** in the parity suite (exempt from Go goldens), add SPEC↔test traceability rows,
  and update `DRIFT-CHECK.md` manifest edges for the new source/docs/spec surfaces.
  - depends-on: D.1, D.2

## Gates

### Start Gate (mandatory)
- Type: human
- Approvers: operator

### Capability Gate: cargo-dist available
- Type: human
- Approvers: operator
- Condition: the `dist` (cargo-dist) CLI is installed so `release.yml` can be generated with
  `dist init`/`dist generate` rather than fully hand-authored.
- Test: `dist --version` (or `cargo dist --version`) exits 0.
- Blocks: Issue A.3
- Instructions: install cargo-dist (`cargo install cargo-dist --locked` or the vendor
  installer), pinned to a known version (yf used `0.32.0`). If unavailable, A.3 falls back to
  hand-authoring the workflow from the cargo-dist schema (as yf did) and this gate is satisfied
  by operator attestation that the hand-authored workflow was reviewed.

### Release-secret note (informational, not an execution gate)
- The real release publish needs the `HOMEBREW_TAP_TOKEN` secret and the `dixson3/homebrew-tap`
  repo (both already used by the current `release.yml`). These are verified at an actual tagged
  release, **not** during plan execution — execution builds and tests the config, it does not
  cut a release.

## Risks & Mitigations

| Risk | Mitigation |
|:---|:---|
| cargo-dist not installed locally | Hand-author `[workspace.metadata.dist]` + `release.yml` from the schema (yf precedent); capability gate + operator attestation. |
| Retiring `release.yml` breaks the Homebrew tap flow | cargo-dist's `publish-homebrew-formula` job pushes to the same `dixson3/homebrew-tap` with `HOMEBREW_TAP_TOKEN`; diff the generated formula against the current prebuilt-binary formula before deleting the old workflow. Land on a branch; a bad release.yml never reaches `main` untested. |
| `self_replace` on a running tokio binary | `self_replace` swaps the on-disk file, not the running image; do the download in a blocking context (`spawn_blocking`) and swap after verify. Covered by unit tests behind the `Fetcher` seam. |
| Preflight latency on every skill invocation | Auth is key-present only (no network); binary-up-to-date reads a cache populated out-of-band by `nag`/`update_check`. No network in the preflight hot path. |
| No real vendor install exists yet → receipt absent | Source classification falls back to `Unknown` (refuses `self update` without `--force`) — identical to yf's behavior; documented, not a bug. Homebrew and from-build paths still classify correctly. |
| Parity goldens are Go-captured; `self`/`preflight` have no Go analog | New commands are Rust-only and explicitly exempted in the parity suite; no Go baseline drift. |
| naba single-package vs cargo-dist workspace assumption | Add a `[workspace]` table to `Cargo.toml` so the package is its own workspace root; verify `cargo build`/`cargo test` still pass with the added table. |

## Follow-on Work

- **Cut the first cargo-dist release** (deferred; filed as a `discovered-from` follow-on bead at
  intake and hoisted upstream at land-the-plane). Execution of this plan **builds and tests** the
  distribution config, `self update` pipeline (behind the `Fetcher` seam), and preflight — it
  does **not** cut a release. `self update` and the binary-up-to-date axis are therefore **inert
  against a live endpoint** until the `dist-manifest.json` URL exists. This bead cuts the first
  `v<semver>` tag, confirms the `curl|sh` installer writes `~/.config/naba/naba-receipt.json`,
  and verifies `naba self update` end-to-end against the published manifest. Referenced by
  Success Criteria 1 and 3.

## Success Criteria

1. `naba self update` fetches the cargo-dist `dist-manifest.json`, verifies the artifact
   sha256, and swaps the binary in place for a **vendor** install; **refuses** on a Homebrew
   install with `brew upgrade naba` guidance; `--check` reports without swapping; `--json`
   emits the documented envelope. **Verified at execution via the `Fetcher` seam** (unit tests);
   **live end-to-end verification is the follow-on release bead** (above).
2. `naba self install --from-build` and `naba self uninstall` work and manage the
   `naba-from-build.json` marker; a successful `self update` runs a post-update `naba skills
   upgrade` (unless `--binary-only`).
3. The cargo-dist config + generated `release.yml` produce, on a tagged release, a `curl|sh`
   installer that installs to `~/.local/bin` and writes `~/.config/naba/naba-receipt.json`;
   **Homebrew remains the documented default** in the README. (Config/workflow validated at
   execution; the first actual release is the follow-on bead above.)
4. `naba skills preflight --json` returns a skill-gate envelope validating provider key-present
   (offline), skills-up-to-date, and a **tri-state** binary-up-to-date axis where an absent/stale
   cache yields `unknown` and keeps the overall status `ok` (never fails a fresh install), with
   correct exit codes; `skills/naba/SKILL.md` invokes it at trigger time.
5. `cargo build`, `cargo test`, `cargo clippy -D warnings`, `cargo fmt --check`, and the parity
   suite all pass; new SPEC IDs are traceable to tests; README/AGENTS.md/SPEC.md/DRIFT-CHECK.md
   reflect the shipped behavior.
