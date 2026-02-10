// Package cli implements the cobra command tree for the naba CLI.
package cli

import (
	"os"

	"github.com/spf13/cobra"
)

var (
	flagJSON    bool
	flagOutput  string
	flagQuiet   bool
	flagModel   string
	flagNoInput bool
)

var rootCmd = &cobra.Command{
	Use:   "naba",
	Short: "Nanobanana image generation CLI",
	Long:  "Generate, edit, and transform images using Google Gemini AI.",
	SilenceUsage:  true,
	SilenceErrors: true,
	PersistentPreRun: func(cmd *cobra.Command, args []string) {
		// Auto-enable JSON mode when stdout is not a TTY
		if !flagJSON {
			if fi, err := os.Stdout.Stat(); err == nil {
				if fi.Mode()&os.ModeCharDevice == 0 {
					flagJSON = true
				}
			}
		}
		// Auto-enable no-input when stdin is not a TTY
		if !flagNoInput {
			if fi, err := os.Stdin.Stat(); err == nil {
				if fi.Mode()&os.ModeCharDevice == 0 {
					flagNoInput = true
				}
			}
		}
	},
}

func init() {
	rootCmd.PersistentFlags().BoolVar(&flagJSON, "json", false, "Output structured JSON")
	rootCmd.PersistentFlags().StringVarP(&flagOutput, "output", "o", "", "Output file path or directory")
	rootCmd.PersistentFlags().BoolVarP(&flagQuiet, "quiet", "q", false, "Suppress progress output")
	rootCmd.PersistentFlags().StringVarP(&flagModel, "model", "m", "", "Override Gemini model")
	rootCmd.PersistentFlags().BoolVar(&flagNoInput, "no-input", false, "Disable interactive prompts")
}

// Execute runs the root command.
func Execute() error {
	return rootCmd.Execute()
}
