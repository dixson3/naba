package cli

import (
	"fmt"

	"github.com/spf13/cobra"
)

// Set via ldflags at build time.
var (
	Version = "dev"
	Commit  = "none"
	Date    = "unknown"
)

func init() {
	rootCmd.AddCommand(versionCmd)
}

var versionCmd = &cobra.Command{
	Use:   "version",
	Short: "Show version information",
	Run: func(cmd *cobra.Command, args []string) {
		fmt.Printf("naba %s (commit: %s, built: %s)\n", Version, Commit, Date)
	},
}
