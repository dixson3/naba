# naba — UX Contract Specification

**Status:** authoritative contract for the current Rust implementation of naba.
**Clause IDs:** `SPEC-<AREA>-NNN`. IDs are stable — never renumber; append only. The
regression suite (`tests/parity/`) references these IDs; a CI traceability check asserts
every clause maps to at least one test case.

Legend for divergence markers:

- **[PINNED]** — the Rust port must reproduce this behavior byte-/semantics-identically;
  a parity test pins it.
- **[DIVERGENCE]** — a sanctioned intentional difference (see [SPEC-DIVERGE](#14-sanctioned-divergence-zones-spec-diverge)). The suite pins
  the *inventory/semantics*, not a byte-identical snapshot.
- **[NEW]** — behavior introduced by the port (multi-provider); no Go counterpart.

## Contents

The specification is split into per-domain files. Clause IDs are stable and are never
renumbered, regardless of which file a clause lives in.

| File | Sections |
|:--|:--|
| [commands.md](commands.md) | §1 Command inventory (SPEC-INV), §2 Global flags (SPEC-GLOBAL), §3 Command groups (SPEC-`<CMD>`, incl. §3.11 skills) |
| [image-config.md](image-config.md) | §4 Validation enums & imageConfig (SPEC-IMG) |
| [providers.md](providers.md) | §5 Provider layer (SPEC-PROVIDER) |
| [configuration.md](configuration.md) | §6 Config schema & precedence (SPEC-CFGSCHEMA), §10 Config migration (SPEC-MIGRATE) |
| [exit-and-errors.md](exit-and-errors.md) | §7 Exit-code matrix (SPEC-EXIT), §9 Verbatim error strings (SPEC-ERR) |
| [json-output.md](json-output.md) | §8 JSON output shapes (SPEC-JSON) |
| [mcp.md](mcp.md) | §11 MCP surface (SPEC-MCP) |
| [skills.md](skills.md) | Skills subcommand → CLI-verb map, §12 Skill-embed (SPEC-EMBED), §18 Skills preflight (SPEC-PREFLIGHT), §19 Harness layout (SPEC-HARNESS), §20 Skills install receipt (SPEC-INSTALL) |
| [distribution.md](distribution.md) | §13 Version injection (SPEC-VERSION-BUILD), §15 Distribution (SPEC-DIST), §16 XDG directories (SPEC-DIRS), §17 Vendor install & self-update (SPEC-SELF) |
| [agent-tools.md](agent-tools.md) | **Tool-agnostic** agent-tools pattern (`AGENT-TOOLS-SKILLS`/`-MCP`/`-JSON`) — the portable contract the per-domain specs above realize; naba is the reference implementation and yoshiko-flow `yf` a conforming one. Its own `AGENT-TOOLS-*` namespace is distinct from `SPEC-*` and is **not** part of the naba parity/traceability required set. |

---

## §14 Sanctioned divergence zones (SPEC-DIVERGE)

The port is a drop-in replacement **except** for the enumerated zones below. Every
divergence is captured by a SPEC clause and covered by a semantics-level (not
byte-snapshot) test.

- **SPEC-DIVERGE-001** Help text: cobra→clap rendering differs (usage layout, flag ordering,
  auto-generated sections). Root/`--model`/`--quality`/config-keys prose may be reworded for
  multi-provider. Tests assert flag *inventory* and enum membership, not full help snapshots.
- **SPEC-DIVERGE-002** Skill integrity hashes: Go embed → Rust embed (SPEC-EMBED-004).
- **SPEC-DIVERGE-003** Version strings: build-injected values (SPEC-VERSION-BUILD-001);
  normalized in tests.
- **SPEC-DIVERGE-004** Multi-provider additions: the `--provider` flag, the `provider`
  config key, provider-aware doctor checks, and provider-named error/help strings are [NEW]
  and have no Go counterpart — they are additive, not regressions.
- **SPEC-DIVERGE-005** The multi-key → OpenRouter reroute (SPEC-PROVIDER-008) is an
  intentional precedence outcome, documented, not a divergence-as-defect.
- **SPEC-DIVERGE-006** Everything **not** enumerated in §14 is [PINNED]: any observable
  difference outside these zones is a port defect, not a sanctioned divergence.
- **SPEC-DIVERGE-007** The `naba self` command group (§17 SPEC-SELF), the
  cargo-dist distribution (§15 SPEC-DIST), the XDG dirs (§16 SPEC-DIRS), and
  `naba skills preflight` (§18 SPEC-PREFLIGHT) are **Rust-only** additions ported from
  yoshiko-flow. They have **no Go counterpart** and are exempt from the Go-captured parity
  goldens; the parity suite records them as Rust-only.
- **SPEC-DIVERGE-008** The plan-008 per-harness install layout (§19 SPEC-HARNESS), the
  skills-install receipt + receipt-driven multi-harness upgrade (§20 SPEC-INSTALL), and the
  dual-purpose two-tree skill render (SPEC-EMBED-005) are **Rust-only** additions with **no Go
  counterpart** — exempt from the Go-captured parity goldens and covered by cargo unit tests
  (`src/harness.rs`, `src/skills.rs`, `src/skills_install.rs`) + the `build.rs` render.
