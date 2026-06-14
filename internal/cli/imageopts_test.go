package cli

import (
	"testing"

	"github.com/dixson3/naba/internal/config"
	"github.com/spf13/cobra"
)

// newImageCmd builds a throwaway command wired with the --model (persistent-style),
// --aspect, --resolution, and --quality flags, then parses argv so Changed() reflects it.
func newImageCmd(t *testing.T, argv ...string) *cobra.Command {
	t.Helper()
	flagModel, flagQuality, flagAspect, flagResolution = "", "", "", ""
	c := &cobra.Command{Use: "x", RunE: func(*cobra.Command, []string) error { return nil }}
	c.Flags().StringVarP(&flagModel, "model", "m", "", "")
	addImageConfigFlags(c)
	addQualityFlag(c)
	if err := c.ParseFlags(argv); err != nil {
		t.Fatal(err)
	}
	return c
}

func TestResolveModel_Precedence(t *testing.T) {
	tests := []struct {
		name string
		argv []string
		cfg  *config.Config
		want string
	}{
		{"flag model beats all", []string{"--model", "X", "--quality", "high"}, &config.Config{Model: "C"}, "X"},
		{"flag quality beats config", []string{"--quality", "high"}, &config.Config{Model: "C"}, "gemini-3-pro-image"},
		{"config model", nil, &config.Config{Model: "C"}, "C"},
		{"config quality", nil, &config.Config{Quality: "fast"}, "gemini-3.1-flash-image"},
		{"config model beats config quality", nil, &config.Config{Model: "C", Quality: "high"}, "C"},
		{"built-in default", nil, &config.Config{}, "gemini-3.1-flash-image"},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			c := newImageCmd(t, tt.argv...)
			got, err := resolveModel(c, tt.cfg)
			if err != nil {
				t.Fatal(err)
			}
			if got != tt.want {
				t.Errorf("resolveModel = %q, want %q", got, tt.want)
			}
		})
	}
}

func TestResolveImageConfig_FlagBeatsConfig(t *testing.T) {
	// Flag set -> flag wins over config.
	c := newImageCmd(t, "--aspect", "16:9")
	cfg := &config.Config{Aspect: "4:3", Resolution: "2K"}
	got, err := resolveImageConfig(c, cfg)
	if err != nil {
		t.Fatal(err)
	}
	if got == nil || got.AspectRatio != "16:9" {
		t.Errorf("flag aspect should win, got %+v", got)
	}
	if got.ImageSize != "2K" {
		t.Errorf("unset resolution flag should fall back to config 2K, got %q", got.ImageSize)
	}
}

func TestResolveImageConfig_ConfigDefault(t *testing.T) {
	c := newImageCmd(t) // no flags
	cfg := &config.Config{Aspect: "9:16"}
	got, err := resolveImageConfig(c, cfg)
	if err != nil {
		t.Fatal(err)
	}
	if got == nil || got.AspectRatio != "9:16" {
		t.Errorf("expected config aspect 9:16, got %+v", got)
	}
}

func TestResolveImageConfig_NilWhenUnset(t *testing.T) {
	c := newImageCmd(t)
	got, err := resolveImageConfig(c, &config.Config{})
	if err != nil {
		t.Fatal(err)
	}
	if got != nil {
		t.Errorf("expected nil imageConfig when nothing set, got %+v", got)
	}
}

func TestResolveImageConfig_InvalidConfigValue(t *testing.T) {
	c := newImageCmd(t)
	_, err := resolveImageConfig(c, &config.Config{Resolution: "1k"}) // lowercase invalid
	if err == nil {
		t.Error("expected error for invalid config resolution")
	}
}
