package gemini

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
)

// TestGenerationConfig_OmitEmptyImageConfig asserts that a bare request (no imageConfig)
// marshals byte-identically to the pre-imageConfig shape — no imageConfig key at all.
func TestGenerationConfig_OmitEmptyImageConfig(t *testing.T) {
	req := GenerateRequest{
		Contents:         []Content{{Role: "user", Parts: []Part{{Text: "x"}}}},
		GenerationConfig: GenerationConfig{ResponseModalities: []string{"TEXT", "IMAGE"}},
	}
	data, err := json.Marshal(req)
	if err != nil {
		t.Fatal(err)
	}
	if strings.Contains(string(data), "imageConfig") {
		t.Errorf("bare request should omit imageConfig, got: %s", data)
	}
}

func TestGenerationConfig_WithImageConfig(t *testing.T) {
	req := GenerateRequest{
		Contents: []Content{{Role: "user", Parts: []Part{{Text: "x"}}}},
		GenerationConfig: GenerationConfig{
			ResponseModalities: []string{"TEXT", "IMAGE"},
			ImageConfig:        &ImageConfig{AspectRatio: "16:9", ImageSize: "2K"},
		},
	}
	data, err := json.Marshal(req)
	if err != nil {
		t.Fatal(err)
	}
	s := string(data)
	for _, want := range []string{`"imageConfig"`, `"aspectRatio":"16:9"`, `"imageSize":"2K"`} {
		if !strings.Contains(s, want) {
			t.Errorf("expected %s in %s", want, s)
		}
	}
}

func TestNewImageConfig(t *testing.T) {
	tests := []struct {
		name       string
		aspect     string
		resolution string
		wantNil    bool
		wantErr    bool
	}{
		{"both empty -> nil", "", "", true, false},
		{"valid aspect only", "16:9", "", false, false},
		{"valid resolution only", "", "2K", false, false},
		{"both valid", "1:1", "512", false, false},
		{"invalid aspect", "5:7", "", false, true},
		{"invalid resolution lowercase k", "", "1k", false, true},
		{"valid aspect invalid resolution", "16:9", "8K", false, true},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			cfg, err := NewImageConfig(tt.aspect, tt.resolution)
			if (err != nil) != tt.wantErr {
				t.Fatalf("err = %v, wantErr %v", err, tt.wantErr)
			}
			if tt.wantErr {
				if apiErr, ok := err.(*APIError); !ok || apiErr.ExitCode != ExitUsage {
					t.Errorf("expected *APIError with ExitUsage, got %v", err)
				}
				return
			}
			if (cfg == nil) != tt.wantNil {
				t.Errorf("cfg nil = %v, want %v", cfg == nil, tt.wantNil)
			}
		})
	}
}

func TestModelForQuality(t *testing.T) {
	tests := []struct {
		quality string
		want    string
		wantErr bool
	}{
		{"fast", "gemini-3.1-flash-image", false},
		{"high", "gemini-3-pro-image", false},
		{"ultra", "", true},
		{"", "", true},
	}
	for _, tt := range tests {
		got, err := ModelForQuality(tt.quality)
		if (err != nil) != tt.wantErr {
			t.Errorf("ModelForQuality(%q) err = %v, wantErr %v", tt.quality, err, tt.wantErr)
		}
		if got != tt.want {
			t.Errorf("ModelForQuality(%q) = %q, want %q", tt.quality, got, tt.want)
		}
	}
}

func TestModelReachable(t *testing.T) {
	available := []string{"models/gemini-3.1-flash-image", "models/gemini-3-pro-image"}
	if !ModelReachable("gemini-3.1-flash-image", available) {
		t.Error("expected gemini-3.1-flash-image reachable")
	}
	if !ModelReachable("models/gemini-3-pro-image", available) {
		t.Error("expected prefixed id reachable")
	}
	if ModelReachable("gemini-2.0-flash-exp-image-generation", available) {
		t.Error("retired model should not be reachable")
	}
}

func TestListModels(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Header.Get("x-goog-api-key") != "test-key" {
			t.Errorf("missing api key header")
		}
		_, _ = w.Write([]byte(`{"models":[{"name":"models/gemini-3.1-flash-image"},{"name":"models/gemini-3-pro-image"}]}`))
	}))
	defer srv.Close()

	c := NewClient("test-key", "")
	c.BaseURL = srv.URL
	names, err := c.ListModels()
	if err != nil {
		t.Fatal(err)
	}
	if len(names) != 2 || names[0] != "gemini-3.1-flash-image" {
		t.Errorf("unexpected names: %v", names)
	}
}

func TestListModels_AuthError(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusUnauthorized)
		_, _ = w.Write([]byte(`{"error":{"code":401,"message":"bad key"}}`))
	}))
	defer srv.Close()

	c := NewClient("bad", "")
	c.BaseURL = srv.URL
	_, err := c.ListModels()
	apiErr, ok := err.(*APIError)
	if !ok || apiErr.ExitCode != ExitAuth {
		t.Fatalf("expected auth APIError, got %v", err)
	}
}
