//! clap-derive CLI surface for all 12 command groups (SPEC §1–§4).
//!
//! Help strings, flag long/short names, types, and defaults are reproduced verbatim
//! from SPEC §3 (help prose is a sanctioned divergence zone, SPEC-DIVERGE-001; wording
//! that drops "Gemini" for multi-provider follows the [DIVERGENCE] notes).

use clap::{Args, Parser, Subcommand};

/// Root command (SPEC-INV-003).
#[derive(Parser, Debug)]
#[command(
    name = "naba",
    about = "Nanobanana image generation CLI",
    long_about = "Generate, edit, and transform images using AI (multi-provider: Gemini, OpenRouter).",
    disable_version_flag = true
)]
pub struct Cli {
    // ---- Global / persistent flags (SPEC-GLOBAL-001, SPEC-GLOBAL-002) ----
    /// Output structured JSON
    #[arg(long, global = true)]
    pub json: bool,

    /// Output file path or directory
    #[arg(short = 'o', long, global = true)]
    pub output: Option<String>,

    /// Suppress progress output
    #[arg(short = 'q', long, global = true)]
    pub quiet: bool,

    /// Override model
    #[arg(short = 'm', long, global = true)]
    pub model: Option<String>,

    /// Disable interactive prompts
    #[arg(long = "no-input", global = true)]
    pub no_input: bool,

    /// Provider: gemini or openrouter
    #[arg(long, global = true)]
    pub provider: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

/// Shared imageConfig flags: `--aspect` / `--resolution` / `--quality` (SPEC-IMG-003/004).
#[derive(Args, Debug)]
pub struct ImageConfigArgs {
    /// Aspect ratio for the generated image (e.g. 1:1, 16:9, 9:16, 21:9)
    #[arg(long, default_value = "")]
    pub aspect: String,

    /// Image resolution (512, 1K, 2K, 4K)
    #[arg(long, default_value = "")]
    pub resolution: String,

    /// Quality tier: fast (flash) or high (pro). Overridden by --model
    #[arg(long, default_value = "")]
    pub quality: String,
}

/// The 12 real command groups (SPEC-INV-001).
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Generate an image from a text prompt
    Generate(GenerateArgs),
    /// Edit an existing image with instructions
    Edit(EditArgs),
    /// Restore or enhance an existing image
    Restore(RestoreArgs),
    /// Generate app icons
    Icon(IconArgs),
    /// Generate seamless patterns and textures
    Pattern(PatternArgs),
    /// Generate technical diagrams
    Diagram(DiagramArgs),
    /// Generate a sequential image series
    Story(StoryArgs),
    /// Manage configuration
    // SPEC-CONFIG-001: the parent `Long` shows the config-file location + valid keys. The path is
    // resolved at runtime (NABA_CONFIG_DIR else ~/.config/naba); help prose is [DIVERGENCE], so the
    // static long_about states the resolution rule rather than a baked absolute path.
    #[command(
        long_about = "Manage naba configuration.\n\nConfig file: $NABA_CONFIG_DIR/config.yaml (default ~/.config/naba/config.yaml)\nValid keys: api_key, model, provider, default_output_dir, aspect, resolution, quality"
    )]
    Config(ConfigArgs),
    /// Check naba's environment health (skills, API key, model, config)
    Doctor(DoctorArgs),
    /// Install, upgrade, remove, or check naba's binary-embedded skills
    Skills(SkillsArgs),
    /// Start MCP server for AI tool integration
    // SPEC-MCP-CLI-001 pins the Long string (rendered by `mcp --help`).
    #[command(
        long_about = "Start a stdio-based Model Context Protocol server that exposes all image generation capabilities as MCP tools for AI assistants."
    )]
    Mcp,
    /// Update, install, or uninstall the naba binary itself
    // SPEC-SELF-001: literal subcommand `self`; the variant is `SelfCmd` because `Self` is a
    // reserved keyword. Update refuses on Homebrew installs (use `brew upgrade naba`).
    #[command(name = "self")]
    SelfCmd(SelfArgs),
    /// Show version information
    Version,
}

// ---- self (SPEC-SELF) ----
#[derive(Args, Debug)]
pub struct SelfArgs {
    #[command(subcommand)]
    pub command: SelfCommand,
}

#[derive(Subcommand, Debug)]
pub enum SelfCommand {
    /// Update naba in place to the latest published release (vendor installs only)
    Update(SelfUpdateArgs),
    /// Install the currently-running build to ~/.local/bin as a from-build install
    Install(SelfInstallArgs),
    /// Remove a from-build install marker (does not delete the binary)
    Uninstall(SelfUninstallArgs),
}

#[derive(Args, Debug)]
pub struct SelfUpdateArgs {
    /// Report whether an update is available without swapping the binary
    #[arg(long)]
    pub check: bool,

    /// Update even when the source is not auto-updatable (e.g. from-build/unknown)
    #[arg(long)]
    pub force: bool,

    /// Swap the binary only; skip the post-update `naba skills upgrade` refresh
    #[arg(long = "binary-only")]
    pub binary_only: bool,
}

#[derive(Args, Debug)]
pub struct SelfInstallArgs {
    /// Record this checkout's build as a from-build install (writes the marker)
    #[arg(long = "from-build")]
    pub from_build: bool,
}

#[derive(Args, Debug)]
pub struct SelfUninstallArgs {
    /// Proceed without interactive confirmation
    #[arg(long)]
    pub force: bool,
}

// ---- generate (SPEC-GEN) ----
#[derive(Args, Debug)]
pub struct GenerateArgs {
    /// Text prompt
    pub prompt: String,

    /// Art style (photorealistic, watercolor, oil-painting, sketch, pixel-art, anime, vintage, modern, abstract, minimalist)
    #[arg(short = 's', long, default_value = "")]
    pub style: String,

    /// Number of variations (1-8)
    #[arg(short = 'n', long, default_value_t = 1)]
    pub count: i64,

    /// Seed for reproducible output
    #[arg(long, default_value_t = 0)]
    pub seed: i64,

    /// Output format (grid, separate)
    #[arg(long, default_value = "separate")]
    pub format: String,

    /// Variation types (lighting, angle, color-palette, composition, mood, season, time-of-day)
    #[arg(short = 'v', long)]
    pub variation: Vec<String>,

    /// Open result in system viewer
    #[arg(long)]
    pub preview: bool,

    #[command(flatten)]
    pub image: ImageConfigArgs,
}

// ---- edit (SPEC-EDIT) ----
#[derive(Args, Debug)]
pub struct EditArgs {
    /// Input image file
    pub file: String,
    /// Edit instructions
    pub prompt: String,

    /// Open result in system viewer
    #[arg(long)]
    pub preview: bool,

    #[command(flatten)]
    pub image: ImageConfigArgs,
}

// ---- restore (SPEC-RESTORE) ----
#[derive(Args, Debug)]
pub struct RestoreArgs {
    /// Input image file
    pub file: String,
    /// Optional restore instructions
    pub prompt: Option<String>,

    /// Open result in system viewer
    #[arg(long)]
    pub preview: bool,

    #[command(flatten)]
    pub image: ImageConfigArgs,
}

// ---- icon (SPEC-ICON) — `--quality` only, no aspect/resolution ----
#[derive(Args, Debug)]
pub struct IconArgs {
    /// Text prompt
    pub prompt: String,

    /// Visual style (flat, skeuomorphic, minimal, modern)
    #[arg(long, default_value = "modern")]
    pub style: String,

    /// Icon sizes in px (repeatable)
    #[arg(long, default_values_t = [256_i64])]
    pub size: Vec<i64>,

    /// Output format (png, jpeg)
    #[arg(long, default_value = "png")]
    pub format: String,

    /// Background (transparent, white, black, or color name)
    #[arg(long, default_value = "transparent")]
    pub background: String,

    /// Corner style (rounded, sharp)
    #[arg(long, default_value = "rounded")]
    pub corners: String,

    /// Open result in system viewer
    #[arg(long)]
    pub preview: bool,

    /// Quality tier: fast (flash) or high (pro). Overridden by --model
    #[arg(long, default_value = "")]
    pub quality: String,
}

// ---- pattern (SPEC-PATTERN) ----
#[derive(Args, Debug)]
pub struct PatternArgs {
    /// Text prompt
    pub prompt: String,

    /// Pattern style (geometric, organic, abstract, floral, tech)
    #[arg(long, default_value = "abstract")]
    pub style: String,

    /// Color scheme (mono, duotone, colorful)
    #[arg(long, default_value = "colorful")]
    pub colors: String,

    /// Element density (sparse, medium, dense)
    #[arg(long, default_value = "medium")]
    pub density: String,

    /// Pattern tile size
    #[arg(long = "tile-size", default_value = "256x256")]
    pub tile_size: String,

    /// Tiling method (tile, mirror)
    #[arg(long, default_value = "tile")]
    pub repeat: String,

    /// Open result in system viewer
    #[arg(long)]
    pub preview: bool,

    #[command(flatten)]
    pub image: ImageConfigArgs,
}

// ---- diagram (SPEC-DIAGRAM) ----
#[derive(Args, Debug)]
pub struct DiagramArgs {
    /// Text prompt
    pub prompt: String,

    /// Diagram type (flowchart, architecture, network, database, wireframe, mindmap, sequence)
    #[arg(long = "type", default_value = "flowchart")]
    pub diagram_type: String,

    /// Visual style (professional, clean, hand-drawn, technical)
    #[arg(long, default_value = "professional")]
    pub style: String,

    /// Layout (horizontal, vertical, hierarchical, circular)
    #[arg(long, default_value = "hierarchical")]
    pub layout: String,

    /// Detail level (simple, detailed, comprehensive)
    #[arg(long, default_value = "detailed")]
    pub complexity: String,

    /// Color scheme (mono, accent, categorical)
    #[arg(long, default_value = "accent")]
    pub colors: String,

    /// Open result in system viewer
    #[arg(long)]
    pub preview: bool,

    #[command(flatten)]
    pub image: ImageConfigArgs,
}

// ---- story (SPEC-STORY) ----
#[derive(Args, Debug)]
pub struct StoryArgs {
    /// Text prompt
    pub prompt: String,

    /// Number of frames (2-8)
    #[arg(long, default_value_t = 4)]
    pub steps: i64,

    /// Visual consistency (consistent, evolving)
    #[arg(long, default_value = "consistent")]
    pub style: String,

    /// Transition style (smooth, dramatic, fade)
    #[arg(long, default_value = "smooth")]
    pub transition: String,

    /// Output layout (separate, grid, comic)
    #[arg(long, default_value = "separate")]
    pub layout: String,

    /// Open results in system viewer
    #[arg(long)]
    pub preview: bool,

    #[command(flatten)]
    pub image: ImageConfigArgs,
}

// ---- config (SPEC-CONFIG) ----
#[derive(Args, Debug)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommand,
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommand {
    /// Get a configuration value
    Get {
        /// Config key
        key: String,
    },
    /// Set a configuration value
    Set {
        /// Config key
        key: String,
        /// Value
        value: String,
    },
}

// ---- doctor (SPEC-DOCTOR) — shares scope/surface/target semantics with skills ----
#[derive(Args, Debug)]
pub struct DoctorArgs {
    /// user → $HOME; project → git root (else cwd)
    #[arg(long, default_value = "user")]
    pub scope: String,

    /// claude → <root>/.claude/skills; agents → <root>/.agents/skills
    #[arg(long, default_value = "claude")]
    pub surface: String,

    /// override skills destination directory (takes precedence over scope/surface)
    #[arg(long, default_value = "")]
    pub target: String,
}

// ---- skills (SPEC-SKILLS) ----
#[derive(Args, Debug)]
pub struct SkillsArgs {
    /// user → $HOME; project → git root (else cwd)
    #[arg(long, global = true, default_value = "user")]
    pub scope: String,

    /// claude → <root>/.claude/skills; agents → <root>/.agents/skills
    #[arg(long, global = true, default_value = "claude")]
    pub surface: String,

    /// override skills destination directory (takes precedence over scope/surface)
    #[arg(long, global = true, default_value = "")]
    pub target: String,

    /// print the actions that would be taken; change nothing
    #[arg(long, global = true)]
    pub dry_run: bool,

    #[command(subcommand)]
    pub command: SkillsCommand,
}

#[derive(Subcommand, Debug)]
pub enum SkillsCommand {
    /// Install embedded skills to the resolved destination
    Install,
    /// Rewrite installed skills from the embedded tree and prune stale files
    Upgrade,
    /// Remove installed skills from the destination
    Remove,
    /// Report whether installed skills are up-to-date, complete, and unmodified
    Status,
}
