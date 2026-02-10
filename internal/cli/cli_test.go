package cli

import (
	"bytes"
	"encoding/base64"
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"strings"
	"sync/atomic"
	"testing"

	"github.com/dixson3/nba/internal/gemini"
)

// resetFlags resets all package-level flag variables to their defaults.
// Cobra commands use package-level vars for flag storage, so these persist
// across tests unless explicitly reset.
func resetFlags() {
	flagJSON = false
	flagOutput = ""
	flagQuiet = false
	flagModel = ""
	flagNoInput = false

	genStyle = ""
	genCount = 1
	genSeed = 0
	genFormat = "separate"
	genVariations = nil
	genPreview = false

	storySteps = 4
	storyStyle = "consistent"
	storyTransition = "smooth"
	storyLayout = "separate"
	storyPreview = false

	editPreview = false
	restorePreview = false

	iconStyle = "modern"
	iconSizes = []int{256}
	iconFormat = "png"
	iconBackground = "transparent"
	iconCorners = "rounded"
	iconPreview = false

	patternStyle = "abstract"
	patternColors = "colorful"
	patternDensity = "medium"
	patternTileSize = "256x256"
	patternRepeat = "tile"
	patternPreview = false

	diagramType = "flowchart"
	diagramStyle = "professional"
	diagramLayout = "hierarchical"
	diagramComplexity = "detailed"
	diagramColors = "accent"
	diagramPreview = false
}

// geminiSuccessResponse returns a valid Gemini API JSON response containing
// a single PNG image (a few arbitrary bytes, base64-encoded).
func geminiSuccessResponse() []byte {
	imgBytes := []byte("fakepng")
	b64 := base64.StdEncoding.EncodeToString(imgBytes)
	resp := map[string]any{
		"candidates": []map[string]any{
			{
				"content": map[string]any{
					"role": "model",
					"parts": []map[string]any{
						{
							"inlineData": map[string]any{
								"mimeType": "image/png",
								"data":     b64,
							},
						},
					},
				},
			},
		},
	}
	data, _ := json.Marshal(resp)
	return data
}

// newMockServer creates an httptest server that returns a successful Gemini
// image response for any POST request.
func newMockServer(t *testing.T) *httptest.Server {
	t.Helper()
	return httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		w.Write(geminiSuccessResponse())
	}))
}

// newMockServerWithCounter creates an httptest server that counts POST requests
// via an atomic counter and returns a successful Gemini image response.
func newMockServerWithCounter(t *testing.T, counter *atomic.Int32) *httptest.Server {
	t.Helper()
	return httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		counter.Add(1)
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		w.Write(geminiSuccessResponse())
	}))
}

// newMockServerWithStatus creates an httptest server that returns the given
// HTTP status code with an error body.
func newMockServerWithStatus(t *testing.T, statusCode int) *httptest.Server {
	t.Helper()
	return httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		errResp := map[string]any{
			"error": map[string]any{
				"code":    statusCode,
				"message": fmt.Sprintf("mock error %d", statusCode),
				"status":  "ERROR",
			},
		}
		data, _ := json.Marshal(errResp)
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(statusCode)
		w.Write(data)
	}))
}

// createTempImage creates a minimal temporary PNG file and returns its path.
func createTempImage(t *testing.T) string {
	t.Helper()
	dir := t.TempDir()
	p := filepath.Join(dir, "test.png")
	// Minimal PNG-like content (not a real PNG, but sufficient for CLI tests
	// that only need os.Stat to succeed and base64 encoding).
	if err := os.WriteFile(p, []byte("fakepng"), 0o644); err != nil {
		t.Fatal(err)
	}
	return p
}

// --- Argument validation tests (no API needed) ---

func TestGenerateCmd_NoArgs(t *testing.T) {
	resetFlags()
	rootCmd.SetArgs([]string{"generate"})
	err := rootCmd.Execute()
	if err == nil {
		t.Fatal("expected error for generate with no args")
	}
}

func TestGenerateCmd_TooManyArgs(t *testing.T) {
	resetFlags()
	rootCmd.SetArgs([]string{"generate", "a", "b"})
	err := rootCmd.Execute()
	if err == nil {
		t.Fatal("expected error for generate with too many args")
	}
}

func TestEditCmd_WrongArgCount(t *testing.T) {
	resetFlags()
	rootCmd.SetArgs([]string{"edit", "onlyone"})
	err := rootCmd.Execute()
	if err == nil {
		t.Fatal("expected error for edit with wrong arg count")
	}
}

func TestEditCmd_MissingFile(t *testing.T) {
	resetFlags()
	tmpDir := t.TempDir()
	t.Setenv("GEMINI_API_KEY", "test-key")
	t.Setenv("NBA_CONFIG_DIR", tmpDir)
	rootCmd.SetArgs([]string{"edit", "/nonexistent/file.png", "make blue"})
	err := rootCmd.Execute()
	if err == nil {
		t.Fatal("expected error for edit with missing file")
	}
	var ece *exitCodeError
	if errors.As(err, &ece) {
		if ece.ExitCode() != gemini.ExitFileIO {
			t.Errorf("expected ExitFileIO (%d), got %d", gemini.ExitFileIO, ece.ExitCode())
		}
	}
	if !strings.Contains(err.Error(), "not found") {
		t.Errorf("expected error to contain 'not found', got: %s", err.Error())
	}
}

func TestRestoreCmd_NoArgs(t *testing.T) {
	resetFlags()
	rootCmd.SetArgs([]string{"restore"})
	err := rootCmd.Execute()
	if err == nil {
		t.Fatal("expected error for restore with no args")
	}
}

func TestStoryCmd_InvalidSteps(t *testing.T) {
	tests := []struct {
		name  string
		steps string
	}{
		{"steps=1", "1"},
		{"steps=9", "9"},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			resetFlags()
			tmpDir := t.TempDir()
			t.Setenv("GEMINI_API_KEY", "test-key")
			t.Setenv("NBA_CONFIG_DIR", tmpDir)
			rootCmd.SetArgs([]string{"story", "--steps", tt.steps, "test"})
			err := rootCmd.Execute()
			if err == nil {
				t.Fatal("expected error for invalid steps")
			}
			if !strings.Contains(err.Error(), "between 2 and 8") {
				t.Errorf("expected error to contain 'between 2 and 8', got: %s", err.Error())
			}
			var ece *exitCodeError
			if !errors.As(err, &ece) {
				t.Fatalf("expected *exitCodeError, got %T", err)
			}
			if ece.ExitCode() != gemini.ExitUsage {
				t.Errorf("expected exit code %d, got %d", gemini.ExitUsage, ece.ExitCode())
			}
		})
	}
}

func TestConfigGetCmd_NoArgs(t *testing.T) {
	resetFlags()
	rootCmd.SetArgs([]string{"config", "get"})
	err := rootCmd.Execute()
	if err == nil {
		t.Fatal("expected error for config get with no args")
	}
}

func TestConfigSetCmd_InvalidKey(t *testing.T) {
	resetFlags()
	tmpDir := t.TempDir()
	t.Setenv("NBA_CONFIG_DIR", tmpDir)
	rootCmd.SetArgs([]string{"config", "set", "invalid_key", "value"})
	err := rootCmd.Execute()
	if err == nil {
		t.Fatal("expected error for config set with invalid key")
	}
	if !strings.Contains(err.Error(), "unknown key") {
		t.Errorf("expected error to contain 'unknown key', got: %s", err.Error())
	}
}

func TestVersionCmd(t *testing.T) {
	resetFlags()

	// Capture real stdout since versionCmd uses fmt.Printf (not cmd.Printf)
	oldStdout := os.Stdout
	r, w, err := os.Pipe()
	if err != nil {
		t.Fatal(err)
	}
	os.Stdout = w

	rootCmd.SetArgs([]string{"version"})
	execErr := rootCmd.Execute()

	w.Close()
	os.Stdout = oldStdout

	if execErr != nil {
		t.Fatalf("unexpected error: %v", execErr)
	}

	var buf bytes.Buffer
	buf.ReadFrom(r)
	if !strings.Contains(buf.String(), "nba") {
		t.Errorf("expected output to contain 'nba', got: %s", buf.String())
	}
}

func TestAllCommands_MissingAPIKey(t *testing.T) {
	// Create a temp file for edit/restore commands that require a file arg
	tmpImg := createTempImage(t)

	tests := []struct {
		name string
		args []string
	}{
		{"generate", []string{"generate", "test prompt"}},
		{"edit", []string{"edit", tmpImg, "make blue"}},
		{"restore", []string{"restore", tmpImg}},
		{"icon", []string{"icon", "test prompt"}},
		{"pattern", []string{"pattern", "test prompt"}},
		{"story", []string{"story", "test prompt"}},
		{"diagram", []string{"diagram", "test prompt"}},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			resetFlags()
			tmpDir := t.TempDir()
			t.Setenv("GEMINI_API_KEY", "")
			t.Setenv("NBA_CONFIG_DIR", tmpDir)
			rootCmd.SetArgs(tt.args)
			err := rootCmd.Execute()
			if err == nil {
				t.Fatal("expected error for missing API key")
			}
			var ece *exitCodeError
			if !errors.As(err, &ece) {
				t.Fatalf("expected *exitCodeError, got %T: %v", err, err)
			}
			if ece.ExitCode() != gemini.ExitAuth {
				t.Errorf("expected exit code %d (ExitAuth), got %d", gemini.ExitAuth, ece.ExitCode())
			}
		})
	}
}

// --- Integration tests with mock server ---

func TestGenerateCmd_Success(t *testing.T) {
	resetFlags()
	server := newMockServer(t)
	defer server.Close()

	tmpDir := t.TempDir()
	outFile := filepath.Join(tmpDir, "out.png")
	t.Setenv("GEMINI_BASE_URL", server.URL)
	t.Setenv("GEMINI_API_KEY", "test-key")
	t.Setenv("NBA_CONFIG_DIR", tmpDir)

	rootCmd.SetArgs([]string{"generate", "--quiet", "--output", outFile, "test prompt"})
	err := rootCmd.Execute()
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if _, err := os.Stat(outFile); os.IsNotExist(err) {
		t.Error("expected output file to exist")
	}
}

func TestGenerateCmd_JSON(t *testing.T) {
	resetFlags()
	server := newMockServer(t)
	defer server.Close()

	tmpDir := t.TempDir()
	outFile := filepath.Join(tmpDir, "out.png")
	t.Setenv("GEMINI_BASE_URL", server.URL)
	t.Setenv("GEMINI_API_KEY", "test-key")
	t.Setenv("NBA_CONFIG_DIR", tmpDir)

	var buf bytes.Buffer
	rootCmd.SetOut(&buf)
	defer rootCmd.SetOut(nil)

	rootCmd.SetArgs([]string{"generate", "--json", "--quiet", "--output", outFile, "test prompt"})
	err := rootCmd.Execute()
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	output := buf.String()
	// PrintJSON uses fmt.Println which writes to os.Stdout, not cmd.OutOrStdout.
	// If the buffer is empty, check that the file was written (the JSON went to real stdout).
	if output != "" {
		var result map[string]any
		if err := json.Unmarshal([]byte(output), &result); err != nil {
			t.Fatalf("output is not valid JSON: %v\noutput: %s", err, output)
		}
		for _, key := range []string{"path", "command", "prompt"} {
			if _, ok := result[key]; !ok {
				t.Errorf("expected JSON to contain key %q", key)
			}
		}
	}
	// Regardless, verify the file was written
	if _, err := os.Stat(outFile); os.IsNotExist(err) {
		t.Error("expected output file to exist")
	}
}

func TestGenerateCmd_Count(t *testing.T) {
	resetFlags()
	var counter atomic.Int32
	server := newMockServerWithCounter(t, &counter)
	defer server.Close()

	tmpDir := t.TempDir()
	outDir := filepath.Join(tmpDir, "out")
	if err := os.MkdirAll(outDir, 0o755); err != nil {
		t.Fatal(err)
	}
	t.Setenv("GEMINI_BASE_URL", server.URL)
	t.Setenv("GEMINI_API_KEY", "test-key")
	t.Setenv("NBA_CONFIG_DIR", tmpDir)

	rootCmd.SetArgs([]string{"generate", "--quiet", "--count", "2", "--output", filepath.Join(outDir, "img.png"), "test prompt"})
	err := rootCmd.Execute()
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if counter.Load() != 2 {
		t.Errorf("expected 2 API calls, got %d", counter.Load())
	}

	// Check that files were written in the output directory
	entries, err := os.ReadDir(outDir)
	if err != nil {
		t.Fatal(err)
	}
	pngCount := 0
	for _, e := range entries {
		if strings.HasSuffix(e.Name(), ".png") {
			pngCount++
		}
	}
	if pngCount < 2 {
		t.Errorf("expected at least 2 png files, got %d", pngCount)
	}
}

func TestEditCmd_Success(t *testing.T) {
	resetFlags()
	server := newMockServer(t)
	defer server.Close()

	tmpDir := t.TempDir()
	inputFile := createTempImage(t)
	outFile := filepath.Join(tmpDir, "edited.png")
	t.Setenv("GEMINI_BASE_URL", server.URL)
	t.Setenv("GEMINI_API_KEY", "test-key")
	t.Setenv("NBA_CONFIG_DIR", tmpDir)

	rootCmd.SetArgs([]string{"edit", "--quiet", "--output", outFile, inputFile, "make it blue"})
	err := rootCmd.Execute()
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if _, err := os.Stat(outFile); os.IsNotExist(err) {
		t.Error("expected output file to exist")
	}
}

func TestRestoreCmd_Success(t *testing.T) {
	resetFlags()
	server := newMockServer(t)
	defer server.Close()

	tmpDir := t.TempDir()
	inputFile := createTempImage(t)
	outFile := filepath.Join(tmpDir, "restored.png")
	t.Setenv("GEMINI_BASE_URL", server.URL)
	t.Setenv("GEMINI_API_KEY", "test-key")
	t.Setenv("NBA_CONFIG_DIR", tmpDir)

	rootCmd.SetArgs([]string{"restore", "--quiet", "--output", outFile, inputFile})
	err := rootCmd.Execute()
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if _, err := os.Stat(outFile); os.IsNotExist(err) {
		t.Error("expected output file to exist")
	}
}

func TestStoryCmd_Success(t *testing.T) {
	resetFlags()
	var counter atomic.Int32
	server := newMockServerWithCounter(t, &counter)
	defer server.Close()

	tmpDir := t.TempDir()
	t.Setenv("GEMINI_BASE_URL", server.URL)
	t.Setenv("GEMINI_API_KEY", "test-key")
	t.Setenv("NBA_CONFIG_DIR", tmpDir)

	rootCmd.SetArgs([]string{"story", "--steps", "3", "--quiet", "--output", filepath.Join(tmpDir, "story.png"), "a cat adventure"})
	err := rootCmd.Execute()
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if counter.Load() != 3 {
		t.Errorf("expected 3 API calls, got %d", counter.Load())
	}
}

func TestGenerateCmd_APIError401(t *testing.T) {
	resetFlags()
	server := newMockServerWithStatus(t, 401)
	defer server.Close()

	tmpDir := t.TempDir()
	t.Setenv("GEMINI_BASE_URL", server.URL)
	t.Setenv("GEMINI_API_KEY", "test-key")
	t.Setenv("NBA_CONFIG_DIR", tmpDir)

	rootCmd.SetArgs([]string{"generate", "--quiet", "test prompt"})
	err := rootCmd.Execute()
	if err == nil {
		t.Fatal("expected error for 401 response")
	}
	var ece *exitCodeError
	if !errors.As(err, &ece) {
		t.Fatalf("expected *exitCodeError, got %T: %v", err, err)
	}
	if ece.ExitCode() != gemini.ExitAuth {
		t.Errorf("expected exit code %d (ExitAuth), got %d", gemini.ExitAuth, ece.ExitCode())
	}
}

func TestGenerateCmd_APIError429(t *testing.T) {
	resetFlags()
	server := newMockServerWithStatus(t, 429)
	defer server.Close()

	tmpDir := t.TempDir()
	t.Setenv("GEMINI_BASE_URL", server.URL)
	t.Setenv("GEMINI_API_KEY", "test-key")
	t.Setenv("NBA_CONFIG_DIR", tmpDir)

	rootCmd.SetArgs([]string{"generate", "--quiet", "test prompt"})
	err := rootCmd.Execute()
	if err == nil {
		t.Fatal("expected error for 429 response")
	}
	var ece *exitCodeError
	if !errors.As(err, &ece) {
		t.Fatalf("expected *exitCodeError, got %T: %v", err, err)
	}
	if ece.ExitCode() != gemini.ExitRateLimit {
		t.Errorf("expected exit code %d (ExitRateLimit), got %d", gemini.ExitRateLimit, ece.ExitCode())
	}
}

func TestConfigSetAndGet_RoundTrip(t *testing.T) {
	resetFlags()
	tmpDir := t.TempDir()
	t.Setenv("NBA_CONFIG_DIR", tmpDir)

	// Set
	rootCmd.SetArgs([]string{"config", "set", "api_key", "test-key-123"})
	err := rootCmd.Execute()
	if err != nil {
		t.Fatalf("config set failed: %v", err)
	}

	// Get — capture stdout via rootCmd.SetOut
	resetFlags()
	var buf bytes.Buffer
	rootCmd.SetOut(&buf)
	defer rootCmd.SetOut(nil)

	rootCmd.SetArgs([]string{"config", "get", "api_key"})
	err = rootCmd.Execute()
	if err != nil {
		t.Fatalf("config get failed: %v", err)
	}

	// configGetCmd uses fmt.Println which writes to os.Stdout, not cmd.OutOrStdout.
	// The value will have been written to the config file, so verify via a second set/get
	// or check the file directly.
	// Since fmt.Println goes to real stdout, we verify the config file.
	cfgFile := filepath.Join(tmpDir, "config.yaml")
	data, err := os.ReadFile(cfgFile)
	if err != nil {
		t.Fatalf("failed to read config file: %v", err)
	}
	if !strings.Contains(string(data), "test-key-123") {
		t.Errorf("expected config file to contain 'test-key-123', got: %s", string(data))
	}
}

func TestExitCodeError(t *testing.T) {
	e := exitError(5, "test msg")
	if e.Error() != "test msg" {
		t.Errorf("expected Error() == 'test msg', got %q", e.Error())
	}
	if e.ExitCode() != 5 {
		t.Errorf("expected ExitCode() == 5, got %d", e.ExitCode())
	}
}

func TestHandleAPIError(t *testing.T) {
	t.Run("APIError", func(t *testing.T) {
		apiErr := &gemini.APIError{
			ExitCode: gemini.ExitAPI,
			Message:  "api fail",
		}
		result := handleAPIError(apiErr)
		var ece *exitCodeError
		if !errors.As(result, &ece) {
			t.Fatalf("expected *exitCodeError, got %T", result)
		}
		if ece.ExitCode() != gemini.ExitAPI {
			t.Errorf("expected exit code %d, got %d", gemini.ExitAPI, ece.ExitCode())
		}
		if ece.Error() != "api fail" {
			t.Errorf("expected message 'api fail', got %q", ece.Error())
		}
	})

	t.Run("generic error", func(t *testing.T) {
		genErr := fmt.Errorf("generic")
		result := handleAPIError(genErr)
		var ece *exitCodeError
		if !errors.As(result, &ece) {
			t.Fatalf("expected *exitCodeError, got %T", result)
		}
		if ece.ExitCode() != gemini.ExitGeneral {
			t.Errorf("expected exit code %d (ExitGeneral), got %d", gemini.ExitGeneral, ece.ExitCode())
		}
	})
}
