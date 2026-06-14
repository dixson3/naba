package gemini

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strings"
)

// ListModels calls the models.list endpoint and returns the available model ids with the
// "models/" prefix stripped (e.g. "gemini-3.1-flash-image"). It performs no image
// generation, so it is a cheap, no-cost liveness check for an API key — used by
// `naba doctor` to validate the key and confirm the configured model is reachable.
func (c *Client) ListModels() ([]string, error) {
	url := fmt.Sprintf("%s/models?pageSize=1000", c.BaseURL)
	httpReq, err := http.NewRequest("GET", url, nil)
	if err != nil {
		return nil, fmt.Errorf("create request: %w", err)
	}
	httpReq.Header.Set("x-goog-api-key", c.APIKey)

	resp, err := c.HTTPClient.Do(httpReq)
	if err != nil {
		return nil, fmt.Errorf("api request failed: %w", err)
	}
	defer func() { _ = resp.Body.Close() }()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("read response: %w", err)
	}
	if resp.StatusCode != http.StatusOK {
		return nil, parseAPIError(resp.StatusCode, body)
	}

	var lr struct {
		Models []struct {
			Name string `json:"name"`
		} `json:"models"`
	}
	if err := json.Unmarshal(body, &lr); err != nil {
		return nil, fmt.Errorf("parse response: %w", err)
	}

	names := make([]string, 0, len(lr.Models))
	for _, m := range lr.Models {
		names = append(names, strings.TrimPrefix(m.Name, "models/"))
	}
	return names, nil
}

// ModelReachable reports whether modelID appears in the available list, applying the
// "models/" prefix normalization on both sides.
func ModelReachable(modelID string, available []string) bool {
	want := strings.TrimPrefix(modelID, "models/")
	for _, m := range available {
		if strings.TrimPrefix(m, "models/") == want {
			return true
		}
	}
	return false
}
