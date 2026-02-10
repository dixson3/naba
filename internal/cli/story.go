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
	storySteps      int
	storyStyle      string
	storyTransition string
	storyLayout     string
	storyPreview    bool
)

func init() {
	storyCmd.Flags().IntVar(&storySteps, "steps", 4, "Number of frames (2-8)")
	storyCmd.Flags().StringVar(&storyStyle, "style", "consistent", "Visual consistency (consistent, evolving)")
	storyCmd.Flags().StringVar(&storyTransition, "transition", "smooth", "Transition style (smooth, dramatic, fade)")
	storyCmd.Flags().StringVar(&storyLayout, "layout", "separate", "Output layout (separate, grid, comic)")
	storyCmd.Flags().BoolVar(&storyPreview, "preview", false, "Open results in system viewer")
	rootCmd.AddCommand(storyCmd)
}

var storyCmd = &cobra.Command{
	Use:   "story <prompt>",
	Short: "Generate a sequential image series",
	Args:  cobra.ExactArgs(1),
	RunE:  runStory,
}

func runStory(cmd *cobra.Command, args []string) error {
	prompt := args[0]
	start := time.Now()

	if storySteps < 2 || storySteps > 8 {
		return exitError(gemini.ExitUsage, "steps must be between 2 and 8")
	}

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

	var allResults []output.Result

	for step := 1; step <= storySteps; step++ {
		enrichedPrompt := gemini.EnrichStoryPrompt(prompt, step, storySteps, storyStyle, storyTransition)

		if !flagQuiet {
			fmt.Fprintf(os.Stderr, "Generating frame %d/%d...\n", step, storySteps)
		}

		images, err := client.Generate(enrichedPrompt)
		if err != nil {
			return handleAPIError(err)
		}

		for _, img := range images {
			path, err := output.WriteImage(img.Data, img.MIMEType, flagOutput, "story", step-1)
			if err != nil {
				return exitError(gemini.ExitFileIO, err.Error())
			}

			result := output.NewResult(path, "story", prompt, start)
			result.Params = map[string]any{
				"step":       step,
				"total":      storySteps,
				"style":      storyStyle,
				"transition": storyTransition,
				"layout":     storyLayout,
			}
			allResults = append(allResults, result)

			if !flagJSON && !flagQuiet {
				fmt.Printf("Saved: %s\n", path)
			}

			if storyPreview {
				_ = output.Preview(path)
			}
		}
	}

	if flagJSON {
		return output.PrintJSONMulti(allResults)
	}

	return nil
}
