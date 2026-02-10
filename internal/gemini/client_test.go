package gemini

import (
	"encoding/base64"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"strings"
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

func TestNewClient_DefaultModel(t *testing.T) {
	c := NewClient("key", "")
	if c.Model != "gemini-2.0-flash-exp-image-generation" {
		t.Errorf("expected default model %q, got %q", "gemini-2.0-flash-exp-image-generation", c.Model)
	}
}

func TestNewClient_CustomModel(t *testing.T) {
	c := NewClient("key", "custom-model")
	if c.Model != "custom-model" {
		t.Errorf("expected model %q, got %q", "custom-model", c.Model)
	}
	if c.BaseURL != defaultBaseURL {
		t.Errorf("expected base URL %q, got %q", defaultBaseURL, c.BaseURL)
	}
	if c.HTTPClient == nil {
		t.Error("expected HTTPClient to be non-nil")
	}
}

func TestNewClient_BaseURLOverride(t *testing.T) {
	t.Setenv("GEMINI_BASE_URL", "http://localhost:9999")
	c := NewClient("key", "")
	if c.BaseURL != "http://localhost:9999" {
		t.Errorf("expected base URL %q, got %q", "http://localhost:9999", c.BaseURL)
	}
}

func TestDoRequest_PromptBlocked(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		resp := GenerateResponse{
			PromptFeedback: &PromptFeedback{
				BlockReason: "SAFETY",
			},
			Candidates: []Candidate{},
		}
		json.NewEncoder(w).Encode(resp)
	}))
	defer server.Close()

	client := NewClient("test-key", "test-model")
	client.BaseURL = server.URL

	_, err := client.Generate("test")
	if err == nil {
		t.Fatal("expected error for blocked prompt")
	}
	apiErr, ok := err.(*APIError)
	if !ok {
		t.Fatalf("expected *APIError, got %T", err)
	}
	if apiErr.ExitCode != ExitAPI {
		t.Errorf("expected exit code %d, got %d", ExitAPI, apiErr.ExitCode)
	}
	if !strings.Contains(apiErr.Message, "prompt blocked") {
		t.Errorf("expected message to contain %q, got %q", "prompt blocked", apiErr.Message)
	}
}

func TestDoRequest_MalformedJSON(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Write([]byte("not json"))
	}))
	defer server.Close()

	client := NewClient("test-key", "test-model")
	client.BaseURL = server.URL

	_, err := client.Generate("test")
	if err == nil {
		t.Fatal("expected error for malformed JSON")
	}
	if !strings.Contains(err.Error(), "parse response") {
		t.Errorf("expected error to contain %q, got %q", "parse response", err.Error())
	}
}

func TestDoRequest_EmptyCandidates(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		resp := GenerateResponse{
			Candidates: []Candidate{},
		}
		json.NewEncoder(w).Encode(resp)
	}))
	defer server.Close()

	client := NewClient("test-key", "test-model")
	client.BaseURL = server.URL

	_, err := client.Generate("test")
	if err == nil {
		t.Fatal("expected error for empty candidates")
	}
	if !strings.Contains(err.Error(), "no images") {
		t.Errorf("expected error to contain %q, got %q", "no images", err.Error())
	}
}

func TestExtractImages_NilContent(t *testing.T) {
	b64 := base64.StdEncoding.EncodeToString([]byte("hello"))
	resp := GenerateResponse{
		Candidates: []Candidate{
			{Content: nil},
			{Content: &Content{
				Role: "model",
				Parts: []Part{{
					InlineData: &InlineData{
						MIMEType: "image/png",
						Data:     b64,
					},
				}},
			}},
		},
	}

	images, err := extractImages(resp)
	if err != nil {
		t.Fatalf("extractImages: %v", err)
	}
	if len(images) != 1 {
		t.Fatalf("expected 1 image, got %d", len(images))
	}
}

func TestExtractImages_TextOnlyParts(t *testing.T) {
	resp := GenerateResponse{
		Candidates: []Candidate{{
			Content: &Content{
				Role:  "model",
				Parts: []Part{{Text: "some text, no image"}},
			},
		}},
	}

	_, err := extractImages(resp)
	if err == nil {
		t.Fatal("expected error for text-only parts")
	}
	if !strings.Contains(err.Error(), "no images") {
		t.Errorf("expected error to contain %q, got %q", "no images", err.Error())
	}
}

func TestExtractImages_MultipleCandidatesMultipleImages(t *testing.T) {
	b64a := base64.StdEncoding.EncodeToString([]byte("img-a"))
	b64b := base64.StdEncoding.EncodeToString([]byte("img-b"))
	b64c := base64.StdEncoding.EncodeToString([]byte("img-c"))
	b64d := base64.StdEncoding.EncodeToString([]byte("img-d"))

	resp := GenerateResponse{
		Candidates: []Candidate{
			{Content: &Content{
				Role: "model",
				Parts: []Part{
					{InlineData: &InlineData{MIMEType: "image/png", Data: b64a}},
					{InlineData: &InlineData{MIMEType: "image/png", Data: b64b}},
				},
			}},
			{Content: &Content{
				Role: "model",
				Parts: []Part{
					{InlineData: &InlineData{MIMEType: "image/jpeg", Data: b64c}},
					{InlineData: &InlineData{MIMEType: "image/jpeg", Data: b64d}},
				},
			}},
		},
	}

	images, err := extractImages(resp)
	if err != nil {
		t.Fatalf("extractImages: %v", err)
	}
	if len(images) != 4 {
		t.Fatalf("expected 4 images, got %d", len(images))
	}
}

func TestExtractImages_MalformedBase64(t *testing.T) {
	resp := GenerateResponse{
		Candidates: []Candidate{{
			Content: &Content{
				Role: "model",
				Parts: []Part{{
					InlineData: &InlineData{
						MIMEType: "image/png",
						Data:     "!!!invalid!!!",
					},
				}},
			},
		}},
	}

	_, err := extractImages(resp)
	if err == nil {
		t.Fatal("expected error for malformed base64")
	}
	if !strings.Contains(err.Error(), "decode image data") {
		t.Errorf("expected error to contain %q, got %q", "decode image data", err.Error())
	}
}

func TestParseAPIError_MalformedBody(t *testing.T) {
	result := parseAPIError(500, []byte("not json"))
	if !strings.Contains(result.Message, "API error (HTTP 500)") {
		t.Errorf("expected message to contain %q, got %q", "API error (HTTP 500)", result.Message)
	}
}

func TestParseAPIError_403(t *testing.T) {
	errResp := ErrorResponse{
		Error: struct {
			Code    int    `json:"code"`
			Message string `json:"message"`
			Status  string `json:"status"`
		}{Code: 403, Message: "forbidden"},
	}
	body, _ := json.Marshal(errResp)

	result := parseAPIError(403, body)
	if result.ExitCode != ExitAuth {
		t.Errorf("expected exit code %d, got %d", ExitAuth, result.ExitCode)
	}
}

func TestParseAPIError_502_503(t *testing.T) {
	tests := []struct {
		statusCode int
		name       string
	}{
		{502, "502 Bad Gateway"},
		{503, "503 Service Unavailable"},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			errResp := ErrorResponse{
				Error: struct {
					Code    int    `json:"code"`
					Message string `json:"message"`
					Status  string `json:"status"`
				}{Code: tt.statusCode, Message: "server error"},
			}
			body, _ := json.Marshal(errResp)

			result := parseAPIError(tt.statusCode, body)
			if result.ExitCode != ExitAPI {
				t.Errorf("expected exit code %d, got %d", ExitAPI, result.ExitCode)
			}
			if !strings.Contains(result.Message, "server error") {
				t.Errorf("expected message to contain %q, got %q", "server error", result.Message)
			}
		})
	}
}

func TestGenerateRequestStructure(t *testing.T) {
	b64 := base64.StdEncoding.EncodeToString([]byte("result-image"))

	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		var req GenerateRequest
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			t.Fatalf("decode request: %v", err)
		}

		if len(req.Contents) == 0 {
			t.Fatal("expected at least one content entry")
		}
		if req.Contents[0].Role != "user" {
			t.Errorf("expected role %q, got %q", "user", req.Contents[0].Role)
		}
		if len(req.Contents[0].Parts) == 0 || req.Contents[0].Parts[0].Text == "" {
			t.Error("expected text part in request")
		}

		hasImage := false
		for _, mod := range req.GenerationConfig.ResponseModalities {
			if mod == "IMAGE" {
				hasImage = true
				break
			}
		}
		if !hasImage {
			t.Error("expected IMAGE in ResponseModalities")
		}

		if !strings.Contains(r.URL.Path, "my-model") {
			t.Errorf("expected URL path to contain model name, got %q", r.URL.Path)
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

	client := NewClient("test-key", "my-model")
	client.BaseURL = server.URL

	images, err := client.Generate("a blue sky")
	if err != nil {
		t.Fatalf("Generate: %v", err)
	}
	if len(images) != 1 {
		t.Fatalf("expected 1 image, got %d", len(images))
	}
}

func TestAPIError_ErrorInterface(t *testing.T) {
	apiErr := &APIError{Message: "test"}
	var err error = apiErr
	if err.Error() != "test" {
		t.Errorf("expected %q, got %q", "test", err.Error())
	}
}
