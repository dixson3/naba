package cli

import (
	"fmt"
	"os"
	"time"

	"github.com/dixson3/nba/internal/config"
	"github.com/dixson3/nba/internal/gemini"
	"github.com/dixson3/nba/internal/output"
	"github.com/spf13/cobra"
)

var (
	patternStyle    string
	patternColors   string
	patternDensity  string
	patternTileSize string
	patternRepeat   string
	patternPreview  bool
)

func init() {
	patternCmd.Flags().StringVar(&patternStyle, "style", "abstract", "Pattern style (geometric, organic, abstract, floral, tech)")
	patternCmd.Flags().StringVar(&patternColors, "colors", "colorful", "Color scheme (mono, duotone, colorful)")
	patternCmd.Flags().StringVar(&patternDensity, "density", "medium", "Element density (sparse, medium, dense)")
	patternCmd.Flags().StringVar(&patternTileSize, "tile-size", "256x256", "Pattern tile size")
	patternCmd.Flags().StringVar(&patternRepeat, "repeat", "tile", "Tiling method (tile, mirror)")
	patternCmd.Flags().BoolVar(&patternPreview, "preview", false, "Open result in system viewer")
	rootCmd.AddCommand(patternCmd)
}

var patternCmd = &cobra.Command{
	Use:   "pattern <prompt>",
	Short: "Generate seamless patterns and textures",
	Args:  cobra.ExactArgs(1),
	RunE:  runPattern,
}

func runPattern(cmd *cobra.Command, args []string) error {
	prompt := args[0]
	start := time.Now()

	apiKey := resolveAPIKey()
	if apiKey == "" {
		return exitError(gemini.ExitAuth, "GEMINI_API_KEY not set.\n\nSet it with: export GEMINI_API_KEY=<your-key>\nOr run: nba config set api_key <your-key>")
	}

	model := flagModel
	if model == "" {
		cfg, _ := config.Load()
		model = cfg.Model
	}

	client := gemini.NewClient(apiKey, model)
	enrichedPrompt := gemini.EnrichPatternPrompt(prompt, patternStyle, patternColors, patternDensity, patternTileSize, patternRepeat)

	if !flagQuiet {
		fmt.Fprintln(os.Stderr, "Generating pattern...")
	}

	images, err := client.Generate(enrichedPrompt)
	if err != nil {
		return handleAPIError(err)
	}

	var allResults []output.Result
	for i, img := range images {
		path, err := output.WriteImage(img.Data, img.MIMEType, flagOutput, "pattern", i)
		if err != nil {
			return exitError(gemini.ExitFileIO, err.Error())
		}

		result := output.NewResult(path, "pattern", prompt, start)
		result.Params = map[string]any{
			"style":     patternStyle,
			"colors":    patternColors,
			"density":   patternDensity,
			"tile_size": patternTileSize,
			"repeat":    patternRepeat,
		}
		allResults = append(allResults, result)

		if !flagJSON && !flagQuiet {
			fmt.Printf("Saved: %s\n", path)
		}

		if patternPreview {
			_ = output.Preview(path)
		}
	}

	if flagJSON {
		if len(allResults) == 1 {
			return output.PrintJSON(allResults[0])
		}
		return output.PrintJSONMulti(allResults)
	}

	return nil
}
