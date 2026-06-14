// Package config manages naba CLI configuration via YAML files and environment variables.
package config

import (
	"fmt"
	"os"
	"path/filepath"

	"gopkg.in/yaml.v3"
)

const configFileName = "config.yaml"

// Config holds the naba configuration.
type Config struct {
	APIKey           string `yaml:"api_key,omitempty"`
	Model            string `yaml:"model,omitempty"`
	DefaultOutputDir string `yaml:"default_output_dir,omitempty"`
	// Aspect / Resolution are imageConfig defaults; per-call flags override them.
	Aspect     string `yaml:"aspect,omitempty"`
	Resolution string `yaml:"resolution,omitempty"`
	// Quality is a model alias (fast/high). The model key takes precedence over it
	// (see ResolveModel), mirroring the flag precedence --model > --quality.
	Quality string `yaml:"quality,omitempty"`
}

// ConfigDir returns the configuration directory path.
func ConfigDir() string {
	if dir := os.Getenv("NABA_CONFIG_DIR"); dir != "" {
		return dir
	}
	home, _ := os.UserHomeDir()
	return filepath.Join(home, ".config", "naba")
}

// ConfigPath returns the full path to the config file.
func ConfigPath() string {
	return filepath.Join(ConfigDir(), configFileName)
}

// Load reads the config file. Returns a zero Config if the file doesn't exist.
func Load() (*Config, error) {
	cfg := &Config{}
	data, err := os.ReadFile(ConfigPath())
	if err != nil {
		if os.IsNotExist(err) {
			return cfg, nil
		}
		return nil, err
	}
	if err := yaml.Unmarshal(data, cfg); err != nil {
		return nil, err
	}
	return cfg, nil
}

// Save writes the config to disk.
func Save(cfg *Config) error {
	dir := ConfigDir()
	if err := os.MkdirAll(dir, 0o755); err != nil {
		return err
	}
	data, err := yaml.Marshal(cfg)
	if err != nil {
		return err
	}
	return os.WriteFile(ConfigPath(), data, 0o644)
}

// Get returns a config value by key name.
func (c *Config) Get(key string) string {
	switch key {
	case "api_key":
		return c.APIKey
	case "model":
		return c.Model
	case "default_output_dir":
		return c.DefaultOutputDir
	case "aspect":
		return c.Aspect
	case "resolution":
		return c.Resolution
	case "quality":
		return c.Quality
	default:
		return ""
	}
}

// Set updates a config value by key name.
func (c *Config) Set(key, value string) bool {
	switch key {
	case "api_key":
		c.APIKey = value
	case "model":
		c.Model = value
	case "default_output_dir":
		c.DefaultOutputDir = value
	case "aspect":
		c.Aspect = value
	case "resolution":
		c.Resolution = value
	case "quality":
		c.Quality = value
	default:
		return false
	}
	return true
}

// ValidKeys returns the list of valid config keys.
func ValidKeys() []string {
	return []string{"api_key", "model", "default_output_dir", "aspect", "resolution", "quality"}
}

// ResolveModel returns the model id implied by config, applying the intra-config
// tiebreak: an explicit `model` key beats the `quality` alias (mirroring the flag
// precedence --model > --quality). Returns "" when neither is set (caller falls back to
// the built-in default). An invalid `quality` value yields an error.
func (c *Config) ResolveModel() (string, error) {
	if c.Model != "" {
		return c.Model, nil
	}
	if c.Quality != "" {
		return modelForQuality(c.Quality)
	}
	return "", nil
}

// modelForQuality mirrors gemini.ModelForQuality without importing gemini (config is a
// lower layer). Kept in lockstep with gemini's model constants.
func modelForQuality(quality string) (string, error) {
	switch quality {
	case "fast":
		return "gemini-3.1-flash-image", nil
	case "high":
		return "gemini-3-pro-image", nil
	default:
		return "", fmt.Errorf("invalid quality %q in config (valid: fast, high)", quality)
	}
}
