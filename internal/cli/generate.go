package cli

import (
	"fmt"
	"os"
	"time"

	"github.com/dixson3/naba/internal/config"
	"github.com/dixson3/naba/internal/gemini"
	"github.com/dixson3/naba/internal/output"
	"github.com/spf13/cobra"
)

var (
	genStyle      string
	genCount      int
	genSeed       int
	genFormat     string
	genVariations []string
	genPreview    bool
)

func init() {
	generateCmd.Flags().StringVarP(&genStyle, "style", "s", "", "Art style (photorealistic, watercolor, oil-painting, sketch, pixel-art, anime, vintage, modern, abstract, minimalist)")
	generateCmd.Flags().IntVarP(&genCount, "count", "n", 1, "Number of variations (1-8)")
	generateCmd.Flags().IntVar(&genSeed, "seed", 0, "Seed for reproducible output")
	generateCmd.Flags().StringVar(&genFormat, "format", "separate", "Output format (grid, separate)")
	generateCmd.Flags().StringSliceVarP(&genVariations, "variation", "v", nil, "Variation types (lighting, angle, color-palette, composition, mood, season, time-of-day)")
	generateCmd.Flags().BoolVar(&genPreview, "preview", false, "Open result in system viewer")
	rootCmd.AddCommand(generateCmd)
}

var generateCmd = &cobra.Command{
	Use:   "generate <prompt>",
	Short: "Generate an image from a text prompt",
	Args:  cobra.ExactArgs(1),
	RunE:  runGenerate,
}

func runGenerate(cmd *cobra.Command, args []string) error {
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
	enrichedPrompt := gemini.EnrichGeneratePrompt(prompt, genStyle, genVariations)

	var allResults []output.Result

	for i := 0; i < genCount; i++ {
		if !flagQuiet {
			if genCount > 1 {
				fmt.Fprintf(os.Stderr, "Generating image %d/%d...\n", i+1, genCount)
			} else {
				fmt.Fprintln(os.Stderr, "Generating image...")
			}
		}

		images, err := client.Generate(enrichedPrompt)
		if err != nil {
			return handleAPIError(err)
		}

		for j, img := range images {
			idx := i*len(images) + j
			path, err := output.WriteImage(img.Data, img.MIMEType, flagOutput, "generate", idx)
			if err != nil {
				return exitError(gemini.ExitFileIO, err.Error())
			}

			result := output.NewResult(path, "generate", prompt, start)
			result.Params = map[string]any{}
			if genStyle != "" {
				result.Params["style"] = genStyle
			}
			if len(genVariations) > 0 {
				result.Params["variations"] = genVariations
			}
			if genCount > 1 {
				result.Params["index"] = idx + 1
				result.Params["count"] = genCount
			}
			allResults = append(allResults, result)

			if !flagJSON && !flagQuiet {
				fmt.Printf("Saved: %s\n", path)
			}

			if genPreview {
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

func resolveAPIKey() string {
	return config.ResolveAPIKey()
}

func handleAPIError(err error) error {
	if apiErr, ok := err.(*gemini.APIError); ok {
		return exitError(apiErr.ExitCode, apiErr.Message)
	}
	return exitError(gemini.ExitGeneral, err.Error())
}

type exitCodeError struct {
	code    int
	message string
}

func (e *exitCodeError) Error() string {
	return e.message
}

func (e *exitCodeError) ExitCode() int {
	return e.code
}

func exitError(code int, msg string) *exitCodeError {
	return &exitCodeError{code: code, message: msg}
}
