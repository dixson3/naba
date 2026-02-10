package cli

import (
	"fmt"
	"strings"

	"github.com/dixson3/nba/internal/config"
	"github.com/dixson3/nba/internal/gemini"
	"github.com/spf13/cobra"
)

func init() {
	configCmd.AddCommand(configGetCmd)
	configCmd.AddCommand(configSetCmd)
	rootCmd.AddCommand(configCmd)
}

var configCmd = &cobra.Command{
	Use:   "config",
	Short: "Manage configuration",
	Long:  fmt.Sprintf("Manage nba configuration.\n\nConfig file: %s\nValid keys: %s", config.ConfigPath(), strings.Join(config.ValidKeys(), ", ")),
}

var configGetCmd = &cobra.Command{
	Use:   "get <key>",
	Short: "Get a configuration value",
	Args:  cobra.ExactArgs(1),
	RunE: func(cmd *cobra.Command, args []string) error {
		key := args[0]
		cfg, err := config.Load()
		if err != nil {
			return exitError(gemini.ExitGeneral, fmt.Sprintf("load config: %v", err))
		}
		val := cfg.Get(key)
		if val == "" {
			return exitError(gemini.ExitGeneral, fmt.Sprintf("key %q is not set\n\nValid keys: %s", key, strings.Join(config.ValidKeys(), ", ")))
		}
		fmt.Println(val)
		return nil
	},
}

var configSetCmd = &cobra.Command{
	Use:   "set <key> <value>",
	Short: "Set a configuration value",
	Args:  cobra.ExactArgs(2),
	RunE: func(cmd *cobra.Command, args []string) error {
		key, value := args[0], args[1]
		cfg, err := config.Load()
		if err != nil {
			return exitError(gemini.ExitGeneral, fmt.Sprintf("load config: %v", err))
		}
		if !cfg.Set(key, value) {
			return exitError(gemini.ExitUsage, fmt.Sprintf("unknown key %q\n\nValid keys: %s", key, strings.Join(config.ValidKeys(), ", ")))
		}
		if err := config.Save(cfg); err != nil {
			return exitError(gemini.ExitFileIO, fmt.Sprintf("save config: %v", err))
		}
		if !flagQuiet {
			fmt.Printf("Set %s = %s\n", key, value)
		}
		return nil
	},
}
