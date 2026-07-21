# Finding E1 — enum inventory, single-source home, golden-test feasibility

**Experiment:** E1 (full enum inventory across 3 sites + golden-test parsing feasibility)
**Confidence:** HIGH (end-to-end read of all sites + `tests/parity/` + crate layout)
**Sources:** `src/mcp.rs` (728–1064), `src/cli.rs` (all `Args` doc-comments), `skills/naba/commands/*.md`,
`src/provider/mod.rs`, `tests/parity/{check_traceability.py,test_mcp.py,conftest.py,pyproject.toml}`,
`src/main.rs`, `Cargo.toml`, `src/embed.rs`

## Already single-sourced — EXCLUDE

- `aspect` → `VALID_ASPECT_RATIOS` (`src/provider/mod.rs:52`), 14 values, imported by mcp (`src/mcp.rs:61,736`).
- `resolution` → `VALID_IMAGE_SIZES` (`src/provider/mod.rs:60`), `512,1K,2K,4K`, imported by mcp (`src/mcp.rs:61,744`).

## The 18 per-domain enums to single-source

All are **inline `"enum": [...]` literals** in `src/mcp.rs` (not constants), duplicated in
`cli.rs` doc-comments and `commands/*.md` tables:

| # | Param | Command | Values (ordered) | mcp.rs | cli.rs | commands/*.md |
|:--|:--|:--|:--|:--|:--|:--|
| 1 | style | generate | photorealistic, watercolor, oil-painting, sketch, pixel-art, anime, vintage, modern, abstract, minimalist | 771 | 166 | generate.md:22 |
| 2 | variation(s) | generate | lighting, angle, color-palette, composition, mood, season, time-of-day | 782 | 182 | generate.md:26 |
| 3 | format | generate | grid, separate | **ABSENT** | 179 | generate.md:25 |
| 4 | style | icon | flat, skeuomorphic, minimal, modern | 865 | 245 | icon.md:23 |
| 5 | corners | icon | rounded, sharp | 878 | 249 | icon.md:27 |
| 6 | format | icon | png, jpeg | 887 | 242 | icon.md:25 |
| 7 | style | pattern | geometric, organic, abstract, floral, tech | 911 | 267 | pattern.md:21 |
| 8 | colors | pattern | mono, duotone, colorful | 920 | 271 | pattern.md:22 |
| 9 | density | pattern | sparse, medium, dense | 929 | 275 | pattern.md:23 |
| 10 | repeat | pattern | tile, mirror | 946 | 283 | pattern.md:25 |
| 11 | type | diagram | flowchart, architecture, network, database, wireframe, mindmap, sequence | 1020 | 301 | diagram.md:23 |
| 12 | style | diagram | professional, clean, hand-drawn, technical | 1030 | 305 | diagram.md:24 |
| 13 | layout | diagram | horizontal, vertical, hierarchical, circular | 1039 | 309 | diagram.md:25 |
| 14 | complexity | diagram | simple, detailed, comprehensive | 1048 | 313 | diagram.md:26 |
| 15 | colors | diagram | mono, accent, categorical | 1057 | 317 | diagram.md:27 |
| 16 | style | story | consistent, evolving | 979 | 339 | story.md:25 |
| 17 | transition | story | smooth, dramatic, fade | 988 | 343 | story.md:26 |
| 18 | layout | story | separate, grid, comic | 997 | 347 | story.md:27 |

**Current drift status: NONE** — all agree verbatim today. plan-010 is **regression prevention**,
not a bug fix.

## Special cases / exclusions

- **`quality`** (`fast,high`) — bare inline literal `src/mcp.rs:756`; in cli.rs it is **prose**
  ("fast (flash) or high (pro)", `cli.rs:58,256`); **absent** from all md tables. Cannot
  ordered-match → bespoke membership check or exclude from strict golden.
- **generate `format`** (#3) — **absent from mcp.rs**; the golden must not assert an mcp site for it.
- **Free-form (NOT enums, exclude):** icon `background` (transparent/white/black/color name,
  no `enum` key `mcp.rs:870`), pattern `tile-size`/`size` (`mcp.rs:936`), numeric ranges
  (`count` 1–8, `steps` 2–8, icon `size`).
- **Param-name mismatches across sites** (site-map must handle): `variation`(cli)/`variations`(mcp);
  `--tile-size`(cli/md)/`size`(mcp); `--type`(cli)/`type`(mcp).

## FOURTH drift site discovered

`tests/parity/test_mcp.py` (~184–262) holds its **own hardcoded copy** of nearly every enum (its
`EXPECTED` dict) — the current guard on mcp.rs, itself an unsourced duplicate. plan-010 must
account for it (keep as independent oracle, or fold into the new source).

## Single-source home

New leaf module **`src/enums.rs`** (`mod enums;` in `src/main.rs`), `pub const STYLE_VALUES:
&[&str] = &[...]`. Rationale: command-surface vocabulary, not a provider concern (provider/mod.rs
is doc-scoped to the provider abstraction). Leaf module → no dependency cycle; `mcp.rs` already
imports provider constants (line 61) and can import `crate::enums` identically; `cli.rs` imports
freely. Optionally relocate `VALID_ASPECT_RATIOS`/`VALID_IMAGE_SIZES` there for cohesion.

## Golden test — FEASIBLE; recommend a Rust `#[cfg(test)]` module

The crate is **binary-only** (no `src/lib.rs`; `Cargo.toml:13 [[bin]] path=src/main.rs`), so an
external `tests/*.rs` cannot `use` crate items. A `#[cfg(test)] mod` **inside `src/`** references
the constants directly — zero indirection. Beats a Python parity script (which would need to parse
Rust source or a generated JSON fixture — both drift-prone).

- **mcp.rs:** after refactor imports the constants → agreement is compile-time identity (trivial/no
  runtime parse).
- **cli.rs help:** introspect clap via `<Cli as clap::CommandFactory>::command()`, read each arg's
  help string, parse the parenthesized list — no source parsing, refactor-proof.
- **commands/*.md:** read from `env!("CARGO_MANIFEST_DIR")/skills/naba/commands/*.md`, parse the
  table cell.
- **Matching:** exact ordered list after normalization; **comma-split, never whitespace-split**
  (hyphenated values `oil-painting`, `pixel-art`, `color-palette`, `time-of-day`, `hand-drawn`).

**Landmine (core rationale):** clap `///` doc-comments and md tables **cannot interpolate a
`const`** (compile-time string literals) — so cli/md enum lists stay hand-maintained text that the
golden test must PARSE and diff. This is exactly why a golden test is needed rather than pure
single-sourcing: only mcp.rs can *reference* the constant; the other sites are *guarded*, not
*generated*.

## Effort/risk

18 enums (+`quality`) × up to 4 sites: ~18 mcp literals → `crate::enums` imports; ~18 cli
doc-comments + ~18 md rows left as guarded hand-text; test_mcp.py `EXPECTED` kept or folded.
Landmines: clap-can't-interpolate (→ golden must parse), `quality` prose/absence (special-case),
generate `format` mcp-absence (site-map exception).
