package config

import (
	"os"
	"path/filepath"
)

const EnvAPIKey = "GEMINI_API_KEY"
const EnvOutputDir = "NABA_OUTPUT_DIR"

// ResolveAPIKey returns the Gemini API key from environment or config file.
func ResolveAPIKey() string {
	if key := os.Getenv(EnvAPIKey); key != "" {
		return key
	}
	cfg, err := Load()
	if err != nil {
		return ""
	}
	return cfg.APIKey
}

// ResolveOutputDir returns the output directory from environment or config file.
// Precedence: NABA_OUTPUT_DIR env var > default_output_dir config > empty string (CWD).
func ResolveOutputDir() string {
	if dir := os.Getenv(EnvOutputDir); dir != "" {
		return dir
	}
	cfg, err := Load()
	if err != nil {
		return ""
	}
	return cfg.DefaultOutputDir
}

// DefaultOutputDir returns the XDG-conventional default output directory
// for generated images (~/.local/share/naba/images).
func DefaultOutputDir() string {
	home, err := os.UserHomeDir()
	if err != nil {
		return ""
	}
	return filepath.Join(home, ".local", "share", "naba", "images")
}
