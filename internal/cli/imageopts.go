package cli

import (
	"github.com/dixson3/naba/internal/config"
	"github.com/dixson3/naba/internal/gemini"
	"github.com/spf13/cobra"
)

// Shared imageConfig flags for the generative commands (generate, edit, restore,
// pattern, diagram, story). icon is deliberately excluded: its --size is canvas pixels,
// a different concept from imageConfig.imageSize. These are package-level vars bound on
// each command; only one command runs per invocation, so the binding is unambiguous.
var (
	flagAspect     string
	flagResolution string
	flagQuality    string
)

// addImageConfigFlags registers --aspect and --resolution on a generative command.
func addImageConfigFlags(cmd *cobra.Command) {
	cmd.Flags().StringVar(&flagAspect, "aspect", "",
		"Aspect ratio for the generated image (e.g. 1:1, 16:9, 9:16, 21:9)")
	cmd.Flags().StringVar(&flagResolution, "resolution", "",
		"Image resolution (512, 1K, 2K, 4K)")
}

// addQualityFlag registers --quality on an image-producing command. It is a model
// alias (fast→gemini-3.1-flash-image, high→gemini-3-pro-image); --model overrides it.
func addQualityFlag(cmd *cobra.Command) {
	cmd.Flags().StringVar(&flagQuality, "quality", "",
		"Quality tier: fast (flash) or high (pro). Overridden by --model")
}

// resolveModel determines the model id with precedence:
//
//	--model > --quality > config model > config quality > built-in default
//
// Explicit flags are detected with cobra Changed() rather than empty-string sentinels,
// so an explicit --model "" is distinguishable from an unset flag. The intra-config
// tiebreak (config model beats config quality) is resolved in config.Config.ResolveModel.
func resolveModel(cmd *cobra.Command, cfg *config.Config) (string, error) {
	if cmd.Flags().Changed("model") && flagModel != "" {
		return flagModel, nil
	}
	if cmd.Flags().Changed("quality") {
		return gemini.ModelForQuality(flagQuality)
	}
	// Config fallback, with the intra-config tiebreak (config model beats config quality)
	// applied in config.ResolveModel.
	if cfg != nil {
		m, err := cfg.ResolveModel()
		if err != nil {
			return "", err
		}
		if m != "" {
			return m, nil
		}
	}
	return gemini.DefaultModel, nil
}

// resolveImageConfig builds a validated *gemini.ImageConfig with precedence
// flag > config > unset. A flag wins only when explicitly set (cobra Changed); otherwise
// the config default applies. Returns (nil, nil) when neither flag nor config sets a
// value, so a bare request stays byte-identical. An invalid enum (from either source)
// yields an ExitUsage error — the API silently ignores bad values.
func resolveImageConfig(cmd *cobra.Command, cfg *config.Config) (*gemini.ImageConfig, error) {
	aspect := flagAspect
	if !cmd.Flags().Changed("aspect") && cfg != nil {
		aspect = cfg.Aspect
	}
	resolution := flagResolution
	if !cmd.Flags().Changed("resolution") && cfg != nil {
		resolution = cfg.Resolution
	}
	return gemini.NewImageConfig(aspect, resolution)
}

// applyImageConfigParams records the resolved aspect/resolution on a Result.Params map
// so JSON output reflects what was actually requested.
func applyImageConfigParams(params map[string]any, cfg *gemini.ImageConfig) {
	if cfg == nil {
		return
	}
	if cfg.AspectRatio != "" {
		params["aspect"] = cfg.AspectRatio
	}
	if cfg.ImageSize != "" {
		params["resolution"] = cfg.ImageSize
	}
}
