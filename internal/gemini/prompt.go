package gemini

import (
	"fmt"
	"strings"
)

// EnrichGeneratePrompt builds an enriched prompt for image generation.
func EnrichGeneratePrompt(prompt, style string, variations []string) string {
	var parts []string
	parts = append(parts, prompt)

	if style != "" {
		parts = append(parts, fmt.Sprintf("Style: %s", style))
	}

	for _, v := range variations {
		parts = append(parts, fmt.Sprintf("Vary the %s", v))
	}

	return strings.Join(parts, ". ")
}

// EnrichEditPrompt builds a prompt for image editing.
func EnrichEditPrompt(prompt string) string {
	return fmt.Sprintf("Edit this image: %s", prompt)
}

// EnrichRestorePrompt builds a prompt for image restoration.
func EnrichRestorePrompt(prompt string) string {
	if prompt == "" {
		return "Restore and enhance this image. Improve quality, fix artifacts, and sharpen details."
	}
	return fmt.Sprintf("Restore and enhance this image: %s", prompt)
}

// EnrichIconPrompt builds a prompt for icon generation.
func EnrichIconPrompt(prompt string, style string, size int, background string, corners string) string {
	parts := []string{
		fmt.Sprintf("Generate an app icon: %s", prompt),
		fmt.Sprintf("Style: %s", style),
		fmt.Sprintf("Size: %dx%d pixels", size, size),
	}
	if background != "transparent" {
		parts = append(parts, fmt.Sprintf("Background: %s", background))
	} else {
		parts = append(parts, "Background: transparent")
	}
	if corners == "rounded" {
		parts = append(parts, "Rounded corners suitable for app icons")
	} else {
		parts = append(parts, "Sharp corners")
	}
	parts = append(parts, "Clean, centered design suitable for use as an application icon")
	return strings.Join(parts, ". ")
}

// EnrichPatternPrompt builds a prompt for pattern generation.
func EnrichPatternPrompt(prompt, style, colors, density, tileSize, repeat string) string {
	parts := []string{
		fmt.Sprintf("Generate a seamless %s pattern: %s", style, prompt),
		fmt.Sprintf("Color scheme: %s", colors),
		fmt.Sprintf("Element density: %s", density),
		fmt.Sprintf("Tile size: %s", tileSize),
	}
	if repeat == "mirror" {
		parts = append(parts, "Use mirror tiling for seamless repetition")
	} else {
		parts = append(parts, "Design for seamless tile repetition")
	}
	return strings.Join(parts, ". ")
}

// EnrichStoryPrompt builds a prompt for one frame of a story sequence.
func EnrichStoryPrompt(prompt string, step, totalSteps int, style, transition string) string {
	parts := []string{
		fmt.Sprintf("Generate frame %d of %d for a visual story: %s", step, totalSteps, prompt),
	}
	if style == "consistent" {
		parts = append(parts, "Maintain consistent visual style, characters, and setting across all frames")
	} else {
		parts = append(parts, "Allow the visual style to evolve naturally across frames")
	}

	switch transition {
	case "dramatic":
		parts = append(parts, "Use dramatic transitions between scenes")
	case "fade":
		parts = append(parts, "Use subtle, fading transitions between scenes")
	default:
		parts = append(parts, "Use smooth, natural transitions between scenes")
	}

	switch step {
	case 1:
		parts = append(parts, "This is the opening scene — establish the setting and characters")
	case totalSteps:
		parts = append(parts, "This is the final scene — bring the story to a conclusion")
	default:
		parts = append(parts, fmt.Sprintf("This is scene %d — continue developing the narrative", step))
	}

	return strings.Join(parts, ". ")
}

// EnrichDiagramPrompt builds a prompt for diagram generation.
func EnrichDiagramPrompt(prompt, diagramType, style, layout, complexity, colors string) string {
	parts := []string{
		fmt.Sprintf("Generate a %s diagram: %s", diagramType, prompt),
		fmt.Sprintf("Visual style: %s", style),
		fmt.Sprintf("Layout: %s", layout),
		fmt.Sprintf("Level of detail: %s", complexity),
		fmt.Sprintf("Color scheme: %s", colors),
		"Include clear labels and annotations",
		"Professional quality suitable for documentation or presentations",
	}
	return strings.Join(parts, ". ")
}
