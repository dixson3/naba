package config

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestConfigDir_EnvOverride(t *testing.T) {
	t.Setenv("NABA_CONFIG_DIR", "/tmp/test-naba")
	if got := ConfigDir(); got != "/tmp/test-naba" {
		t.Errorf("ConfigDir() = %q, want %q", got, "/tmp/test-naba")
	}
}

func TestConfigDir_Default(t *testing.T) {
	t.Setenv("NABA_CONFIG_DIR", "")
	got := ConfigDir()
	if !strings.HasSuffix(got, filepath.Join(".config", "naba")) {
		t.Errorf("ConfigDir() = %q, want suffix %q", got, filepath.Join(".config", "naba"))
	}
}

func TestConfigPath(t *testing.T) {
	t.Setenv("NABA_CONFIG_DIR", "/tmp/test-naba")
	want := "/tmp/test-naba/config.yaml"
	if got := ConfigPath(); got != want {
		t.Errorf("ConfigPath() = %q, want %q", got, want)
	}
}

func TestLoad_MissingFile(t *testing.T) {
	t.Setenv("NABA_CONFIG_DIR", t.TempDir())
	cfg, err := Load()
	if err != nil {
		t.Fatalf("Load() error = %v, want nil", err)
	}
	if cfg.APIKey != "" || cfg.Model != "" || cfg.DefaultOutputDir != "" {
		t.Errorf("Load() returned non-zero config: %+v", cfg)
	}
}

func TestLoad_ValidFile(t *testing.T) {
	dir := t.TempDir()
	t.Setenv("NABA_CONFIG_DIR", dir)

	content := "api_key: test-key\nmodel: gemini-pro\ndefault_output_dir: /tmp/out\n"
	if err := os.WriteFile(filepath.Join(dir, "config.yaml"), []byte(content), 0o644); err != nil {
		t.Fatalf("failed to write test config: %v", err)
	}

	cfg, err := Load()
	if err != nil {
		t.Fatalf("Load() error = %v, want nil", err)
	}
	if cfg.APIKey != "test-key" {
		t.Errorf("APIKey = %q, want %q", cfg.APIKey, "test-key")
	}
	if cfg.Model != "gemini-pro" {
		t.Errorf("Model = %q, want %q", cfg.Model, "gemini-pro")
	}
	if cfg.DefaultOutputDir != "/tmp/out" {
		t.Errorf("DefaultOutputDir = %q, want %q", cfg.DefaultOutputDir, "/tmp/out")
	}
}

func TestLoad_MalformedYAML(t *testing.T) {
	dir := t.TempDir()
	t.Setenv("NABA_CONFIG_DIR", dir)

	if err := os.WriteFile(filepath.Join(dir, "config.yaml"), []byte(":::invalid"), 0o644); err != nil {
		t.Fatalf("failed to write test config: %v", err)
	}

	_, err := Load()
	if err == nil {
		t.Fatal("Load() error = nil, want non-nil for malformed YAML")
	}
}

func TestSave_RoundTrip(t *testing.T) {
	t.Setenv("NABA_CONFIG_DIR", t.TempDir())

	original := &Config{
		APIKey:           "round-trip-key",
		Model:            "gemini-2.0-flash",
		DefaultOutputDir: "/tmp/images",
	}
	if err := Save(original); err != nil {
		t.Fatalf("Save() error = %v", err)
	}

	loaded, err := Load()
	if err != nil {
		t.Fatalf("Load() error = %v", err)
	}
	if loaded.APIKey != original.APIKey {
		t.Errorf("APIKey = %q, want %q", loaded.APIKey, original.APIKey)
	}
	if loaded.Model != original.Model {
		t.Errorf("Model = %q, want %q", loaded.Model, original.Model)
	}
	if loaded.DefaultOutputDir != original.DefaultOutputDir {
		t.Errorf("DefaultOutputDir = %q, want %q", loaded.DefaultOutputDir, original.DefaultOutputDir)
	}
}

func TestSave_CreatesDirectory(t *testing.T) {
	base := t.TempDir()
	nested := filepath.Join(base, "deep", "nested", "config")
	t.Setenv("NABA_CONFIG_DIR", nested)

	cfg := &Config{APIKey: "test"}
	if err := Save(cfg); err != nil {
		t.Fatalf("Save() error = %v", err)
	}

	path := filepath.Join(nested, "config.yaml")
	if _, err := os.Stat(path); err != nil {
		t.Errorf("config file not found at %q: %v", path, err)
	}
}

func TestConfig_Get(t *testing.T) {
	cfg := Config{
		APIKey:           "my-key",
		Model:            "gemini-pro",
		DefaultOutputDir: "/out",
	}

	tests := []struct {
		key  string
		want string
	}{
		{"api_key", "my-key"},
		{"model", "gemini-pro"},
		{"default_output_dir", "/out"},
		{"unknown", ""},
	}

	for _, tt := range tests {
		t.Run(tt.key, func(t *testing.T) {
			if got := cfg.Get(tt.key); got != tt.want {
				t.Errorf("Get(%q) = %q, want %q", tt.key, got, tt.want)
			}
		})
	}
}

func TestConfig_Set(t *testing.T) {
	tests := []struct {
		key     string
		value   string
		wantOK  bool
		checkFn func(c *Config) string
	}{
		{"api_key", "new-key", true, func(c *Config) string { return c.APIKey }},
		{"model", "new-model", true, func(c *Config) string { return c.Model }},
		{"default_output_dir", "/new/dir", true, func(c *Config) string { return c.DefaultOutputDir }},
		{"unknown", "val", false, nil},
	}

	for _, tt := range tests {
		t.Run(tt.key, func(t *testing.T) {
			cfg := Config{}
			ok := cfg.Set(tt.key, tt.value)
			if ok != tt.wantOK {
				t.Errorf("Set(%q, %q) = %v, want %v", tt.key, tt.value, ok, tt.wantOK)
			}
			if tt.checkFn != nil {
				if got := tt.checkFn(&cfg); got != tt.value {
					t.Errorf("after Set(%q, %q), field = %q, want %q", tt.key, tt.value, got, tt.value)
				}
			}
		})
	}
}

func TestValidKeys(t *testing.T) {
	want := []string{"api_key", "model", "default_output_dir"}
	got := ValidKeys()
	if len(got) != len(want) {
		t.Fatalf("ValidKeys() returned %d keys, want %d", len(got), len(want))
	}
	for i, k := range want {
		if got[i] != k {
			t.Errorf("ValidKeys()[%d] = %q, want %q", i, got[i], k)
		}
	}
}

func TestResolveAPIKey_EnvVar(t *testing.T) {
	t.Setenv("NABA_CONFIG_DIR", t.TempDir())
	t.Setenv("GEMINI_API_KEY", "env-key")

	if got := ResolveAPIKey(); got != "env-key" {
		t.Errorf("ResolveAPIKey() = %q, want %q", got, "env-key")
	}
}

func TestResolveAPIKey_ConfigFallback(t *testing.T) {
	t.Setenv("NABA_CONFIG_DIR", t.TempDir())
	t.Setenv("GEMINI_API_KEY", "")

	cfg := &Config{APIKey: "config-key"}
	if err := Save(cfg); err != nil {
		t.Fatalf("Save() error = %v", err)
	}

	if got := ResolveAPIKey(); got != "config-key" {
		t.Errorf("ResolveAPIKey() = %q, want %q", got, "config-key")
	}
}

func TestResolveAPIKey_EnvTakesPrecedence(t *testing.T) {
	t.Setenv("NABA_CONFIG_DIR", t.TempDir())
	t.Setenv("GEMINI_API_KEY", "env-key")

	cfg := &Config{APIKey: "config-key"}
	if err := Save(cfg); err != nil {
		t.Fatalf("Save() error = %v", err)
	}

	if got := ResolveAPIKey(); got != "env-key" {
		t.Errorf("ResolveAPIKey() = %q, want %q (env should take precedence)", got, "env-key")
	}
}

func TestResolveAPIKey_Neither(t *testing.T) {
	t.Setenv("NABA_CONFIG_DIR", t.TempDir())
	t.Setenv("GEMINI_API_KEY", "")

	if got := ResolveAPIKey(); got != "" {
		t.Errorf("ResolveAPIKey() = %q, want empty string", got)
	}
}

func TestResolveAPIKey_BrokenConfig(t *testing.T) {
	dir := t.TempDir()
	t.Setenv("NABA_CONFIG_DIR", dir)
	t.Setenv("GEMINI_API_KEY", "")

	if err := os.WriteFile(filepath.Join(dir, "config.yaml"), []byte(":::invalid"), 0o644); err != nil {
		t.Fatalf("failed to write test config: %v", err)
	}

	if got := ResolveAPIKey(); got != "" {
		t.Errorf("ResolveAPIKey() = %q, want empty string for broken config", got)
	}
}
