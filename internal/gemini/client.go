// Package gemini provides a client for the Google Gemini image generation API.
package gemini

import (
	"bytes"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"strings"
	"time"
)

const (
	defaultBaseURL = "https://generativelanguage.googleapis.com/v1beta"
	defaultModel   = "gemini-2.0-flash-exp-image-generation"

	ExitGeneral    = 1
	ExitUsage      = 2
	ExitAuth       = 3
	ExitRateLimit  = 4
	ExitAPI        = 5
	ExitFileIO     = 10
)

// Client is a Gemini API client for image generation.
type Client struct {
	APIKey     string
	Model      string
	BaseURL    string
	HTTPClient *http.Client
}

// NewClient creates a new Gemini client with the given API key.
func NewClient(apiKey, model string) *Client {
	if model == "" {
		model = defaultModel
	}
	baseURL := defaultBaseURL
	if override := os.Getenv("GEMINI_BASE_URL"); override != "" {
		baseURL = override
	}
	return &Client{
		APIKey:  apiKey,
		Model:   model,
		BaseURL: baseURL,
		HTTPClient: &http.Client{
			Timeout: 120 * time.Second,
		},
	}
}

// Generate sends a text prompt and returns generated images.
func (c *Client) Generate(prompt string) ([]ImageResult, error) {
	req := GenerateRequest{
		Contents: []Content{
			{
				Role:  "user",
				Parts: []Part{{Text: prompt}},
			},
		},
		GenerationConfig: GenerationConfig{
			ResponseModalities: []string{"TEXT", "IMAGE"},
		},
	}
	return c.doRequest(req)
}

// GenerateWithImage sends a prompt with an input image and returns generated images.
func (c *Client) GenerateWithImage(prompt, imagePath string) ([]ImageResult, error) {
	imageData, mimeType, err := readImageFile(imagePath)
	if err != nil {
		return nil, err
	}

	req := GenerateRequest{
		Contents: []Content{
			{
				Role: "user",
				Parts: []Part{
					{Text: prompt},
					{InlineData: &InlineData{
						MIMEType: mimeType,
						Data:     imageData,
					}},
				},
			},
		},
		GenerationConfig: GenerationConfig{
			ResponseModalities: []string{"TEXT", "IMAGE"},
		},
	}
	return c.doRequest(req)
}

func (c *Client) doRequest(req GenerateRequest) ([]ImageResult, error) {
	body, err := json.Marshal(req)
	if err != nil {
		return nil, fmt.Errorf("marshal request: %w", err)
	}

	url := fmt.Sprintf("%s/models/%s:generateContent", c.BaseURL, c.Model)
	httpReq, err := http.NewRequest("POST", url, bytes.NewReader(body))
	if err != nil {
		return nil, fmt.Errorf("create request: %w", err)
	}
	httpReq.Header.Set("Content-Type", "application/json")
	httpReq.Header.Set("x-goog-api-key", c.APIKey)

	resp, err := c.HTTPClient.Do(httpReq)
	if err != nil {
		return nil, fmt.Errorf("api request failed: %w", err)
	}
	defer func() { _ = resp.Body.Close() }()

	respBody, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("read response: %w", err)
	}

	if resp.StatusCode != http.StatusOK {
		return nil, parseAPIError(resp.StatusCode, respBody)
	}

	var genResp GenerateResponse
	if err := json.Unmarshal(respBody, &genResp); err != nil {
		return nil, fmt.Errorf("parse response: %w", err)
	}

	if genResp.PromptFeedback != nil && genResp.PromptFeedback.BlockReason != "" {
		return nil, &APIError{
			StatusCode: 0,
			Message:    fmt.Sprintf("prompt blocked: %s", genResp.PromptFeedback.BlockReason),
			ExitCode:   ExitAPI,
		}
	}

	return extractImages(genResp)
}

func extractImages(resp GenerateResponse) ([]ImageResult, error) {
	var images []ImageResult
	for _, candidate := range resp.Candidates {
		if candidate.Content == nil {
			continue
		}
		for _, part := range candidate.Content.Parts {
			if part.InlineData == nil {
				continue
			}
			data, err := base64.StdEncoding.DecodeString(part.InlineData.Data)
			if err != nil {
				return nil, fmt.Errorf("decode image data: %w", err)
			}
			images = append(images, ImageResult{
				Data:     data,
				MIMEType: part.InlineData.MIMEType,
			})
		}
	}
	if len(images) == 0 {
		return nil, &APIError{
			StatusCode: 0,
			Message:    "no images in response",
			ExitCode:   ExitAPI,
		}
	}
	return images, nil
}

func readImageFile(path string) (string, string, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return "", "", &APIError{
			StatusCode: 0,
			Message:    fmt.Sprintf("read image file %q: %v", path, err),
			ExitCode:   ExitFileIO,
		}
	}

	mimeType := detectMIMEType(path)
	encoded := base64.StdEncoding.EncodeToString(data)
	return encoded, mimeType, nil
}

func detectMIMEType(path string) string {
	switch strings.ToLower(filepath.Ext(path)) {
	case ".png":
		return "image/png"
	case ".jpg", ".jpeg":
		return "image/jpeg"
	case ".gif":
		return "image/gif"
	case ".webp":
		return "image/webp"
	case ".bmp":
		return "image/bmp"
	default:
		return "image/png"
	}
}

// APIError represents a structured error from the Gemini API.
type APIError struct {
	StatusCode int
	Message    string
	ExitCode   int
}

func (e *APIError) Error() string {
	return e.Message
}

func parseAPIError(statusCode int, body []byte) *APIError {
	var errResp ErrorResponse
	_ = json.Unmarshal(body, &errResp)

	msg := errResp.Error.Message
	if msg == "" {
		msg = fmt.Sprintf("API error (HTTP %d)", statusCode)
	}

	exitCode := ExitAPI
	switch {
	case statusCode == 401 || statusCode == 403:
		exitCode = ExitAuth
		msg = fmt.Sprintf("authentication failed: %s\n\nSet GEMINI_API_KEY or run: naba config set api_key <your-key>", msg)
	case statusCode == 429:
		exitCode = ExitRateLimit
		msg = fmt.Sprintf("rate limit exceeded: %s\n\nWait a moment and try again.", msg)
	case statusCode >= 500:
		msg = fmt.Sprintf("Gemini server error: %s\n\nThis is a temporary issue. Try again shortly.", msg)
	}

	return &APIError{
		StatusCode: statusCode,
		Message:    msg,
		ExitCode:   exitCode,
	}
}
