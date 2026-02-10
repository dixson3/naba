package config

import "os"

const EnvAPIKey = "GEMINI_API_KEY"

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
