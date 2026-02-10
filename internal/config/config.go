package config

import (
	"os"
	"path/filepath"

	"gopkg.in/yaml.v3"
)

const configFileName = "config.yaml"

// Config holds the nba configuration.
type Config struct {
	APIKey           string `yaml:"api_key,omitempty"`
	Model            string `yaml:"model,omitempty"`
	DefaultOutputDir string `yaml:"default_output_dir,omitempty"`
}

// ConfigDir returns the configuration directory path.
func ConfigDir() string {
	if dir := os.Getenv("NBA_CONFIG_DIR"); dir != "" {
		return dir
	}
	home, _ := os.UserHomeDir()
	return filepath.Join(home, ".config", "nba")
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
	default:
		return false
	}
	return true
}

// ValidKeys returns the list of valid config keys.
func ValidKeys() []string {
	return []string{"api_key", "model", "default_output_dir"}
}
