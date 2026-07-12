//! Command dispatch. The image commands `generate`/`edit`/`restore` (Issue 4.1) are wired
//! end-to-end here: selector → config → imageConfig → provider → output. `config get`/`set`
//! and `version` are implemented; the remaining groups (icon/pattern/diagram/story bodies,
//! doctor, skills, mcp) are still stubs (Issues 4.2–4.4).

use std::path::Path;
use std::time::Instant;

use serde_json::{Map, Value};

use crate::cli::{
    Commands, ConfigCommand, DiagramArgs, EditArgs, GenerateArgs, IconArgs, ImageConfigArgs,
    PatternArgs, RestoreArgs, SkillsCommand, StoryArgs,
};
use crate::config::{self, Config};
use crate::error::{AppError, AppResult};
use crate::output::{self, WriteResult};
use crate::prompt;
use crate::provider::{
    build_provider, gemini, missing_key_error, resolve_selection, EnvKeys, GenerateRequest,
    ImageConfig, Selection, SelectionInputs,
};
use crate::version;

/// Effective global flags after TTY autodetect (SPEC-GLOBAL-003).
#[derive(Debug, Clone)]
pub struct Globals {
    pub json: bool,
    pub output: Option<String>,
    pub quiet: bool,
    pub model: Option<String>,
    pub no_input: bool,
    pub provider: Option<String>,
}

pub async fn dispatch(command: Commands, globals: &Globals) -> AppResult<()> {
    match command {
        Commands::Version => {
            println!("{}", version::version_line());
            Ok(())
        }
        Commands::Generate(args) => run_generate(args, globals).await,
        Commands::Edit(args) => run_edit(args, globals).await,
        Commands::Restore(args) => run_restore(args, globals).await,
        Commands::Icon(args) => run_icon(args, globals).await,
        Commands::Pattern(args) => run_pattern(args, globals).await,
        Commands::Diagram(args) => run_diagram(args, globals).await,
        Commands::Story(args) => run_story(args, globals).await,
        Commands::Config(cfg) => match cfg.command {
            // SPEC-CONFIG-002 / SPEC-ERR-008: get load error → exit 1 `load config: %v`;
            // unset key → exit 1 `key %q is not set`; else print the value.
            ConfigCommand::Get { key } => {
                let value = config::get_value(&key)?;
                println!("{value}");
                Ok(())
            }
            // SPEC-CONFIG-003 / SPEC-ERR-009: set load error → exit 1 `load config: %v`;
            // unknown key → exit 2 `unknown key %q`; save error → exit 10 `save config: %v`;
            // success (unless --quiet) → `Set %s = %s`.
            ConfigCommand::Set { key, value } => {
                config::set_value(&key, &value)?;
                if !globals.quiet {
                    println!("Set {key} = {value}");
                }
                Ok(())
            }
        },
        Commands::Doctor(args) => {
            let opts = crate::doctor::Opts {
                scope: args.scope,
                surface: args.surface,
                target: args.target,
            };
            crate::doctor::run(&opts, globals).await
        }
        Commands::Skills(sk) => {
            let opts = crate::skills::Opts {
                scope: sk.scope,
                surface: sk.surface,
                target: sk.target,
                dry_run: sk.dry_run,
                quiet: globals.quiet,
            };
            match sk.command {
                SkillsCommand::Install => crate::skills::run(crate::skills::Mode::Install, &opts),
                SkillsCommand::Upgrade => crate::skills::run(crate::skills::Mode::Upgrade, &opts),
                SkillsCommand::Remove => crate::skills::run(crate::skills::Mode::Remove, &opts),
                SkillsCommand::Status => crate::skills::status(&opts),
            }
        }
        Commands::Mcp => crate::mcp::serve().await,
    }
}

// ---------------------------------------------------------------------------
// Shared image pipeline (generate / edit / restore)
// ---------------------------------------------------------------------------

/// `Some(s)` for a non-empty string, else `None`.
fn opt(s: String) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

/// Resolve provider + model + API key for an image command, returning the [`Selection`] and the
/// loaded [`Config`] (the latter also feeds imageConfig resolution). Mirrors Go's `runGenerate`
/// preamble: config load is tolerant (a load error yields a zero config, matching Go's
/// `cfg, _ := config.Load()`), an invalid config `quality` surfaces as exit 1 (SPEC-ERR-007) via
/// `to_config_defaults`, and an empty resolved key raises the provider-named SPEC-ERR-001
/// "not set" error (exit 3) at call time.
fn resolve_selection_for(globals: &Globals, quality: &str) -> AppResult<(Selection, Config)> {
    // Tolerant load (Go ignores the config-load error in the image commands).
    let cfg = Config::load().unwrap_or_default();
    // Resolved config defaults (provider + `model`/`quality`→tier). Invalid config quality → 1.
    let cfg_defaults = cfg.to_config_defaults()?;
    // Feed the selector the *resolved* keys so config `api_key` (gemini) counts for autodetect
    // and the resolved key rides onto the Selection (SPEC-CFGSCHEMA-003 env > config).
    let env_keys = EnvKeys {
        gemini: opt(cfg.resolve_api_key()),
        openrouter: opt(cfg.resolve_openrouter_api_key()),
    };

    // SPEC-ERR-016 / SPEC-PROVIDER-007: a CLI `--model` without a CLI `--provider` is a usage
    // error (exit 2), enforced by the selector. The command layer routes straight through the
    // selector (no pre-resolution) so the guard fires (Issue 4.5 removed the 4.1 workaround).
    let inputs = SelectionInputs {
        provider: globals.provider.clone(),
        model: globals.model.clone(),
        quality: Some(quality.to_string()),
    };
    let selection = resolve_selection(&inputs, &cfg_defaults, &env_keys)?;
    // SPEC-ERR-001: empty resolved key → provider-named "not set" auth error (exit 3) at call time.
    if selection.api_key.is_empty() {
        return Err(missing_key_error(&selection.provider));
    }
    Ok((selection, cfg))
}

/// Resolve the effective imageConfig: CLI flag (set) > config (`aspect`/`resolution`) > unset
/// (SPEC-IMG-006). An empty flag string counts as unset (Go uses cobra `Changed`; a degenerate
/// explicit `--aspect ""` is untested and treated as unset here). Invalid values → exit 2.
fn resolve_image_config(image: &ImageConfigArgs, cfg: &Config) -> AppResult<Option<ImageConfig>> {
    let aspect = if image.aspect.is_empty() {
        cfg.aspect.clone()
    } else {
        image.aspect.clone()
    };
    let resolution = if image.resolution.is_empty() {
        cfg.resolution.clone()
    } else {
        image.resolution.clone()
    };
    ImageConfig::new(&aspect, &resolution)
}

/// Record the resolved aspect/resolution onto a `Result.Params` map (Go `applyImageConfigParams`):
/// keys `aspect` / `resolution`, only when present.
fn apply_image_config_params(params: &mut Map<String, Value>, cfg: &Option<ImageConfig>) {
    if let Some(c) = cfg {
        if let Some(a) = c.aspect.as_deref() {
            if !a.is_empty() {
                params.insert("aspect".into(), Value::from(a.to_string()));
            }
        }
        if let Some(s) = c.size.as_deref() {
            if !s.is_empty() {
                params.insert("resolution".into(), Value::from(s.to_string()));
            }
        }
    }
}

/// Write an image and emit the extension-correction note on stderr (unless quiet), mirroring
/// Go's `writeAndReport`. A file error → exit 10 (SPEC file IO).
fn write_and_report(
    data: &[u8],
    mime: &str,
    output_path: &str,
    command: &str,
    index: usize,
    quiet: bool,
) -> AppResult<WriteResult> {
    let w = output::write_image_result(data, mime, output_path, command, index)
        .map_err(|e| AppError::file_io(e.to_string()))?;
    if w.corrected && !quiet {
        let base = Path::new(&w.path)
            .file_name()
            .map(|f| f.to_string_lossy().into_owned())
            .unwrap_or_else(|| w.path.clone());
        eprintln!(
            "{}",
            output::extension_correction_note(&w.requested_format, &w.actual_format, &base)
        );
    }
    Ok(w)
}

/// Emit the collected results as JSON per SPEC-JSON-002 (single object for one result, array for
/// more). An empty result set prints `null` — matching Go's `PrintJSONMulti` on a nil slice
/// (e.g. `generate -n 0`, SPEC-GEN-003).
fn print_results_json(results: &[output::Result]) {
    match results.len() {
        1 => output::print_json(&results[0]),
        0 => println!("null"),
        _ => output::print_json_multi(results),
    }
}

// ---------------------------------------------------------------------------
// generate (SPEC-GEN)
// ---------------------------------------------------------------------------

async fn run_generate(args: GenerateArgs, globals: &Globals) -> AppResult<()> {
    let start = Instant::now();
    let output_path = globals.output.clone().unwrap_or_default();

    let (selection, cfg) = resolve_selection_for(globals, &args.image.quality)?;
    let image_config = resolve_image_config(&args.image, &cfg)?;
    let provider = build_provider(&selection);

    let enriched = prompt::enrich_generate_prompt(&args.prompt, &args.style, &args.variation);

    let mut all: Vec<output::Result> = Vec::new();

    // SPEC-GEN-003: --count is NOT range-validated; loop `count` times (0/negative → 0 times).
    let mut i: i64 = 0;
    while i < args.count {
        if !globals.quiet {
            if args.count > 1 {
                eprintln!("Generating image {}/{}...", i + 1, args.count);
            } else {
                eprintln!("Generating image...");
            }
        }

        let req = GenerateRequest {
            prompt: enriched.clone(),
            model: selection.model.clone(),
            image_config: image_config.clone(),
            input_image: None,
            quality: selection.quality.clone(),
        };
        let images = provider.generate(&req).await?;

        for (j, img) in images.iter().enumerate() {
            let idx = (i as usize) * images.len() + j;
            let w = write_and_report(
                &img.bytes,
                &img.mime,
                &output_path,
                "generate",
                idx,
                globals.quiet,
            )?;

            let mut result = output::Result::new(&w.path, "generate", &args.prompt, start);
            result.apply_format(&w);

            let mut params = Map::new();
            apply_image_config_params(&mut params, &image_config);
            if !args.style.is_empty() {
                params.insert("style".into(), Value::from(args.style.clone()));
            }
            if !args.variation.is_empty() {
                params.insert("variations".into(), Value::from(args.variation.clone()));
            }
            if args.count > 1 {
                params.insert("index".into(), Value::from((idx as i64) + 1));
                params.insert("count".into(), Value::from(args.count));
            }
            result.params = params;

            if !globals.json && !globals.quiet {
                println!("Saved: {}", w.path);
            }
            if args.preview {
                output::preview(&w.path);
            }
            all.push(result);
        }
        i += 1;
    }

    if globals.json {
        print_results_json(&all);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// icon (SPEC-ICON) — plain-generate path, no imageConfig, loop per --size
// ---------------------------------------------------------------------------

async fn run_icon(args: IconArgs, globals: &Globals) -> AppResult<()> {
    let start = Instant::now();
    let output_path = globals.output.clone().unwrap_or_default();

    // icon takes `--quality` (its own field, not the shared imageConfig block).
    let (selection, _cfg) = resolve_selection_for(globals, &args.quality)?;
    let provider = build_provider(&selection);

    let multi_size = args.size.len() > 1;
    let mut all: Vec<output::Result> = Vec::new();

    for (i, &size) in args.size.iter().enumerate() {
        let enriched = prompt::enrich_icon_prompt(
            &args.prompt,
            &args.style,
            size,
            &args.background,
            &args.corners,
        );

        if !globals.quiet {
            eprintln!("Generating {size}x{size} icon...");
        }

        // SPEC-ICON-003: plain generate path — no image_config sent.
        let req = GenerateRequest {
            prompt: enriched,
            model: selection.model.clone(),
            image_config: None,
            input_image: None,
            quality: selection.quality.clone(),
        };
        let images = provider.generate(&req).await?;

        for img in &images {
            // SPEC-ICON-004: output naming. `-o` empty → `icon-<size><ext>`; `-o` set with
            // multiple sizes → `<base>-<size><ext>`; single size + `-o` → `-o` verbatim.
            let out_path = if output_path.is_empty() {
                let ext = output::ext_for_format(&args.format);
                format!("icon-{size}{ext}")
            } else if multi_size {
                let ext = Path::new(&output_path)
                    .extension()
                    .map(|e| format!(".{}", e.to_string_lossy()))
                    .unwrap_or_default();
                let base = &output_path[..output_path.len() - ext.len()];
                format!("{base}-{size}{ext}")
            } else {
                output_path.clone()
            };

            let w = write_and_report(&img.bytes, &img.mime, &out_path, "icon", i, globals.quiet)?;

            let mut result = output::Result::new(&w.path, "icon", &args.prompt, start);
            result.apply_format(&w);

            let mut params = Map::new();
            params.insert("size".into(), Value::from(size));
            params.insert("style".into(), Value::from(args.style.clone()));
            params.insert("format".into(), Value::from(args.format.clone()));
            params.insert("background".into(), Value::from(args.background.clone()));
            params.insert("corners".into(), Value::from(args.corners.clone()));
            result.params = params;

            if !globals.json && !globals.quiet {
                println!("Saved: {}", w.path);
            }
            if args.preview {
                output::preview(&w.path);
            }
            all.push(result);
        }
    }

    if globals.json {
        print_results_json(&all);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// pattern (SPEC-PATTERN) — imageConfig path, single generate call
// ---------------------------------------------------------------------------

async fn run_pattern(args: PatternArgs, globals: &Globals) -> AppResult<()> {
    let start = Instant::now();
    let output_path = globals.output.clone().unwrap_or_default();

    let (selection, cfg) = resolve_selection_for(globals, &args.image.quality)?;
    let image_config = resolve_image_config(&args.image, &cfg)?;
    let provider = build_provider(&selection);

    let enriched = prompt::enrich_pattern_prompt(
        &args.prompt,
        &args.style,
        &args.colors,
        &args.density,
        &args.tile_size,
        &args.repeat,
    );

    if !globals.quiet {
        eprintln!("Generating pattern...");
    }

    let req = GenerateRequest {
        prompt: enriched,
        model: selection.model.clone(),
        image_config: image_config.clone(),
        input_image: None,
        quality: selection.quality.clone(),
    };
    let images = provider.generate(&req).await?;

    let mut all: Vec<output::Result> = Vec::new();
    for (i, img) in images.iter().enumerate() {
        let w = write_and_report(
            &img.bytes,
            &img.mime,
            &output_path,
            "pattern",
            i,
            globals.quiet,
        )?;

        let mut result = output::Result::new(&w.path, "pattern", &args.prompt, start);
        result.apply_format(&w);

        let mut params = Map::new();
        params.insert("style".into(), Value::from(args.style.clone()));
        params.insert("colors".into(), Value::from(args.colors.clone()));
        params.insert("density".into(), Value::from(args.density.clone()));
        params.insert("tile_size".into(), Value::from(args.tile_size.clone()));
        params.insert("repeat".into(), Value::from(args.repeat.clone()));
        apply_image_config_params(&mut params, &image_config);
        result.params = params;

        if !globals.json && !globals.quiet {
            println!("Saved: {}", w.path);
        }
        if args.preview {
            output::preview(&w.path);
        }
        all.push(result);
    }

    if globals.json {
        print_results_json(&all);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// diagram (SPEC-DIAGRAM) — imageConfig path, single generate call
// ---------------------------------------------------------------------------

async fn run_diagram(args: DiagramArgs, globals: &Globals) -> AppResult<()> {
    let start = Instant::now();
    let output_path = globals.output.clone().unwrap_or_default();

    let (selection, cfg) = resolve_selection_for(globals, &args.image.quality)?;
    let image_config = resolve_image_config(&args.image, &cfg)?;
    let provider = build_provider(&selection);

    let enriched = prompt::enrich_diagram_prompt(
        &args.prompt,
        &args.diagram_type,
        &args.style,
        &args.layout,
        &args.complexity,
        &args.colors,
    );

    if !globals.quiet {
        eprintln!("Generating diagram...");
    }

    let req = GenerateRequest {
        prompt: enriched,
        model: selection.model.clone(),
        image_config: image_config.clone(),
        input_image: None,
        quality: selection.quality.clone(),
    };
    let images = provider.generate(&req).await?;

    let mut all: Vec<output::Result> = Vec::new();
    for (i, img) in images.iter().enumerate() {
        let w = write_and_report(
            &img.bytes,
            &img.mime,
            &output_path,
            "diagram",
            i,
            globals.quiet,
        )?;

        let mut result = output::Result::new(&w.path, "diagram", &args.prompt, start);
        result.apply_format(&w);

        let mut params = Map::new();
        params.insert("type".into(), Value::from(args.diagram_type.clone()));
        params.insert("style".into(), Value::from(args.style.clone()));
        params.insert("layout".into(), Value::from(args.layout.clone()));
        params.insert("complexity".into(), Value::from(args.complexity.clone()));
        params.insert("colors".into(), Value::from(args.colors.clone()));
        apply_image_config_params(&mut params, &image_config);
        result.params = params;

        if !globals.json && !globals.quiet {
            println!("Saved: {}", w.path);
        }
        if args.preview {
            output::preview(&w.path);
        }
        all.push(result);
    }

    if globals.json {
        print_results_json(&all);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// story (SPEC-STORY) — imageConfig path, loop per frame, ALWAYS array JSON
// ---------------------------------------------------------------------------

async fn run_story(args: StoryArgs, globals: &Globals) -> AppResult<()> {
    let start = Instant::now();
    let output_path = globals.output.clone().unwrap_or_default();

    // SPEC-STORY-003: --steps validated BEFORE any generation → ExitUsage (exit 2).
    if args.steps < 2 || args.steps > 8 {
        return Err(AppError::usage("steps must be between 2 and 8"));
    }

    let (selection, cfg) = resolve_selection_for(globals, &args.image.quality)?;
    let image_config = resolve_image_config(&args.image, &cfg)?;
    let provider = build_provider(&selection);

    let mut all: Vec<output::Result> = Vec::new();

    for step in 1..=args.steps {
        // SPEC-STORY-004: --layout is collected but not passed to the prompt.
        let enriched = prompt::enrich_story_prompt(
            &args.prompt,
            step,
            args.steps,
            &args.style,
            &args.transition,
        );

        if !globals.quiet {
            eprintln!("Generating frame {step}/{}...", args.steps);
        }

        let req = GenerateRequest {
            prompt: enriched,
            model: selection.model.clone(),
            image_config: image_config.clone(),
            input_image: None,
            quality: selection.quality.clone(),
        };
        let images = provider.generate(&req).await?;

        for img in &images {
            let w = write_and_report(
                &img.bytes,
                &img.mime,
                &output_path,
                "story",
                (step - 1) as usize,
                globals.quiet,
            )?;

            let mut result = output::Result::new(&w.path, "story", &args.prompt, start);
            result.apply_format(&w);

            let mut params = Map::new();
            params.insert("step".into(), Value::from(step));
            params.insert("total".into(), Value::from(args.steps));
            params.insert("style".into(), Value::from(args.style.clone()));
            params.insert("transition".into(), Value::from(args.transition.clone()));
            params.insert("layout".into(), Value::from(args.layout.clone()));
            apply_image_config_params(&mut params, &image_config);
            result.params = params;

            if !globals.json && !globals.quiet {
                println!("Saved: {}", w.path);
            }
            if args.preview {
                output::preview(&w.path);
            }
            all.push(result);
        }
    }

    // SPEC-STORY-006 / SPEC-JSON-003: ALWAYS the array form, even for a single frame.
    if globals.json {
        output::print_json_multi(&all);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// edit (SPEC-EDIT) / restore (SPEC-RESTORE) — shared image-input path
// ---------------------------------------------------------------------------

async fn run_edit(args: EditArgs, globals: &Globals) -> AppResult<()> {
    run_image_input(
        globals,
        &args.file,
        prompt::enrich_edit_prompt(&args.prompt),
        &args.prompt,
        "edit",
        "Editing image...",
        &args.image,
        args.preview,
    )
    .await
}

async fn run_restore(args: RestoreArgs, globals: &Globals) -> AppResult<()> {
    let raw_prompt = args.prompt.clone().unwrap_or_default();
    run_image_input(
        globals,
        &args.file,
        prompt::enrich_restore_prompt(&raw_prompt),
        &raw_prompt,
        "restore",
        "Restoring image...",
        &args.image,
        args.preview,
    )
    .await
}

/// The shared edit/restore body: key check → input-file check → read → single provider call →
/// write + JSON. `raw_prompt` is the un-enriched prompt recorded on `Result.prompt` and (for
/// restore) empty when omitted.
#[allow(clippy::too_many_arguments)]
async fn run_image_input(
    globals: &Globals,
    file: &str,
    enriched: String,
    raw_prompt: &str,
    command: &str,
    progress: &str,
    image: &ImageConfigArgs,
    preview: bool,
) -> AppResult<()> {
    let start = Instant::now();
    let output_path = globals.output.clone().unwrap_or_default();

    // Key check first (matches Go: apiKey preflight before the os.Stat file check).
    let (selection, cfg) = resolve_selection_for(globals, &image.quality)?;

    // SPEC-EDIT-003 / SPEC-RESTORE-003: missing input → exit 10 `input file not found: %s`.
    if !Path::new(file).exists() {
        return Err(AppError::file_io(format!("input file not found: {file}")));
    }
    // Read bytes + detect MIME (SPEC-ERR-015 on read failure, exit 10).
    let input_image = gemini::read_image_file(file)?;

    let image_config = resolve_image_config(image, &cfg)?;
    let provider = build_provider(&selection);

    if !globals.quiet {
        eprintln!("{progress}");
    }

    let req = GenerateRequest {
        prompt: enriched,
        model: selection.model.clone(),
        image_config: image_config.clone(),
        input_image: Some(input_image),
        quality: selection.quality.clone(),
    };
    let images = provider.generate(&req).await?;

    let mut all: Vec<output::Result> = Vec::new();
    for (i, img) in images.iter().enumerate() {
        let w = write_and_report(
            &img.bytes,
            &img.mime,
            &output_path,
            command,
            i,
            globals.quiet,
        )?;

        let mut result = output::Result::new(&w.path, command, raw_prompt, start);
        result.apply_format(&w);

        let mut params = Map::new();
        params.insert("input".into(), Value::from(file.to_string()));
        apply_image_config_params(&mut params, &image_config);
        result.params = params;

        if !globals.json && !globals.quiet {
            println!("Saved: {}", w.path);
        }
        if preview {
            output::preview(&w.path);
        }
        all.push(result);
    }

    if globals.json {
        print_results_json(&all);
    }
    Ok(())
}
