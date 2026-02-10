package gemini

import (
	"encoding/base64"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"testing"
)

func TestGenerate(t *testing.T) {
	imgData := []byte("fake-png-data")
	b64 := base64.StdEncoding.EncodeToString(imgData)

	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Header.Get("x-goog-api-key") != "test-key" {
			t.Errorf("expected api key header, got %q", r.Header.Get("x-goog-api-key"))
		}
		if r.Header.Get("Content-Type") != "application/json" {
			t.Errorf("expected json content type, got %q", r.Header.Get("Content-Type"))
		}

		var req GenerateRequest
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			t.Fatalf("decode request: %v", err)
		}
		if len(req.Contents) == 0 || len(req.Contents[0].Parts) == 0 {
			t.Fatal("expected content parts in request")
		}
		if req.Contents[0].Parts[0].Text == "" {
			t.Error("expected text in request")
		}

		resp := GenerateResponse{
			Candidates: []Candidate{{
				Content: &Content{
					Role: "model",
					Parts: []Part{{
						InlineData: &InlineData{
							MIMEType: "image/png",
							Data:     b64,
						},
					}},
				},
			}},
		}
		json.NewEncoder(w).Encode(resp)
	}))
	defer server.Close()

	client := NewClient("test-key", "test-model")
	client.BaseURL = server.URL

	images, err := client.Generate("a red apple")
	if err != nil {
		t.Fatalf("Generate: %v", err)
	}
	if len(images) != 1 {
		t.Fatalf("expected 1 image, got %d", len(images))
	}
	if string(images[0].Data) != string(imgData) {
		t.Error("image data mismatch")
	}
	if images[0].MIMEType != "image/png" {
		t.Errorf("expected image/png, got %s", images[0].MIMEType)
	}
}

func TestGenerateWithImage(t *testing.T) {
	// Create a temp image file
	tmpDir := t.TempDir()
	imgPath := filepath.Join(tmpDir, "test.png")
	if err := os.WriteFile(imgPath, []byte("test-image"), 0o644); err != nil {
		t.Fatal(err)
	}

	b64Out := base64.StdEncoding.EncodeToString([]byte("edited-image"))

	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		var req GenerateRequest
		json.NewDecoder(r.Body).Decode(&req)

		if len(req.Contents[0].Parts) < 2 {
			t.Fatal("expected text and image parts")
		}
		if req.Contents[0].Parts[0].Text == "" {
			t.Error("expected text prompt")
		}
		if req.Contents[0].Parts[1].InlineData == nil {
			t.Fatal("expected inline data")
		}
		if req.Contents[0].Parts[1].InlineData.MIMEType != "image/png" {
			t.Errorf("expected image/png mime type, got %s", req.Contents[0].Parts[1].InlineData.MIMEType)
		}

		resp := GenerateResponse{
			Candidates: []Candidate{{
				Content: &Content{
					Role: "model",
					Parts: []Part{{
						InlineData: &InlineData{
							MIMEType: "image/png",
							Data:     b64Out,
						},
					}},
				},
			}},
		}
		json.NewEncoder(w).Encode(resp)
	}))
	defer server.Close()

	client := NewClient("test-key", "")
	client.BaseURL = server.URL

	images, err := client.GenerateWithImage("make it blue", imgPath)
	if err != nil {
		t.Fatalf("GenerateWithImage: %v", err)
	}
	if len(images) != 1 {
		t.Fatalf("expected 1 image, got %d", len(images))
	}
	if string(images[0].Data) != "edited-image" {
		t.Error("image data mismatch")
	}
}

func TestAPIError401(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusUnauthorized)
		json.NewEncoder(w).Encode(ErrorResponse{
			Error: struct {
				Code    int    `json:"code"`
				Message string `json:"message"`
				Status  string `json:"status"`
			}{Code: 401, Message: "invalid api key"},
		})
	}))
	defer server.Close()

	client := NewClient("bad-key", "")
	client.BaseURL = server.URL

	_, err := client.Generate("test")
	if err == nil {
		t.Fatal("expected error")
	}
	apiErr, ok := err.(*APIError)
	if !ok {
		t.Fatalf("expected APIError, got %T", err)
	}
	if apiErr.ExitCode != ExitAuth {
		t.Errorf("expected exit code %d, got %d", ExitAuth, apiErr.ExitCode)
	}
}

func TestAPIError429(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusTooManyRequests)
		json.NewEncoder(w).Encode(ErrorResponse{
			Error: struct {
				Code    int    `json:"code"`
				Message string `json:"message"`
				Status  string `json:"status"`
			}{Code: 429, Message: "rate limited"},
		})
	}))
	defer server.Close()

	client := NewClient("test-key", "")
	client.BaseURL = server.URL

	_, err := client.Generate("test")
	if err == nil {
		t.Fatal("expected error")
	}
	apiErr, ok := err.(*APIError)
	if !ok {
		t.Fatalf("expected APIError, got %T", err)
	}
	if apiErr.ExitCode != ExitRateLimit {
		t.Errorf("expected exit code %d, got %d", ExitRateLimit, apiErr.ExitCode)
	}
}

func TestAPIError500(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusInternalServerError)
		json.NewEncoder(w).Encode(ErrorResponse{
			Error: struct {
				Code    int    `json:"code"`
				Message string `json:"message"`
				Status  string `json:"status"`
			}{Code: 500, Message: "internal error"},
		})
	}))
	defer server.Close()

	client := NewClient("test-key", "")
	client.BaseURL = server.URL

	_, err := client.Generate("test")
	if err == nil {
		t.Fatal("expected error")
	}
	apiErr, ok := err.(*APIError)
	if !ok {
		t.Fatalf("expected APIError, got %T", err)
	}
	if apiErr.ExitCode != ExitAPI {
		t.Errorf("expected exit code %d, got %d", ExitAPI, apiErr.ExitCode)
	}
}

func TestNoImagesInResponse(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		resp := GenerateResponse{
			Candidates: []Candidate{{
				Content: &Content{
					Role:  "model",
					Parts: []Part{{Text: "I cannot generate that image."}},
				},
			}},
		}
		json.NewEncoder(w).Encode(resp)
	}))
	defer server.Close()

	client := NewClient("test-key", "")
	client.BaseURL = server.URL

	_, err := client.Generate("test")
	if err == nil {
		t.Fatal("expected error for no images")
	}
}

func TestReadImageFileMissing(t *testing.T) {
	_, _, err := readImageFile("/nonexistent/path.png")
	if err == nil {
		t.Fatal("expected error for missing file")
	}
	apiErr, ok := err.(*APIError)
	if !ok {
		t.Fatalf("expected APIError, got %T", err)
	}
	if apiErr.ExitCode != ExitFileIO {
		t.Errorf("expected exit code %d, got %d", ExitFileIO, apiErr.ExitCode)
	}
}

func TestDetectMIMEType(t *testing.T) {
	tests := []struct {
		path     string
		expected string
	}{
		{"image.png", "image/png"},
		{"image.PNG", "image/png"},
		{"image.jpg", "image/jpeg"},
		{"image.jpeg", "image/jpeg"},
		{"image.gif", "image/gif"},
		{"image.webp", "image/webp"},
		{"image.bmp", "image/bmp"},
		{"image.unknown", "image/png"},
	}
	for _, tt := range tests {
		got := detectMIMEType(tt.path)
		if got != tt.expected {
			t.Errorf("detectMIMEType(%q) = %q, want %q", tt.path, got, tt.expected)
		}
	}
}
