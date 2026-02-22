package cli

import (
	"github.com/dixson3/naba/internal/mcp"
	"github.com/spf13/cobra"
)

func init() {
	rootCmd.AddCommand(mcpCmd)
}

var mcpCmd = &cobra.Command{
	Use:   "mcp",
	Short: "Start MCP server for AI tool integration",
	Long:  "Start a stdio-based Model Context Protocol server that exposes all image generation capabilities as MCP tools for AI assistants.",
	Args:  cobra.NoArgs,
	RunE: func(cmd *cobra.Command, args []string) error {
		return mcp.Serve(Version)
	},
}
