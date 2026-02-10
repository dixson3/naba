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

var editPreview bool

func init() {
	editCmd.Flags().BoolVar(&editPreview, "preview", false, "Open result in system viewer")
	rootCmd.AddCommand(editCmd)
}

var editCmd = &cobra.Command{
	Use:   "edit <file> <prompt>",
	Short: "Edit an existing image with instructions",
	Args:  cobra.ExactArgs(2),
	RunE:  runEdit,
}

func runEdit(cmd *cobra.Command, args []string) error {
	imagePath := args[0]
	prompt := args[1]
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
	enrichedPrompt := gemini.EnrichEditPrompt(prompt)

	if !flagQuiet {
		fmt.Fprintln(os.Stderr, "Editing image...")
	}

	images, err := client.GenerateWithImage(enrichedPrompt, imagePath)
	if err != nil {
		return handleAPIError(err)
	}

	var allResults []output.Result
	for i, img := range images {
		path, err := output.WriteImage(img.Data, img.MIMEType, flagOutput, "edit", i)
		if err != nil {
			return exitError(gemini.ExitFileIO, err.Error())
		}

		result := output.NewResult(path, "edit", prompt, start)
		result.Params = map[string]any{"input": imagePath}
		allResults = append(allResults, result)

		if !flagJSON && !flagQuiet {
			fmt.Printf("Saved: %s\n", path)
		}

		if editPreview {
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
