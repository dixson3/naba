//! Single source of truth for naba's per-command param enums (bead `naba-lca`, plan-010).
//!
//! Each `pub const <COMMAND>_<PARAM>_VALUES: &[&str]` is the one place a per-domain enum's
//! valid-value list lives. Values are verbatim and order-preserving from the former inline
//! `"enum": [...]` literals in [`crate::mcp`].
//!
//! Three surfaces once duplicated these lists by hand and could silently drift:
//!
//! 1. [`crate::mcp`] `tools()` — JSON-Schema `"enum"` arrays. These now *reference* these
//!    constants, so mcp agreement is compile-time identity.
//! 2. `src/cli.rs` clap `--help` doc-comments — compile-time string literals that cannot
//!    interpolate a `const`, so they stay hand-authored and are *guarded* by the golden test.
//! 3. `skills/naba/commands/*.md` tables — likewise hand-authored, likewise *guarded*.
//!
//! Names are domain-qualified because `style` occurs 5× (generate/icon/pattern/diagram/story)
//! and `format`/`colors`/`layout` each occur twice — a bare per-param name would collide.
//!
//! `aspect`/`resolution` are single-sourced elsewhere (`VALID_ASPECT_RATIOS`/`VALID_IMAGE_SIZES`
//! in [`crate::provider`]) and are out of scope here. `quality` (`fast`/`high`) is prose in cli
//! and absent from the md tables, so it is deliberately NOT single-sourced and excluded from the
//! golden test.
//!
//! The `#[cfg(test)] mod enum_golden` below guards the two hand-authored surfaces against drift.

// --- generate ---

/// `generate --style` (art style).
pub const GENERATE_STYLE_VALUES: &[&str] = &[
    "photorealistic",
    "watercolor",
    "oil-painting",
    "sketch",
    "pixel-art",
    "anime",
    "vintage",
    "modern",
    "abstract",
    "minimalist",
];

/// `generate --variations` (variation types to apply). Named `variation` in cli, `variations` in mcp.
pub const GENERATE_VARIATION_VALUES: &[&str] = &[
    "lighting",
    "angle",
    "color-palette",
    "composition",
    "mood",
    "season",
    "time-of-day",
];

/// `generate --format` (output layout). Absent from mcp.rs (no enum site there); guarded in cli + md only.
pub const GENERATE_FORMAT_VALUES: &[&str] = &["grid", "separate"];

// --- icon ---

/// `icon --style` (visual style of the icon).
pub const ICON_STYLE_VALUES: &[&str] = &["flat", "skeuomorphic", "minimal", "modern"];

/// `icon --corners` (corner style).
pub const ICON_CORNERS_VALUES: &[&str] = &["rounded", "sharp"];

/// `icon --format` (output format).
pub const ICON_FORMAT_VALUES: &[&str] = &["png", "jpeg"];

// --- pattern ---

/// `pattern --style` (pattern style).
pub const PATTERN_STYLE_VALUES: &[&str] = &["geometric", "organic", "abstract", "floral", "tech"];

/// `pattern --colors` (color scheme).
pub const PATTERN_COLORS_VALUES: &[&str] = &["mono", "duotone", "colorful"];

/// `pattern --density` (element density).
pub const PATTERN_DENSITY_VALUES: &[&str] = &["sparse", "medium", "dense"];

/// `pattern --repeat` (tiling method).
pub const PATTERN_REPEAT_VALUES: &[&str] = &["tile", "mirror"];

// --- diagram ---

/// `diagram --type` (type of diagram). Named `--type` in cli, `type` in mcp.
pub const DIAGRAM_TYPE_VALUES: &[&str] = &[
    "flowchart",
    "architecture",
    "network",
    "database",
    "wireframe",
    "mindmap",
    "sequence",
];

/// `diagram --style` (visual style).
pub const DIAGRAM_STYLE_VALUES: &[&str] = &["professional", "clean", "hand-drawn", "technical"];

/// `diagram --layout` (layout orientation).
pub const DIAGRAM_LAYOUT_VALUES: &[&str] = &["horizontal", "vertical", "hierarchical", "circular"];

/// `diagram --complexity` (level of detail).
pub const DIAGRAM_COMPLEXITY_VALUES: &[&str] = &["simple", "detailed", "comprehensive"];

/// `diagram --colors` (color scheme).
pub const DIAGRAM_COLORS_VALUES: &[&str] = &["mono", "accent", "categorical"];

// --- story ---

/// `story --style` (visual consistency across frames).
pub const STORY_STYLE_VALUES: &[&str] = &["consistent", "evolving"];

/// `story --transition` (transition style between frames).
pub const STORY_TRANSITION_VALUES: &[&str] = &["smooth", "dramatic", "fade"];

/// `story --layout` (output layout format).
pub const STORY_LAYOUT_VALUES: &[&str] = &["separate", "grid", "comic"];

/// Golden drift-guard (bead `naba-lca`, plan-010 Issue 2.1).
///
/// Asserts the two *hand-authored* surfaces agree with the single-source constants above:
///
/// - **clap `--help`** — introspected via [`clap::CommandFactory`] (NOT source-parsed, so the
///   guard is refactor-proof). Each enum arg is matched on its clap **long name** (`--type` has
///   id `diagram_type`; matching the id would silently miss it), and the value list is the
///   substring between the first `(` and `)` of the help string.
/// - **`skills/naba/commands/<cmd>.md`** — the table row is matched by flag name (col 1), and the
///   value list is the **last non-empty cell** (`generate.md` is 4-col, the rest 3-col — a fixed
///   column index would false-fail).
///
/// Both are comma-split, trimmed, and compared **ordered**. `quality` (prose in cli, absent in md)
/// and `aspect`/`resolution` (single-sourced elsewhere, multi-paren help) are excluded. Values
/// carry hyphens (`oil-painting`, `time-of-day`), never a bare space, so a comma split is safe.
/// `GENERATE_FORMAT_VALUES` has no mcp site but IS present in cli help + md, so it is guarded here.
#[cfg(test)]
mod enum_golden {
    use super::*;
    use crate::cli::Cli;
    use clap::CommandFactory;

    /// One guarded enum: the constant plus where its list lives in clap and in the md tables.
    struct Case {
        values: &'static [&'static str],
        /// clap subcommand name (lowercased variant).
        subcmd: &'static str,
        /// clap arg long name (`--<long>`), which may differ from the field id.
        long: &'static str,
        /// `skills/naba/commands/<md_file>`.
        md_file: &'static str,
        /// flag as written in the md table's first cell, sans backticks.
        md_flag: &'static str,
    }

    fn cases() -> Vec<Case> {
        vec![
            Case {
                values: GENERATE_STYLE_VALUES,
                subcmd: "generate",
                long: "style",
                md_file: "generate.md",
                md_flag: "--style",
            },
            Case {
                values: GENERATE_VARIATION_VALUES,
                subcmd: "generate",
                long: "variation",
                md_file: "generate.md",
                md_flag: "--variation",
            },
            Case {
                values: GENERATE_FORMAT_VALUES,
                subcmd: "generate",
                long: "format",
                md_file: "generate.md",
                md_flag: "--format",
            },
            Case {
                values: ICON_STYLE_VALUES,
                subcmd: "icon",
                long: "style",
                md_file: "icon.md",
                md_flag: "--style",
            },
            Case {
                values: ICON_CORNERS_VALUES,
                subcmd: "icon",
                long: "corners",
                md_file: "icon.md",
                md_flag: "--corners",
            },
            Case {
                values: ICON_FORMAT_VALUES,
                subcmd: "icon",
                long: "format",
                md_file: "icon.md",
                md_flag: "--format",
            },
            Case {
                values: PATTERN_STYLE_VALUES,
                subcmd: "pattern",
                long: "style",
                md_file: "pattern.md",
                md_flag: "--style",
            },
            Case {
                values: PATTERN_COLORS_VALUES,
                subcmd: "pattern",
                long: "colors",
                md_file: "pattern.md",
                md_flag: "--colors",
            },
            Case {
                values: PATTERN_DENSITY_VALUES,
                subcmd: "pattern",
                long: "density",
                md_file: "pattern.md",
                md_flag: "--density",
            },
            Case {
                values: PATTERN_REPEAT_VALUES,
                subcmd: "pattern",
                long: "repeat",
                md_file: "pattern.md",
                md_flag: "--repeat",
            },
            Case {
                values: DIAGRAM_TYPE_VALUES,
                subcmd: "diagram",
                long: "type",
                md_file: "diagram.md",
                md_flag: "--type",
            },
            Case {
                values: DIAGRAM_STYLE_VALUES,
                subcmd: "diagram",
                long: "style",
                md_file: "diagram.md",
                md_flag: "--style",
            },
            Case {
                values: DIAGRAM_LAYOUT_VALUES,
                subcmd: "diagram",
                long: "layout",
                md_file: "diagram.md",
                md_flag: "--layout",
            },
            Case {
                values: DIAGRAM_COMPLEXITY_VALUES,
                subcmd: "diagram",
                long: "complexity",
                md_file: "diagram.md",
                md_flag: "--complexity",
            },
            Case {
                values: DIAGRAM_COLORS_VALUES,
                subcmd: "diagram",
                long: "colors",
                md_file: "diagram.md",
                md_flag: "--colors",
            },
            Case {
                values: STORY_STYLE_VALUES,
                subcmd: "story",
                long: "style",
                md_file: "story.md",
                md_flag: "--style",
            },
            Case {
                values: STORY_TRANSITION_VALUES,
                subcmd: "story",
                long: "transition",
                md_file: "story.md",
                md_flag: "--transition",
            },
            Case {
                values: STORY_LAYOUT_VALUES,
                subcmd: "story",
                long: "layout",
                md_file: "story.md",
                md_flag: "--layout",
            },
        ]
    }

    /// The comma-split value list between the first `(` and the following `)`.
    fn parse_paren_list(s: &str) -> Vec<String> {
        let open = s
            .find('(')
            .unwrap_or_else(|| panic!("help string has no '(': {s:?}"));
        let close = s[open..]
            .find(')')
            .map(|i| open + i)
            .unwrap_or_else(|| panic!("help string has no ')': {s:?}"));
        s[open + 1..close]
            .split(',')
            .map(|v| v.trim().to_string())
            .collect()
    }

    /// The help string clap renders for `--<long>` on subcommand `<subcmd>`.
    fn clap_help(subcmd: &str, long: &str) -> String {
        let cmd = Cli::command();
        let sub = cmd
            .get_subcommands()
            .find(|c| c.get_name() == subcmd)
            .unwrap_or_else(|| panic!("subcommand '{subcmd}' not found"));
        let arg = sub
            .get_arguments()
            .find(|a| a.get_long() == Some(long))
            .unwrap_or_else(|| panic!("arg --{long} not found on '{subcmd}'"));
        arg.get_help()
            .map(|h| h.to_string())
            .unwrap_or_else(|| panic!("arg --{long} on '{subcmd}' has no help"))
    }

    /// The comma-split value list from the `--flag` row's last non-empty cell in `<md_file>`.
    fn md_values(md_file: &str, flag: &str) -> Vec<String> {
        let path = format!(
            "{}/skills/naba/commands/{}",
            env!("CARGO_MANIFEST_DIR"),
            md_file
        );
        let content =
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("failed to read {path}: {e}"));
        for line in content.lines() {
            let t = line.trim();
            if !t.starts_with('|') {
                continue;
            }
            let cells: Vec<&str> = t.trim_matches('|').split('|').map(str::trim).collect();
            let Some(first) = cells.first() else { continue };
            if first.trim_matches('`') != flag {
                continue;
            }
            let last = cells
                .iter()
                .rev()
                .find(|c| !c.is_empty())
                .unwrap_or_else(|| panic!("no value cell for {flag} in {md_file}"));
            return last.split(',').map(|v| v.trim().to_string()).collect();
        }
        panic!("flag {flag} not found in {md_file}");
    }

    #[test]
    fn clap_help_matches_constants() {
        for c in cases() {
            let want: Vec<String> = c.values.iter().map(|s| s.to_string()).collect();
            let got = parse_paren_list(&clap_help(c.subcmd, c.long));
            assert_eq!(
                got, want,
                "clap `{} --{}` help drifted from {}_VALUES constant",
                c.subcmd, c.long, c.subcmd
            );
        }
    }

    #[test]
    fn md_tables_match_constants() {
        for c in cases() {
            let want: Vec<String> = c.values.iter().map(|s| s.to_string()).collect();
            let got = md_values(c.md_file, c.md_flag);
            assert_eq!(
                got, want,
                "{} table row `{}` drifted from constant",
                c.md_file, c.md_flag
            );
        }
    }
}
