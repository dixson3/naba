package output

import (
	"encoding/json"
	"fmt"
	"time"
)

// Result holds metadata about a generated image for JSON output.
type Result struct {
	Path      string            `json:"path"`
	Command   string            `json:"command"`
	Prompt    string            `json:"prompt"`
	ElapsedMs int64             `json:"elapsed_ms"`
	Params    map[string]any    `json:"params,omitempty"`
}

// PrintJSON outputs a Result as formatted JSON to stdout.
func PrintJSON(r Result) error {
	data, err := json.MarshalIndent(r, "", "  ")
	if err != nil {
		return err
	}
	fmt.Println(string(data))
	return nil
}

// PrintJSONMulti outputs multiple Results as a JSON array to stdout.
func PrintJSONMulti(results []Result) error {
	data, err := json.MarshalIndent(results, "", "  ")
	if err != nil {
		return err
	}
	fmt.Println(string(data))
	return nil
}

// NewResult creates a Result with common fields populated.
func NewResult(path, command, prompt string, start time.Time) Result {
	return Result{
		Path:      path,
		Command:   command,
		Prompt:    prompt,
		ElapsedMs: time.Since(start).Milliseconds(),
	}
}
