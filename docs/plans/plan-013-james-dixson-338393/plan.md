---
type: Plan
okf_spec: OKF-PLAN
id: plan-013-james-dixson-338393
author: james-dixson
created: '2026-07-23'
status: complete
deliverable_class: standard
fingerprint: b9cea91a3178b93ac97fd03b08ac6a6a451d3daec6506227396a470267bf1ae0
epic: naba-mol-7z1
---
# Plan: Port yoshiko-flow web theme structure to naba web/ (sticky header, left nav tree, centered content, right TOC) with a dark + light-blue accent color scheme

**ID:** plan-013-james-dixson-338393
**Author:** james-dixson
**Created:** 2026-07-23
**Status:** complete
**Deliverable-class:** standard
**Epic:** naba-mol-7z1
**Fingerprint:** b9cea91a3178b93ac97fd03b08ac6a6a451d3daec6506227396a470267bf1ae0
**Phase log:**
- 2026-07-23 scoping: initial scope captured
- 2026-07-23 investigating: 1 experiment (theme mapping) — no blocking unknowns
- 2026-07-23 drafting: plan v1 presented
- 2026-07-23 review: red-team pass-1 APPROVE (2 open concerns for operator)

## Objective

Replace naba's `naba-terminal` web theme with a port of yoshiko-flow's `yoshikoflow`
documentation theme — sticky header, left nav tree, centered content, right on-page TOC —
recolored to a **dark theme with a light-blue accent** (in place of yoshiko-flow's purple).
The naba marketing site (`web/`, a Pelican static site) gets a documentation-style layout
that visually matches its sibling project while keeping naba's own content and identity.

## Motivation

naba and yoshiko-flow are sibling projects by the same author. naba's current
`naba-terminal` theme is a single-column dark green "terminal" look; yoshiko-flow ships a
more capable documentation layout (persistent left nav, right-rail TOC with scroll-spy,
responsive mobile drawer). Adopting the yoshiko-flow structure gives naba's docs better
navigation and a consistent cross-project family look, differentiated by a light-blue —
rather than purple — accent so the two sites remain distinct. The two Pelican sites already
share an identical config skeleton and the same `home_content` plugin, so this is a
low-risk theme swap rather than a rebuild (see Investigation Findings).

## Upstream Issues

| Issue | Title | Disposition | Notes | Resolved By |
|:--|:--|:--|:--|:--|

No existing GitHub issue matches this work (only open issues are #16 VOICE docs and #17
CHANGELOG, both unrelated). A single coarse tracking issue is filed at intake per the naba
upstream convention.

## Investigation Findings

Full detail in [findings/exp-001-theme-mapping.md](findings/exp-001-theme-mapping.md).
Summary:

- **Shared skeleton.** Both sites are Pelican with an identical `pelicanconf.py` skeleton,
  the same `home_content` plugin (`HOME_HERO` / `HOME_CARDS` from markdown), and the same
  `index.html` / `page.html` shape. The port is a theme swap + palette shift, not a rebuild.
- **base.html** is the main structural change: skip-link, `#nav-toggle` checkbox, the
  `.layout` grid (nav / content / toc), and an inline JS block that builds the right-rail
  TOC, scroll-spy, and mobile drawer. Ported largely verbatim.
- **nav.html is new** for naba and hand-authored. yoshiko-flow's version has a dynamic
  `SKILL_NAV` loop fed by a `skill_pages` plugin naba does **not** have — that loop is
  dropped; naba's nav groups are static.
- **header/footer** partials move the page menu into the left nav (header keeps only
  GitHub) and render the footer inside the content column.
- **style.css** is replaced with the yoshikoflow CSS, recolored purple → light blue.
- **home_content plugin needs no change** — naba's copy is functionally identical.

## Approach

Port in place under `web/`, replacing the `naba-terminal` theme with a documentation theme
directory `naba-docs`, driven from the yoshiko-flow source. Work in the plan's execution
worktree; verify by building the site (`web/` Makefile) and inspecting the generated output.

1. **Scaffold the new theme dir** `web/themes/naba-docs/` by copying yoshiko-flow's
   `yoshikoflow` templates + CSS as the starting point, then adapt to naba (below). Point
   `THEME` in `pelicanconf.py` at it and retire `naba-terminal`.
2. **Recolor** the CSS `:root` accent tokens purple → light blue (`--accent #7cc4f0`,
   `--accent-hi #a9d8f5`, `--accent-soft rgba(124,196,240,0.14)`); keep the dark
   background/surface tokens. Swap the inline favicon accent in `base.html`.
3. **Author naba's left nav** (`partials/nav.html`) with static groups — Getting started
   (Overview, Install, Usage) and Reference (Config, Skills, MCP) — and no `SKILL_NAV` loop.
4. **Adapt header/footer/index/page** partials to naba's brand, `NABA_RELEASE` version
   variable, and content (hero + feature cards unchanged in data).
5. **Build + verify**: build clean, confirm the layout renders (nav, TOC, scroll-spy,
   mobile drawer), the palette is light-blue, and accent contrast is legible on surfaces.
6. **Cleanup + docs**: remove the old theme dir and stale `output/`, update any `web/`
   README or AGENTS note that references the theme name.

### Deliverable class

`standard` — the deliverable is a static site theme, fully observable from a local build.
Not `ci-release`.

## Epics

### Epic 1: Theme scaffold + palette

- Issue 1.1: Create `web/themes/naba-docs/` and copy the yoshiko-flow `yoshikoflow`
  templates (`base.html`, `index.html`, `page.html`, `notfound.html`,
  `partials/{header,footer,nav}.html`) and `static/css/style.css` as the starting point.
- Issue 1.2: Recolor `static/css/style.css` `:root` tokens purple → light blue
  (`--accent`, `--accent-hi`, `--accent-soft`); verify on-accent text contrast
  (skip-link, brand-glyph, primary button, active nav item) stays legible.
  - depends-on: 1.1
- Issue 1.3: Point `THEME` at `themes/naba-docs` in `pelicanconf.py`; update the inline
  favicon accent color in `base.html` to light blue.
  - depends-on: 1.1

### Epic 2: naba structural adaptation

- Issue 2.1: Author `partials/nav.html` with naba's static nav groups (Getting started:
  Overview/Install/Usage; Reference: Config/Skills/MCP); drop the `SKILL_NAV` loop and
  active-state logic for pages naba doesn't have.
  - depends-on: 1.1
- Issue 2.2: Adapt `partials/header.html` (naba brand, `NABA_RELEASE`, GitHub link) and
  `partials/footer.html` (naba footer links, MIT/author) to naba.
  - depends-on: 1.1
- Issue 2.3: Reconcile `index.html` (hero + `HOME_CARDS`) and `page.html` with the new
  base template; decide and apply the hero/card glyph treatment consistently.
  - depends-on: 1.1

### Epic 3: Build, verify, cleanup

- Issue 3.1: Build the site with the `web/` Makefile; fix any template/CSS errors so the
  build is clean.
  - depends-on: 1.2, 1.3, 2.1, 2.2, 2.3
- Issue 3.2: Verify rendered output — left nav highlights the current page, right TOC
  builds with scroll-spy, mobile drawer toggles, palette is light-blue, no console errors.
  - depends-on: 3.1
- Issue 3.3: Remove the retired `web/themes/naba-terminal/` dir and regenerate `output/`;
  update any `web/README.md` / AGENTS reference to the old theme name.
  - depends-on: 3.2

## Gates

### Start Gate (mandatory)

- Type: human
- Approvers: operator

### Capability Gate: site builds clean

- Type: auto
- Condition: the naba `web/` site builds without error on the new theme
- Test: `cd web && make html` (exit 0)
- Blocks: Issue 3.2, Issue 3.3
- Instructions: run the build in the execute worktree; resolve template/CSS errors before
  proceeding to visual verification and cleanup.

## Risks & Mitigations

| Risk | Mitigation |
|:--|:--|
| Light-blue accent has poorer contrast than purple on dark surfaces (buttons, active nav). | Contrast check in Issue 1.2; keep on-accent text dark (`#17131f`); nudge `--accent` lighter/darker if a WCAG AA target on `--surface` isn't met. |
| yoshiko-flow CSS references classes naba's templates don't emit (or vice versa) after dropping `SKILL_NAV`. | Reconcile in Issue 2.3 + build in 3.1; the visual check in 3.2 catches unstyled/broken elements. |
| Theme rename (`naba-terminal` → `naba-docs`) leaves dangling references (Makefile, README, `output/`). | Issue 3.3 greps for the old name and regenerates `output/`; the clean build in 3.1 fails loudly if `THEME` is misconfigured. |
| Porting yoshiko-flow's inline base.html JS drifts from its source and breaks TOC/scroll-spy. | Port verbatim in Issue 1.1; verify TOC + scroll-spy behavior explicitly in 3.2. |

## Success Criteria

1. naba `web/` builds clean (`make html`, exit 0) using the new `naba-docs` theme.
2. The rendered site has the yoshiko-flow structure: sticky header, left nav tree,
   centered content, right on-page TOC with scroll-spy, and a working mobile nav drawer.
3. The palette is a dark theme with a **light-blue** accent (no purple remains in the CSS
   or favicon), and accent-colored interactive elements are legible on dark surfaces.
4. naba's own content is intact — hero, feature cards, and all pages (install, usage,
   config, skills, mcp) render correctly with the new left-nav grouping.
5. The retired `naba-terminal` theme is removed and no stale references to it remain.
