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

var restorePreview bool

func init() {
	restoreCmd.Flags().BoolVar(&restorePreview, "preview", false, "Open result in system viewer")
	rootCmd.AddCommand(restoreCmd)
}

var restoreCmd = &cobra.Command{
	Use:   "restore <file> [prompt]",
	Short: "Restore or enhance an existing image",
	Args:  cobra.RangeArgs(1, 2),
	RunE:  runRestore,
}

func runRestore(cmd *cobra.Command, args []string) error {
	imagePath := args[0]
	var prompt string
	if len(args) > 1 {
		prompt = args[1]
	}
	start := time.Now()

	apiKey := resolveAPIKey()
	if apiKey == "" {
		return exitError(gemini.ExitAuth, "GEMINI_API_KEY not set.\n\nSet it with: export GEMINI_API_KEY=<your-key>\nOr run: nba config set api_key <your-key>")
	}

	if _, err := os.Stat(imagePath); os.IsNotExist(err) {
		return exitError(gemini.ExitFileIO, fmt.Sprintf("input file not found: %s", imagePath))
	}

	model := flagModel
	if model == "" {
		cfg, _ := config.Load()
		model = cfg.Model
	}

	client := gemini.NewClient(apiKey, model)
	enrichedPrompt := gemini.EnrichRestorePrompt(prompt)

	if !flagQuiet {
		fmt.Fprintln(os.Stderr, "Restoring image...")
	}

	images, err := client.GenerateWithImage(enrichedPrompt, imagePath)
	if err != nil {
		return handleAPIError(err)
	}

	var allResults []output.Result
	for i, img := range images {
		path, err := output.WriteImage(img.Data, img.MIMEType, flagOutput, "restore", i)
		if err != nil {
			return exitError(gemini.ExitFileIO, err.Error())
		}

		result := output.NewResult(path, "restore", prompt, start)
		result.Params = map[string]any{"input": imagePath}
		allResults = append(allResults, result)

		if !flagJSON && !flagQuiet {
			fmt.Printf("Saved: %s\n", path)
		}

		if restorePreview {
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
