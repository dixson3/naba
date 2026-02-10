package cli

import (
	"fmt"
	"os"
	"path/filepath"
	"time"

	"github.com/dixson3/naba/internal/config"
	"github.com/dixson3/naba/internal/gemini"
	"github.com/dixson3/naba/internal/output"
	"github.com/spf13/cobra"
)

var (
	iconStyle      string
	iconSizes      []int
	iconFormat     string
	iconBackground string
	iconCorners    string
	iconPreview    bool
)

func init() {
	iconCmd.Flags().StringVar(&iconStyle, "style", "modern", "Visual style (flat, skeuomorphic, minimal, modern)")
	iconCmd.Flags().IntSliceVar(&iconSizes, "size", []int{256}, "Icon sizes in px (repeatable)")
	iconCmd.Flags().StringVar(&iconFormat, "format", "png", "Output format (png, jpeg)")
	iconCmd.Flags().StringVar(&iconBackground, "background", "transparent", "Background (transparent, white, black, or color name)")
	iconCmd.Flags().StringVar(&iconCorners, "corners", "rounded", "Corner style (rounded, sharp)")
	iconCmd.Flags().BoolVar(&iconPreview, "preview", false, "Open result in system viewer")
	rootCmd.AddCommand(iconCmd)
}

var iconCmd = &cobra.Command{
	Use:   "icon <prompt>",
	Short: "Generate app icons",
	Args:  cobra.ExactArgs(1),
	RunE:  runIcon,
}

func runIcon(cmd *cobra.Command, args []string) error {
	prompt := args[0]
	start := time.Now()

	apiKey := resolveAPIKey()
	if apiKey == "" {
		return exitError(gemini.ExitAuth, "GEMINI_API_KEY not set.\n\nSet it with: export GEMINI_API_KEY=<your-key>\nOr run: naba config set api_key <your-key>")
	}

	model := flagModel
	if model == "" {
		cfg, _ := config.Load()
		model = cfg.Model
	}

	client := gemini.NewClient(apiKey, model)

	var allResults []output.Result

	for i, size := range iconSizes {
		enrichedPrompt := gemini.EnrichIconPrompt(prompt, iconStyle, size, iconBackground, iconCorners)

		if !flagQuiet {
			fmt.Fprintf(os.Stderr, "Generating %dx%d icon...\n", size, size)
		}

		images, err := client.Generate(enrichedPrompt)
		if err != nil {
			return handleAPIError(err)
		}

		for _, img := range images {
			outPath := flagOutput
			if outPath == "" {
				ext := output.ExtForFormat(iconFormat)
				outPath = fmt.Sprintf("icon-%d%s", size, ext)
			} else if len(iconSizes) > 1 {
				ext := filepath.Ext(outPath)
				base := outPath[:len(outPath)-len(ext)]
				outPath = fmt.Sprintf("%s-%d%s", base, size, ext)
			}

			path, err := output.WriteImage(img.Data, img.MIMEType, outPath, "icon", i)
			if err != nil {
				return exitError(gemini.ExitFileIO, err.Error())
			}

			result := output.NewResult(path, "icon", prompt, start)
			result.Params = map[string]any{
				"size":       size,
				"style":      iconStyle,
				"format":     iconFormat,
				"background": iconBackground,
				"corners":    iconCorners,
			}
			allResults = append(allResults, result)

			if !flagJSON && !flagQuiet {
				fmt.Printf("Saved: %s\n", path)
			}

			if iconPreview {
				_ = output.Preview(path)
			}
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
