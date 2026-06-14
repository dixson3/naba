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
	diagramType       string
	diagramStyle      string
	diagramLayout     string
	diagramComplexity string
	diagramColors     string
	diagramPreview    bool
)

func init() {
	diagramCmd.Flags().StringVar(&diagramType, "type", "flowchart", "Diagram type (flowchart, architecture, network, database, wireframe, mindmap, sequence)")
	diagramCmd.Flags().StringVar(&diagramStyle, "style", "professional", "Visual style (professional, clean, hand-drawn, technical)")
	diagramCmd.Flags().StringVar(&diagramLayout, "layout", "hierarchical", "Layout (horizontal, vertical, hierarchical, circular)")
	diagramCmd.Flags().StringVar(&diagramComplexity, "complexity", "detailed", "Detail level (simple, detailed, comprehensive)")
	diagramCmd.Flags().StringVar(&diagramColors, "colors", "accent", "Color scheme (mono, accent, categorical)")
	diagramCmd.Flags().BoolVar(&diagramPreview, "preview", false, "Open result in system viewer")
	addImageConfigFlags(diagramCmd)
	addQualityFlag(diagramCmd)
	rootCmd.AddCommand(diagramCmd)
}

var diagramCmd = &cobra.Command{
	Use:   "diagram <prompt>",
	Short: "Generate technical diagrams",
	Args:  cobra.ExactArgs(1),
	RunE:  runDiagram,
}

func runDiagram(cmd *cobra.Command, args []string) error {
	prompt := args[0]
	start := time.Now()

	apiKey := resolveAPIKey()
	if apiKey == "" {
		return exitError(gemini.ExitAuth, "GEMINI_API_KEY not set.\n\nSet it with: export GEMINI_API_KEY=<your-key>\nOr run: naba config set api_key <your-key>")
	}

	cfg, _ := config.Load()
	model, err := resolveModel(cmd, cfg)
	if err != nil {
		return handleAPIError(err)
	}

	imgCfg, err := resolveImageConfig(cmd, cfg)
	if err != nil {
		return handleAPIError(err)
	}

	client := gemini.NewClient(apiKey, model)
	enrichedPrompt := gemini.EnrichDiagramPrompt(prompt, diagramType, diagramStyle, diagramLayout, diagramComplexity, diagramColors)

	if !flagQuiet {
		fmt.Fprintln(os.Stderr, "Generating diagram...")
	}

	images, err := client.GenerateWithConfig(enrichedPrompt, imgCfg)
	if err != nil {
		return handleAPIError(err)
	}

	var allResults []output.Result
	for i, img := range images {
		w, err := writeAndReport(img.Data, img.MIMEType, flagOutput, "diagram", i)
		if err != nil {
			return err
		}
		path := w.Path

		result := output.NewResult(path, "diagram", prompt, start)
		applyFormat(&result, w)
		result.Params = map[string]any{
			"type":       diagramType,
			"style":      diagramStyle,
			"layout":     diagramLayout,
			"complexity": diagramComplexity,
			"colors":     diagramColors,
		}
		applyImageConfigParams(result.Params, imgCfg)
		allResults = append(allResults, result)

		if !flagJSON && !flagQuiet {
			fmt.Printf("Saved: %s\n", path)
		}

		if diagramPreview {
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
