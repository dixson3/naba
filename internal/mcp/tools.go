package mcp

import (
	mcpsdk "github.com/mark3labs/mcp-go/mcp"

	"github.com/dixson3/naba/internal/gemini"
)

// imageConfigOpts returns the shared aspect/resolution/quality tool params for the
// generative image tools. aspect/resolution map to generationConfig.imageConfig; quality
// is a model alias (fast→flash, high→pro). The same imageConfig applies to every image
// when a tool generates more than one (count/steps/sizes>1).
func imageConfigOpts() []mcpsdk.ToolOption {
	return []mcpsdk.ToolOption{
		mcpsdk.WithString("aspect",
			mcpsdk.Description("Aspect ratio (generationConfig.imageConfig.aspectRatio)"),
			mcpsdk.Enum(gemini.ValidAspectRatios...),
		),
		mcpsdk.WithString("resolution",
			mcpsdk.Description("Image resolution (generationConfig.imageConfig.imageSize)"),
			mcpsdk.Enum(gemini.ValidImageSizes...),
		),
		qualityOpt(),
	}
}

// qualityOpt returns the quality (model alias) tool param.
func qualityOpt() mcpsdk.ToolOption {
	return mcpsdk.WithString("quality",
		mcpsdk.Description("Quality tier: fast (gemini-3.1-flash-image) or high (gemini-3-pro-image)"),
		mcpsdk.Enum("fast", "high"),
	)
}

func generateImageTool() mcpsdk.Tool {
	opts := []mcpsdk.ToolOption{
		mcpsdk.WithDescription("Generate an image from a text prompt"),
		mcpsdk.WithString("prompt",
			mcpsdk.Required(),
			mcpsdk.Description("The text prompt describing the image to generate"),
		),
		mcpsdk.WithString("style",
			mcpsdk.Description("Art style"),
			mcpsdk.Enum("photorealistic", "watercolor", "oil-painting", "sketch", "pixel-art", "anime", "vintage", "modern", "abstract", "minimalist"),
		),
		mcpsdk.WithArray("variations",
			mcpsdk.Description("Variation types to apply"),
			mcpsdk.WithStringItems(
				mcpsdk.Enum("lighting", "angle", "color-palette", "composition", "mood", "season", "time-of-day"),
			),
		),
		mcpsdk.WithNumber("count",
			mcpsdk.Description("Number of variations to generate (1-8)"),
			mcpsdk.DefaultNumber(1),
			mcpsdk.Min(1),
			mcpsdk.Max(8),
		),
		mcpsdk.WithNumber("seed",
			mcpsdk.Description("Seed for reproducible output"),
		),
	}
	opts = append(opts, imageConfigOpts()...)
	return mcpsdk.NewTool("generate_image", opts...)
}

func editImageTool() mcpsdk.Tool {
	opts := []mcpsdk.ToolOption{
		mcpsdk.WithDescription("Edit an existing image based on a text prompt"),
		mcpsdk.WithString("prompt",
			mcpsdk.Required(),
			mcpsdk.Description("The text prompt describing the edits to make"),
		),
		mcpsdk.WithString("file",
			mcpsdk.Required(),
			mcpsdk.Description("The file path of the input image to edit"),
		),
	}
	opts = append(opts, imageConfigOpts()...)
	return mcpsdk.NewTool("edit_image", opts...)
}

func restoreImageTool() mcpsdk.Tool {
	opts := []mcpsdk.ToolOption{
		mcpsdk.WithDescription("Restore or enhance an existing image"),
		mcpsdk.WithString("file",
			mcpsdk.Required(),
			mcpsdk.Description("The file path of the input image to restore"),
		),
		mcpsdk.WithString("prompt",
			mcpsdk.Description("The text prompt describing the restoration to perform"),
		),
	}
	opts = append(opts, imageConfigOpts()...)
	return mcpsdk.NewTool("restore_image", opts...)
}

func generateIconTool() mcpsdk.Tool {
	return mcpsdk.NewTool("generate_icon",
		mcpsdk.WithDescription("Generate app icons in multiple sizes"),
		mcpsdk.WithString("prompt",
			mcpsdk.Required(),
			mcpsdk.Description("Description of the icon to generate"),
		),
		mcpsdk.WithArray("sizes",
			mcpsdk.Description("Icon sizes in pixels (e.g. 64, 128, 256, 512)"),
			mcpsdk.WithNumberItems(
				mcpsdk.Min(16),
				mcpsdk.Max(1024),
			),
		),
		mcpsdk.WithString("style",
			mcpsdk.Description("Visual style of the icon"),
			mcpsdk.DefaultString("modern"),
			mcpsdk.Enum("flat", "skeuomorphic", "minimal", "modern"),
		),
		mcpsdk.WithString("background",
			mcpsdk.Description("Background type"),
			mcpsdk.DefaultString("transparent"),
		),
		mcpsdk.WithString("corners",
			mcpsdk.Description("Corner style"),
			mcpsdk.DefaultString("rounded"),
			mcpsdk.Enum("rounded", "sharp"),
		),
		mcpsdk.WithString("format",
			mcpsdk.Description("Output format"),
			mcpsdk.DefaultString("png"),
			mcpsdk.Enum("png", "jpeg"),
		),
		// icon takes the model-selecting quality param but no imageConfig: --size is
		// canvas pixels, a different concept from imageConfig.imageSize.
		qualityOpt(),
	)
}

func generatePatternTool() mcpsdk.Tool {
	opts := []mcpsdk.ToolOption{
		mcpsdk.WithDescription("Generate seamless patterns and textures"),
		mcpsdk.WithString("prompt",
			mcpsdk.Required(),
			mcpsdk.Description("Description of the pattern to generate"),
		),
		mcpsdk.WithString("style",
			mcpsdk.Description("Pattern style"),
			mcpsdk.DefaultString("abstract"),
			mcpsdk.Enum("geometric", "organic", "abstract", "floral", "tech"),
		),
		mcpsdk.WithString("colors",
			mcpsdk.Description("Color scheme"),
			mcpsdk.DefaultString("colorful"),
			mcpsdk.Enum("mono", "duotone", "colorful"),
		),
		mcpsdk.WithString("density",
			mcpsdk.Description("Element density"),
			mcpsdk.DefaultString("medium"),
			mcpsdk.Enum("sparse", "medium", "dense"),
		),
		mcpsdk.WithString("size",
			mcpsdk.Description("Pattern tile size (e.g. 256x256, 512x512)"),
			mcpsdk.DefaultString("256x256"),
		),
		mcpsdk.WithString("repeat",
			mcpsdk.Description("Tiling method"),
			mcpsdk.DefaultString("tile"),
			mcpsdk.Enum("tile", "mirror"),
		),
	}
	opts = append(opts, imageConfigOpts()...)
	return mcpsdk.NewTool("generate_pattern", opts...)
}

func generateStoryTool() mcpsdk.Tool {
	opts := []mcpsdk.ToolOption{
		mcpsdk.WithDescription("Generate a sequence of images that tell a visual story"),
		mcpsdk.WithString("prompt",
			mcpsdk.Required(),
			mcpsdk.Description("Description of the story to visualize"),
		),
		mcpsdk.WithNumber("steps",
			mcpsdk.Description("Number of sequential images (2-8)"),
			mcpsdk.DefaultNumber(4),
			mcpsdk.Min(2),
			mcpsdk.Max(8),
		),
		mcpsdk.WithString("style",
			mcpsdk.Description("Visual consistency across frames"),
			mcpsdk.DefaultString("consistent"),
			mcpsdk.Enum("consistent", "evolving"),
		),
		mcpsdk.WithString("transition",
			mcpsdk.Description("Transition style between frames"),
			mcpsdk.DefaultString("smooth"),
			mcpsdk.Enum("smooth", "dramatic", "fade"),
		),
		mcpsdk.WithString("layout",
			mcpsdk.Description("Output layout format"),
			mcpsdk.DefaultString("separate"),
			mcpsdk.Enum("separate", "grid", "comic"),
		),
	}
	opts = append(opts, imageConfigOpts()...)
	return mcpsdk.NewTool("generate_story", opts...)
}

func listImagesTool() mcpsdk.Tool {
	return mcpsdk.NewTool("list_images",
		mcpsdk.WithDescription("List recently generated images in the output directory"),
		mcpsdk.WithNumber("limit",
			mcpsdk.Description("Maximum number of images to return"),
			mcpsdk.DefaultNumber(20),
		),
	)
}

func generateDiagramTool() mcpsdk.Tool {
	opts := []mcpsdk.ToolOption{
		mcpsdk.WithDescription("Generate technical diagrams and flowcharts"),
		mcpsdk.WithString("prompt",
			mcpsdk.Required(),
			mcpsdk.Description("Description of the diagram to generate"),
		),
		mcpsdk.WithString("type",
			mcpsdk.Description("Type of diagram"),
			mcpsdk.DefaultString("flowchart"),
			mcpsdk.Enum("flowchart", "architecture", "network", "database", "wireframe", "mindmap", "sequence"),
		),
		mcpsdk.WithString("style",
			mcpsdk.Description("Visual style"),
			mcpsdk.DefaultString("professional"),
			mcpsdk.Enum("professional", "clean", "hand-drawn", "technical"),
		),
		mcpsdk.WithString("layout",
			mcpsdk.Description("Layout orientation"),
			mcpsdk.DefaultString("hierarchical"),
			mcpsdk.Enum("horizontal", "vertical", "hierarchical", "circular"),
		),
		mcpsdk.WithString("complexity",
			mcpsdk.Description("Level of detail"),
			mcpsdk.DefaultString("detailed"),
			mcpsdk.Enum("simple", "detailed", "comprehensive"),
		),
		mcpsdk.WithString("colors",
			mcpsdk.Description("Color scheme"),
			mcpsdk.DefaultString("accent"),
			mcpsdk.Enum("mono", "accent", "categorical"),
		),
	}
	opts = append(opts, imageConfigOpts()...)
	return mcpsdk.NewTool("generate_diagram", opts...)
}
