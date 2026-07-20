# Finding E7 + E8: SPEC consolidation/reconciliation + Go-remnant inventory

**Date:** 2026-07-20
**Experiments:** E7 (spec inventory + reconciliation), E8 (Go remnants)
**Confidence:** HIGH (full read of SPEC.md §1–§18, all docs/specifications/*, cli.rs, targeted
provider/config cross-check; whole-tree Go token sweep).

## E7 — Spec inventory

`SPEC.md` is **largely the current Rust contract**; the stale parts are its *framing* + a few
clause lags. Everything under `docs/specifications/` is stale (Go-era) or a duplicate.

| Document / section | Class | Notes |
|:-------------------|:------|:------|
| `SPEC.md` header (1–15) | current, **reframe** | Frames itself as "Go→Rust port", "Captured from Go naba" — stale post-cutover; body is the live Rust contract. |
| §1 INV | current w/ **divergence** | Says "14 command groups"; Rust has **15** — `self` (cli.rs:108, §17) omitted from count/list. |
| §2 GLOBAL, §3 per-command, §4 IMG | current | Match cli.rs verbatim. |
| §3.11 SKILLS | current w/ **divergence** | Lists install/upgrade/remove/status; omits **`preflight`** (cli.rs:432, §18). |
| §2 GLOBAL-002 / §1 INV-003 / §6 help prose | current w/ minor divergence | `--provider` help says "gemini or openrouter", omits **bedrock**; runtime `valid_keys()` includes it (config.rs:176-181, registry.rs:87). Sanctioned SPEC-DIVERGE-001 but reconcile. |
| §5, §6, §7, §8, §9, §10, §11, §12, §13–§18 | current | Match impl (default models, registry, nested config, self/preflight surfaces). |
| `docs/specifications/PRD.md` | **stale** (Go) | "Language: Go", cobra, goreleaser, `internal/*.go`. Superseded by SPEC §1–§14. |
| `docs/specifications/TODO.md` | **stale** | Go-era TODOs, several resolved in Rust. |
| `docs/specifications/EDD/CORE.md` | **stale** | "four-package Go module behind cmd/naba/main.go". |
| `IG/configuration.md` | duplicate (current) | Dupe of SPEC §5/§6; minimal Go residue. |
| `IG/image-generation.md` | duplicate + stale Go sample | Content current; code sample is Go (cobra/internal). |
| `IG/mcp-server.md` | duplicate + stale | "7 tools" + Go handler; SPEC §11 pins **8**. |
| `IG/output-handling.md` | duplicate (current) | Dupe of SPEC §8. |
| `IG/skills.md` | duplicate + stale refs | cobra, `internal/cli/*.go`, `go:embed`; dupe of SPEC §12 + DRIFT-CHECK. |

### Proposed target `docs/specifications/` structure (split §1–§18; clause IDs stay stable)

| New file | SPEC sections |
|:---------|:--------------|
| `README.md` (index) | Header (reframed) + clause-ID conventions + §14 SPEC-DIVERGE legend |
| `commands.md` | §1 INV, §2 GLOBAL, §3 per-command |
| `image-config.md` | §4 IMG |
| `providers.md` | §5 PROVIDER (incl. Bedrock 012/013) |
| `configuration.md` | §6 CFGSCHEMA, §10 MIGRATE |
| `exit-and-errors.md` | §7 EXIT, §9 ERR |
| `json-output.md` | §8 JSON |
| `mcp.md` | §11 MCP + §11.1 skills-as-resources |
| `skills.md` | §12 EMBED, §18 PREFLIGHT (+ **new harness-layout SPEC**, this plan) |
| `distribution.md` | §13 VERSION-BUILD, §15 DIST, §16 DIRS, §17 SELF |

- **Retire (delete):** PRD.md, TODO.md, EDD/CORE.md.
- **Merge then retire:** IG/* → harvest current content, strip Go samples/`internal/` refs, fold
  into the new files (configuration→configuration+providers; image-generation→commands+image-config;
  mcp-server→mcp; output-handling→json-output; skills→skills+mcp).
- **Reconcile lags during the split:** add `self` to §1 (15 groups); add `preflight` to §3.11;
  add bedrock to `--provider`/config help prose.

### Diary disposition — `docs/diary/26-02-21.20-45.mcp-server-refinement.md`

**Retire/archive.** Go-era build log (mcp-go, `internal/mcp/server.go`, "7 tools"); every durable
decision already in the Rust SPEC (ResourceLink→SPEC-MCP-013, XDG output→SPEC-CFGSCHEMA-004/005,
`list_images` filter→SPEC-MCP-011). No proto-plan value. Remove from scope or move to
`docs/diary/archive/`.

## E8 — Go-remnant inventory

**No Go source/build artifacts remain** (`find` → no `*.go`/`go.mod`/`go.sum`/`.goreleaser.yaml`).
All remnants are doc/config references.

| # | Remnant | Action |
|:--|:--------|:-------|
| 1 | `.golangci.yml` (whole file) | **remove** — Go linter config; lint is `cargo clippy`. |
| 2–4 | `PRD.md`, `TODO.md`, `EDD/CORE.md` | **remove** (retire; see E7). |
| 5–9 | `IG/*` Go code samples/refs | **repurpose** (merge current content into new split files, drop Go samples, retarget refs to `src/`). |
| 10 | `README.md:408` "Upgrading from the Go build" | **keep** (intentional one-time migration note; optionally time-box). |
| 11 | `Makefile:2` "`*-go` parity-baseline retired" comment | **trim** the `*-go` parenthetical. |
| 12 | `DRIFT-CHECK.md:26` Go-source-nodes narrative | **keep/trim** — nodes already retargeted to `src/` in-sentence; historical. |
| 13 | `tests/parity/README.md` (Go→Rust framing, mcp-go xfail) | **repurpose** — suite is **already pure Rust golden** (`NABA_BIN` defaults to `./naba`; imports no Go); scrub Go-baseline framing, re-verify the xfail. |
| 14 | parity docstrings (conftest/test_*/harness/cases README "Go build (default)") | **repurpose** — rewrite to "the shipped Rust binary". |
| 15 | `cases/*.yaml` "cobra said…" prose | **repurpose** — keep SPEC-DIVERGE clause cites, trim cobra-comparison prose. |
| 16 | `traceability_exemptions.yaml:22` cobra→clap | **keep** — cites sanctioned SPEC-DIVERGE-001. |
| — | `mcp.rs:701-702` "match the Go-captured golden verbatim" (from E6) | **trim** comment. |

**Net:** only hard deletes are `.golangci.yml` + the three Go-era spec docs. The parity suite needs
**no** Go harness/golden removal — it is functionally already a pure Rust golden suite; only its
documentation framing mentions Go. Everything else is doc-reference cleanup. Historical plan
records (plan-001…008) are archival, not remnants — keep.
