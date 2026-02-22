package mcp

import (
	"context"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"strings"
	"sync/atomic"
	"testing"

	mcpsdk "github.com/mark3labs/mcp-go/mcp"
)

// geminiResponse builds a JSON response with a base64 PNG image.
func geminiResponse(t *testing.T) []byte {
	t.Helper()
	// 1x1 red PNG
	pngData := []byte{
		0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d,
		0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
		0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53, 0xde, 0x00, 0x00, 0x00,
		0x0c, 0x49, 0x44, 0x41, 0x54, 0x08, 0xd7, 0x63, 0xf8, 0xcf, 0xc0, 0x00,
		0x00, 0x00, 0x03, 0x00, 0x01, 0x36, 0x28, 0x19, 0x00, 0x00, 0x00, 0x00,
		0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
	}
	encoded := base64.StdEncoding.EncodeToString(pngData)
	resp := map[string]any{
		"candidates": []map[string]any{
			{
				"content": map[string]any{
					"role": "model",
					"parts": []map[string]any{
						{"inlineData": map[string]any{
							"mimeType": "image/png",
							"data":     encoded,
						}},
					},
				},
			},
		},
	}
	data, err := json.Marshal(resp)
	if err != nil {
		t.Fatal(err)
	}
	return data
}

// newMockGeminiServer creates a test server that returns a valid image response.
func newMockGeminiServer(t *testing.T) *httptest.Server {
	t.Helper()
	return httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.Write(geminiResponse(t))
	}))
}

// newCountingServer creates a server that counts requests.
func newCountingServer(t *testing.T) (*httptest.Server, *atomic.Int32) {
	t.Helper()
	var count atomic.Int32
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		count.Add(1)
		w.Header().Set("Content-Type", "application/json")
		w.Write(geminiResponse(t))
	}))
	return srv, &count
}

// makeRequest builds a CallToolRequest with the given arguments.
func makeRequest(args map[string]any) mcpsdk.CallToolRequest {
	return mcpsdk.CallToolRequest{
		Params: mcpsdk.CallToolParams{
			Arguments: args,
		},
	}
}

// --- Arg Validation Tests ---

func TestGenerateImage_MissingPrompt(t *testing.T) {
	result, err := handleGenerateImage(context.Background(), makeRequest(map[string]any{}))
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if !result.IsError {
		t.Fatal("expected error result")
	}
	text := contentText(t, result)
	if !strings.Contains(text, "prompt") {
		t.Fatalf("expected error about prompt, got: %s", text)
	}
}

func TestEditImage_MissingPrompt(t *testing.T) {
	result, err := handleEditImage(context.Background(), makeRequest(map[string]any{
		"file": "/tmp/test.png",
	}))
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if !result.IsError {
		t.Fatal("expected error result")
	}
	text := contentText(t, result)
	if !strings.Contains(text, "prompt") {
		t.Fatalf("expected error about prompt, got: %s", text)
	}
}

func TestEditImage_MissingFile(t *testing.T) {
	result, err := handleEditImage(context.Background(), makeRequest(map[string]any{
		"prompt": "make it blue",
	}))
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if !result.IsError {
		t.Fatal("expected error result")
	}
	text := contentText(t, result)
	if !strings.Contains(text, "file") {
		t.Fatalf("expected error about file, got: %s", text)
	}
}

func TestEditImage_NonexistentFile(t *testing.T) {
	t.Setenv("GEMINI_API_KEY", "test-key")
	result, err := handleEditImage(context.Background(), makeRequest(map[string]any{
		"prompt": "make it blue",
		"file":   "/nonexistent/path/image.png",
	}))
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if !result.IsError {
		t.Fatal("expected error result")
	}
	text := contentText(t, result)
	if !strings.Contains(text, "not found") {
		t.Fatalf("expected file not found error, got: %s", text)
	}
}

func TestRestoreImage_MissingFile(t *testing.T) {
	result, err := handleRestoreImage(context.Background(), makeRequest(map[string]any{}))
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if !result.IsError {
		t.Fatal("expected error result")
	}
	text := contentText(t, result)
	if !strings.Contains(text, "file") {
		t.Fatalf("expected error about file, got: %s", text)
	}
}

func TestGenerateStory_InvalidSteps(t *testing.T) {
	t.Setenv("GEMINI_API_KEY", "test-key")
	for _, steps := range []int{0, 1, 9, 100} {
		t.Run(fmt.Sprintf("steps=%d", steps), func(t *testing.T) {
			result, err := handleGenerateStory(context.Background(), makeRequest(map[string]any{
				"prompt": "a story",
				"steps":  float64(steps), // JSON numbers are float64
			}))
			if err != nil {
				t.Fatalf("unexpected error: %v", err)
			}
			if !result.IsError {
				t.Fatalf("expected error result for steps=%d", steps)
			}
			text := contentText(t, result)
			if !strings.Contains(text, "steps") {
				t.Fatalf("expected error about steps, got: %s", text)
			}
		})
	}
}

func TestGenerateImage_InvalidCount(t *testing.T) {
	t.Setenv("GEMINI_API_KEY", "test-key")
	for _, count := range []int{0, -1, 9, 100} {
		t.Run(fmt.Sprintf("count=%d", count), func(t *testing.T) {
			result, err := handleGenerateImage(context.Background(), makeRequest(map[string]any{
				"prompt": "test",
				"count":  float64(count),
			}))
			if err != nil {
				t.Fatalf("unexpected error: %v", err)
			}
			if !result.IsError {
				t.Fatalf("expected error result for count=%d", count)
			}
			text := contentText(t, result)
			if !strings.Contains(text, "count") {
				t.Fatalf("expected error about count, got: %s", text)
			}
		})
	}
}

// --- Auth Error Tests ---

func TestAllHandlers_NoAPIKey(t *testing.T) {
	t.Setenv("GEMINI_API_KEY", "")
	t.Setenv("NABA_CONFIG_DIR", t.TempDir())

	// Create a real temp file so edit/restore don't fail on file-not-found first
	tmpFile := filepath.Join(t.TempDir(), "test.png")
	os.WriteFile(tmpFile, []byte("fake"), 0o644)

	handlers := []struct {
		name    string
		handler func(context.Context, mcpsdk.CallToolRequest) (*mcpsdk.CallToolResult, error)
		args    map[string]any
	}{
		{"generate_image", handleGenerateImage, map[string]any{"prompt": "test"}},
		{"edit_image", handleEditImage, map[string]any{"prompt": "test", "file": tmpFile}},
		{"restore_image", handleRestoreImage, map[string]any{"file": tmpFile}},
		{"generate_icon", handleGenerateIcon, map[string]any{"prompt": "test"}},
		{"generate_pattern", handleGeneratePattern, map[string]any{"prompt": "test"}},
		{"generate_story", handleGenerateStory, map[string]any{"prompt": "test", "steps": float64(2)}},
		{"generate_diagram", handleGenerateDiagram, map[string]any{"prompt": "test"}},
	}

	for _, h := range handlers {
		t.Run(h.name, func(t *testing.T) {
			result, err := h.handler(context.Background(), makeRequest(h.args))
			if err != nil {
				t.Fatalf("unexpected error: %v", err)
			}
			if !result.IsError {
				t.Fatal("expected error result for missing API key")
			}
			text := contentText(t, result)
			if !strings.Contains(text, "GEMINI_API_KEY") {
				t.Fatalf("expected API key error, got: %s", text)
			}
		})
	}
}

// --- Success Path Tests ---

func TestGenerateImage_Success(t *testing.T) {
	srv := newMockGeminiServer(t)
	defer srv.Close()

	tmpDir := t.TempDir()
	origDir, _ := os.Getwd()
	os.Chdir(tmpDir)
	defer os.Chdir(origDir)

	t.Setenv("GEMINI_API_KEY", "test-key")
	t.Setenv("GEMINI_BASE_URL", srv.URL)
	t.Setenv("NABA_CONFIG_DIR", t.TempDir())

	result, err := handleGenerateImage(context.Background(), makeRequest(map[string]any{
		"prompt": "a red apple",
	}))
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if result.IsError {
		t.Fatalf("unexpected error result: %s", contentText(t, result))
	}

	// Should have text content (path) and image content
	if len(result.Content) < 2 {
		t.Fatalf("expected at least 2 content items, got %d", len(result.Content))
	}

	// First content should be text with a file path
	text := contentText(t, result)
	if !strings.Contains(text, "naba-generate") {
		t.Fatalf("expected path containing naba-generate, got: %s", text)
	}

	// Verify file exists
	if _, err := os.Stat(text); err != nil {
		t.Fatalf("output file does not exist: %s", text)
	}
}

func TestEditImage_Success(t *testing.T) {
	srv := newMockGeminiServer(t)
	defer srv.Close()

	tmpDir := t.TempDir()
	origDir, _ := os.Getwd()
	os.Chdir(tmpDir)
	defer os.Chdir(origDir)

	t.Setenv("GEMINI_API_KEY", "test-key")
	t.Setenv("GEMINI_BASE_URL", srv.URL)
	t.Setenv("NABA_CONFIG_DIR", t.TempDir())

	// Create a test input image
	inputFile := filepath.Join(tmpDir, "input.png")
	os.WriteFile(inputFile, []byte("fake png"), 0o644)

	result, err := handleEditImage(context.Background(), makeRequest(map[string]any{
		"prompt": "make sky blue",
		"file":   inputFile,
	}))
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if result.IsError {
		t.Fatalf("unexpected error result: %s", contentText(t, result))
	}
	if len(result.Content) < 2 {
		t.Fatalf("expected at least 2 content items, got %d", len(result.Content))
	}
}

func TestRestoreImage_Success(t *testing.T) {
	srv := newMockGeminiServer(t)
	defer srv.Close()

	tmpDir := t.TempDir()
	origDir, _ := os.Getwd()
	os.Chdir(tmpDir)
	defer os.Chdir(origDir)

	t.Setenv("GEMINI_API_KEY", "test-key")
	t.Setenv("GEMINI_BASE_URL", srv.URL)
	t.Setenv("NABA_CONFIG_DIR", t.TempDir())

	inputFile := filepath.Join(tmpDir, "old-photo.jpg")
	os.WriteFile(inputFile, []byte("fake jpg"), 0o644)

	result, err := handleRestoreImage(context.Background(), makeRequest(map[string]any{
		"file": inputFile,
	}))
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if result.IsError {
		t.Fatalf("unexpected error result: %s", contentText(t, result))
	}
	if len(result.Content) < 2 {
		t.Fatalf("expected at least 2 content items, got %d", len(result.Content))
	}
}

func TestGeneratePattern_Success(t *testing.T) {
	srv := newMockGeminiServer(t)
	defer srv.Close()

	tmpDir := t.TempDir()
	origDir, _ := os.Getwd()
	os.Chdir(tmpDir)
	defer os.Chdir(origDir)

	t.Setenv("GEMINI_API_KEY", "test-key")
	t.Setenv("GEMINI_BASE_URL", srv.URL)
	t.Setenv("NABA_CONFIG_DIR", t.TempDir())

	result, err := handleGeneratePattern(context.Background(), makeRequest(map[string]any{
		"prompt": "floral wallpaper",
		"style":  "organic",
		"colors": "duotone",
	}))
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if result.IsError {
		t.Fatalf("unexpected error result: %s", contentText(t, result))
	}
	if len(result.Content) < 2 {
		t.Fatalf("expected at least 2 content items, got %d", len(result.Content))
	}
}

func TestGenerateDiagram_Success(t *testing.T) {
	srv := newMockGeminiServer(t)
	defer srv.Close()

	tmpDir := t.TempDir()
	origDir, _ := os.Getwd()
	os.Chdir(tmpDir)
	defer os.Chdir(origDir)

	t.Setenv("GEMINI_API_KEY", "test-key")
	t.Setenv("GEMINI_BASE_URL", srv.URL)
	t.Setenv("NABA_CONFIG_DIR", t.TempDir())

	result, err := handleGenerateDiagram(context.Background(), makeRequest(map[string]any{
		"prompt": "auth flow",
		"type":   "flowchart",
	}))
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if result.IsError {
		t.Fatalf("unexpected error result: %s", contentText(t, result))
	}
	if len(result.Content) < 2 {
		t.Fatalf("expected at least 2 content items, got %d", len(result.Content))
	}
}

// --- Multi-Output Tests ---

func TestGenerateImage_MultipleCount(t *testing.T) {
	srv, count := newCountingServer(t)
	defer srv.Close()

	tmpDir := t.TempDir()
	origDir, _ := os.Getwd()
	os.Chdir(tmpDir)
	defer os.Chdir(origDir)

	t.Setenv("GEMINI_API_KEY", "test-key")
	t.Setenv("GEMINI_BASE_URL", srv.URL)
	t.Setenv("NABA_CONFIG_DIR", t.TempDir())

	result, err := handleGenerateImage(context.Background(), makeRequest(map[string]any{
		"prompt": "test",
		"count":  float64(3),
	}))
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if result.IsError {
		t.Fatalf("unexpected error result: %s", contentText(t, result))
	}

	// Should have made 3 API calls
	if got := count.Load(); got != 3 {
		t.Fatalf("expected 3 API calls, got %d", got)
	}

	// Should have 3 text + 3 image content items = 6 total
	if len(result.Content) != 6 {
		t.Fatalf("expected 6 content items, got %d", len(result.Content))
	}
}

func TestGenerateStory_MultipleSteps(t *testing.T) {
	srv, count := newCountingServer(t)
	defer srv.Close()

	tmpDir := t.TempDir()
	origDir, _ := os.Getwd()
	os.Chdir(tmpDir)
	defer os.Chdir(origDir)

	t.Setenv("GEMINI_API_KEY", "test-key")
	t.Setenv("GEMINI_BASE_URL", srv.URL)
	t.Setenv("NABA_CONFIG_DIR", t.TempDir())

	result, err := handleGenerateStory(context.Background(), makeRequest(map[string]any{
		"prompt": "a cat adventure",
		"steps":  float64(3),
	}))
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if result.IsError {
		t.Fatalf("unexpected error result: %s", contentText(t, result))
	}

	if got := count.Load(); got != 3 {
		t.Fatalf("expected 3 API calls for 3 steps, got %d", got)
	}

	// 3 text + 3 image = 6
	if len(result.Content) != 6 {
		t.Fatalf("expected 6 content items, got %d", len(result.Content))
	}
}

func TestGenerateIcon_MultipleSizes(t *testing.T) {
	srv, count := newCountingServer(t)
	defer srv.Close()

	tmpDir := t.TempDir()
	origDir, _ := os.Getwd()
	os.Chdir(tmpDir)
	defer os.Chdir(origDir)

	t.Setenv("GEMINI_API_KEY", "test-key")
	t.Setenv("GEMINI_BASE_URL", srv.URL)
	t.Setenv("NABA_CONFIG_DIR", t.TempDir())

	result, err := handleGenerateIcon(context.Background(), makeRequest(map[string]any{
		"prompt": "rocket icon",
		"sizes":  []any{float64(64), float64(256)},
	}))
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if result.IsError {
		t.Fatalf("unexpected error result: %s", contentText(t, result))
	}

	if got := count.Load(); got != 2 {
		t.Fatalf("expected 2 API calls for 2 sizes, got %d", got)
	}

	// 2 text + 2 image = 4
	if len(result.Content) != 4 {
		t.Fatalf("expected 4 content items, got %d", len(result.Content))
	}
}

// --- Tool Definition Tests ---

func TestToolDefinitions_HaveRequiredParams(t *testing.T) {
	tools := []struct {
		name     string
		tool     mcpsdk.Tool
		required []string
	}{
		{"generate_image", generateImageTool(), []string{"prompt"}},
		{"edit_image", editImageTool(), []string{"prompt", "file"}},
		{"restore_image", restoreImageTool(), []string{"file"}},
		{"generate_icon", generateIconTool(), []string{"prompt"}},
		{"generate_pattern", generatePatternTool(), []string{"prompt"}},
		{"generate_story", generateStoryTool(), []string{"prompt"}},
		{"generate_diagram", generateDiagramTool(), []string{"prompt"}},
	}

	for _, tt := range tools {
		t.Run(tt.name, func(t *testing.T) {
			if tt.tool.Name != tt.name {
				t.Fatalf("expected tool name %q, got %q", tt.name, tt.tool.Name)
			}
			required := tt.tool.InputSchema.Required
			for _, req := range tt.required {
				found := false
				for _, r := range required {
					if r == req {
						found = true
						break
					}
				}
				if !found {
					t.Fatalf("expected %q to be required, got required: %v", req, required)
				}
			}
		})
	}
}

// --- Helpers ---

// contentText extracts the first text content from a result.
func contentText(t *testing.T, result *mcpsdk.CallToolResult) string {
	t.Helper()
	for _, c := range result.Content {
		if tc, ok := c.(mcpsdk.TextContent); ok {
			return tc.Text
		}
	}
	t.Fatal("no text content found in result")
	return ""
}
