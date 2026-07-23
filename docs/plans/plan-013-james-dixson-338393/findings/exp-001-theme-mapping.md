---
type: Finding
okf_spec: OKF-PLAN
plan: plan-013-james-dixson-338393
---
# Finding: naba-terminal → yoshikoflow theme mapping

**Date:** 2026-07-23
**Question:** What exactly differs between naba's `naba-terminal` Pelican theme and
yoshiko-flow's `yoshikoflow` theme, and what must change to port the structure with a
dark + light-blue accent scheme?

## Key result: the two sites share one skeleton

Both are Pelican static sites with an identical `pelicanconf.py` skeleton, the same
`home_content` plugin (hero + feature cards from markdown), and the same `index.html` /
`page.html` template shape. The port is a **theme swap + palette shift**, not a rebuild.

## Reference (source of truth for the port)

- naba theme: `web/themes/naba-terminal/` (dark green, single-column `.wrap` layout)
- yoshiko-flow theme: `~/workspace/dixson3/yoshiko-flow/web/themes/yoshikoflow/`
  (dark documentation layout: sticky header, left nav tree, centered content, right TOC;
  purple accent `--accent: #a78bfa`)

## File-by-file delta

| File | naba-terminal (now) | yoshikoflow (target) | Port action |
|:--|:--|:--|:--|
| `templates/base.html` | simple `<main class="wrap">` shell | skip-link + `#nav-toggle` checkbox + `.layout` grid (nav / content / toc) + inline JS that builds the right-rail TOC, scroll-spy, and mobile drawer | Replace shell; port the JS verbatim; swap favicon accent color |
| `templates/partials/nav.html` | **absent** | left nav tree, hand-authored groups + a dynamic `SKILL_NAV` loop | **New file**; hand-author naba's groups; **drop the `SKILL_NAV` loop** (naba has no `skill_pages` plugin) |
| `templates/partials/header.html` | full `MENUITEMS` nav in header | burger + brand-glyph/name/version + spacer + GitHub-only link | Port; page menu moves to the **left nav**, header keeps GitHub |
| `templates/partials/footer.html` | `.site-footer` below main | `.content-footer` inside the content column | Port; footer renders inside `<main>` per new base |
| `templates/index.html` | hero + cards, uses `.glyph` spans | near-identical, no glyph spans | Trivial; keep or drop hero/card glyphs (cosmetic) |
| `templates/page.html` | `>_` glyph in title | identical minus glyph | Trivial |
| `static/css/style.css` | terminal single-column CSS, green accent | full docs CSS (sidebar/TOC layout) + purple accent | **Replace** with yoshikoflow CSS; recolor purple → light blue |

## home_content plugin: no change

naba's `web/plugins/home_content.py` is functionally identical to yoshiko-flow's. The hero
+ cards data model (`HOME_HERO`, `HOME_CARDS`) is unchanged, so `index.html` needs no data
rewiring.

## naba pages → left-nav grouping (proposed)

naba content pages: `install`, `usage`, `config`, `skills`, `mcp` (+ home/overview).
No `skill_pages` plugin ⇒ no dynamic skills subtree. Proposed hand-authored groups:

- **Getting started** — Overview (`/`), Install, Usage
- **Reference** — Config, Skills, MCP

## Palette shift (purple → light blue)

Swap the yoshikoflow accent tokens. Proposed:

| Token | yoshikoflow (purple) | naba (light blue) |
|:--|:--|:--|
| `--accent` | `#a78bfa` | `#7cc4f0` |
| `--accent-hi` | `#c4b5fd` | `#a9d8f5` |
| `--accent-soft` | `rgba(167,139,250,0.14)` | `rgba(124,196,240,0.14)` |

Background/surface tokens stay dark (`--bg: #16161e` …). Secondary accents (`--cyan`,
`--green`, `--amber`) can stay or be nudged; the skip-link/brand-glyph `on-accent` text
color (`#17131f`) stays dark for contrast on the light-blue accent.

## Unknowns / risks (none blocking)

- **Theme dir name:** `naba-terminal` becomes a misnomer. Rename to `naba-docs` (updates
  `THEME` in `pelicanconf.py`) or keep the name to minimize churn — a cosmetic decision.
- **Secondary-accent legibility:** verify light-blue accent contrast against `--surface`
  panels (buttons, active nav item) after the build.
- **Build parity:** `make html` (or the web/ Makefile target) must build clean and the
  generated `output/theme/css/style.css` must reflect the new palette.
