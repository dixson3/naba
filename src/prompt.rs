//! Prompt builders for the image commands (Issue 4.1). Direct ports of Go
//! `internal/gemini/prompt.go` — the fragment strings and the `". "` (period-space) join
//! are **[PINNED] verbatim** (SPEC-GEN-005 / SPEC-EDIT-004 / SPEC-RESTORE-004). The mocked
//! provider records the outgoing prompt and the parity suite asserts it exactly, so any
//! wording or punctuation drift is a port defect.

/// `EnrichGeneratePrompt(prompt, style, variations)` (SPEC-GEN-005): fragments are the raw
/// `prompt`; then `Style: <style>` iff `style` is non-empty; then `Vary the <v>` per variation.
pub fn enrich_generate_prompt(prompt: &str, style: &str, variations: &[String]) -> String {
    let mut parts: Vec<String> = vec![prompt.to_string()];
    if !style.is_empty() {
        parts.push(format!("Style: {style}"));
    }
    for v in variations {
        parts.push(format!("Vary the {v}"));
    }
    parts.join(". ")
}

/// `EnrichEditPrompt(prompt)` (SPEC-EDIT-004) = `"Edit this image: " + prompt`.
pub fn enrich_edit_prompt(prompt: &str) -> String {
    format!("Edit this image: {prompt}")
}

/// `EnrichRestorePrompt(prompt)` (SPEC-RESTORE-004): an empty prompt yields the fixed default
/// string; a non-empty prompt yields `"Restore and enhance this image: " + prompt`.
pub fn enrich_restore_prompt(prompt: &str) -> String {
    if prompt.is_empty() {
        "Restore and enhance this image. Improve quality, fix artifacts, and sharpen details."
            .to_string()
    } else {
        format!("Restore and enhance this image: {prompt}")
    }
}

/// `EnrichIconPrompt(prompt, style, size, background, corners)` (SPEC-ICON-006). Fragments joined
/// with `". "`: the `Generate an app icon` line, `Style`, `Size: <size>x<size> pixels`, a
/// background line (verbatim value unless `transparent`), a corners line (rounded → app-icon
/// phrasing, else `Sharp corners`), and the trailing clean/centered line.
pub fn enrich_icon_prompt(
    prompt: &str,
    style: &str,
    size: i64,
    background: &str,
    corners: &str,
) -> String {
    let mut parts: Vec<String> = vec![
        format!("Generate an app icon: {prompt}"),
        format!("Style: {style}"),
        format!("Size: {size}x{size} pixels"),
    ];
    if background != "transparent" {
        parts.push(format!("Background: {background}"));
    } else {
        parts.push("Background: transparent".to_string());
    }
    if corners == "rounded" {
        parts.push("Rounded corners suitable for app icons".to_string());
    } else {
        parts.push("Sharp corners".to_string());
    }
    parts.push("Clean, centered design suitable for use as an application icon".to_string());
    parts.join(". ")
}

/// `EnrichPatternPrompt(prompt, style, colors, density, tileSize, repeat)` (SPEC-PATTERN-004).
pub fn enrich_pattern_prompt(
    prompt: &str,
    style: &str,
    colors: &str,
    density: &str,
    tile_size: &str,
    repeat: &str,
) -> String {
    let mut parts: Vec<String> = vec![
        format!("Generate a seamless {style} pattern: {prompt}"),
        format!("Color scheme: {colors}"),
        format!("Element density: {density}"),
        format!("Tile size: {tile_size}"),
    ];
    if repeat == "mirror" {
        parts.push("Use mirror tiling for seamless repetition".to_string());
    } else {
        parts.push("Design for seamless tile repetition".to_string());
    }
    parts.join(". ")
}

/// `EnrichDiagramPrompt(prompt, diagramType, style, layout, complexity, colors)` (SPEC-DIAGRAM-004).
pub fn enrich_diagram_prompt(
    prompt: &str,
    diagram_type: &str,
    style: &str,
    layout: &str,
    complexity: &str,
    colors: &str,
) -> String {
    let parts: Vec<String> = vec![
        format!("Generate a {diagram_type} diagram: {prompt}"),
        format!("Visual style: {style}"),
        format!("Layout: {layout}"),
        format!("Level of detail: {complexity}"),
        format!("Color scheme: {colors}"),
        "Include clear labels and annotations".to_string(),
        "Professional quality suitable for documentation or presentations".to_string(),
    ];
    parts.join(". ")
}

/// `EnrichStoryPrompt(prompt, step, totalSteps, style, transition)` (SPEC-STORY-007). `layout` is
/// collected but intentionally not passed here (SPEC-STORY-004). The style/transition/step
/// switches match Go exactly (note the em-dash `—` in the scene lines).
pub fn enrich_story_prompt(
    prompt: &str,
    step: i64,
    total_steps: i64,
    style: &str,
    transition: &str,
) -> String {
    let mut parts: Vec<String> = vec![format!(
        "Generate frame {step} of {total_steps} for a visual story: {prompt}"
    )];

    if style == "consistent" {
        parts.push(
            "Maintain consistent visual style, characters, and setting across all frames"
                .to_string(),
        );
    } else {
        parts.push("Allow the visual style to evolve naturally across frames".to_string());
    }

    match transition {
        "dramatic" => parts.push("Use dramatic transitions between scenes".to_string()),
        "fade" => parts.push("Use subtle, fading transitions between scenes".to_string()),
        _ => parts.push("Use smooth, natural transitions between scenes".to_string()),
    }

    if step == 1 {
        parts.push("This is the opening scene — establish the setting and characters".to_string());
    } else if step == total_steps {
        parts.push("This is the final scene — bring the story to a conclusion".to_string());
    } else {
        parts.push(format!(
            "This is scene {step} — continue developing the narrative"
        ));
    }

    parts.join(". ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    // ---- generate (SPEC-GEN-005) ----

    #[test]
    fn generate_basic_is_just_the_prompt() {
        assert_eq!(enrich_generate_prompt("a cat", "", &[]), "a cat");
    }

    #[test]
    fn generate_with_style() {
        assert_eq!(
            enrich_generate_prompt("a cat", "watercolor", &[]),
            "a cat. Style: watercolor"
        );
    }

    #[test]
    fn generate_with_variations() {
        assert_eq!(
            enrich_generate_prompt("a cat", "", &v(&["lighting", "angle"])),
            "a cat. Vary the lighting. Vary the angle"
        );
    }

    #[test]
    fn generate_with_style_and_variations() {
        assert_eq!(
            enrich_generate_prompt("a cat", "anime", &v(&["mood"])),
            "a cat. Style: anime. Vary the mood"
        );
    }

    // ---- edit (SPEC-EDIT-004) ----

    #[test]
    fn edit_prefixes_the_prompt() {
        assert_eq!(
            enrich_edit_prompt("make it blue"),
            "Edit this image: make it blue"
        );
    }

    // ---- restore (SPEC-RESTORE-004) ----

    #[test]
    fn restore_empty_prompt_is_the_default() {
        assert_eq!(
            enrich_restore_prompt(""),
            "Restore and enhance this image. Improve quality, fix artifacts, and sharpen details."
        );
    }

    #[test]
    fn restore_non_empty_prompt() {
        assert_eq!(
            enrich_restore_prompt("fix the colors"),
            "Restore and enhance this image: fix the colors"
        );
    }

    // ---- icon (SPEC-ICON-006) ----

    #[test]
    fn icon_default_transparent_rounded() {
        assert_eq!(
            enrich_icon_prompt("a rocket", "modern", 256, "transparent", "rounded"),
            "Generate an app icon: a rocket. Style: modern. Size: 256x256 pixels. \
             Background: transparent. Rounded corners suitable for app icons. \
             Clean, centered design suitable for use as an application icon"
        );
    }

    #[test]
    fn icon_colored_background_sharp_corners() {
        assert_eq!(
            enrich_icon_prompt("a rocket", "flat", 512, "white", "sharp"),
            "Generate an app icon: a rocket. Style: flat. Size: 512x512 pixels. \
             Background: white. Sharp corners. \
             Clean, centered design suitable for use as an application icon"
        );
    }

    // ---- pattern (SPEC-PATTERN-004) ----

    #[test]
    fn pattern_default_tile() {
        assert_eq!(
            enrich_pattern_prompt("waves", "abstract", "colorful", "medium", "256x256", "tile"),
            "Generate a seamless abstract pattern: waves. Color scheme: colorful. \
             Element density: medium. Tile size: 256x256. Design for seamless tile repetition"
        );
    }

    #[test]
    fn pattern_mirror_repeat() {
        assert_eq!(
            enrich_pattern_prompt("leaves", "floral", "duotone", "dense", "512x512", "mirror"),
            "Generate a seamless floral pattern: leaves. Color scheme: duotone. \
             Element density: dense. Tile size: 512x512. Use mirror tiling for seamless repetition"
        );
    }

    // ---- diagram (SPEC-DIAGRAM-004) ----

    #[test]
    fn diagram_all_fragments() {
        assert_eq!(
            enrich_diagram_prompt(
                "a login flow",
                "flowchart",
                "professional",
                "hierarchical",
                "detailed",
                "accent"
            ),
            "Generate a flowchart diagram: a login flow. Visual style: professional. \
             Layout: hierarchical. Level of detail: detailed. Color scheme: accent. \
             Include clear labels and annotations. \
             Professional quality suitable for documentation or presentations"
        );
    }

    // ---- story (SPEC-STORY-007) ----

    #[test]
    fn story_opening_frame_consistent_smooth() {
        assert_eq!(
            enrich_story_prompt("a hero's journey", 1, 4, "consistent", "smooth"),
            "Generate frame 1 of 4 for a visual story: a hero's journey. \
             Maintain consistent visual style, characters, and setting across all frames. \
             Use smooth, natural transitions between scenes. \
             This is the opening scene — establish the setting and characters"
        );
    }

    #[test]
    fn story_middle_frame_evolving_dramatic() {
        assert_eq!(
            enrich_story_prompt("a hero's journey", 2, 4, "evolving", "dramatic"),
            "Generate frame 2 of 4 for a visual story: a hero's journey. \
             Allow the visual style to evolve naturally across frames. \
             Use dramatic transitions between scenes. \
             This is scene 2 — continue developing the narrative"
        );
    }

    #[test]
    fn story_final_frame_fade() {
        assert_eq!(
            enrich_story_prompt("a hero's journey", 4, 4, "consistent", "fade"),
            "Generate frame 4 of 4 for a visual story: a hero's journey. \
             Maintain consistent visual style, characters, and setting across all frames. \
             Use subtle, fading transitions between scenes. \
             This is the final scene — bring the story to a conclusion"
        );
    }
}
