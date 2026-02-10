package output

import (
	"bytes"
	"encoding/json"
	"io"
	"os"
	"strings"
	"testing"
	"time"
)

func captureStdout(fn func()) string {
	old := os.Stdout
	r, w, _ := os.Pipe()
	os.Stdout = w

	fn()

	w.Close()
	os.Stdout = old

	var buf bytes.Buffer
	io.Copy(&buf, r)
	return buf.String()
}

func TestNewResult(t *testing.T) {
	start := time.Now().Add(-100 * time.Millisecond)
	r := NewResult("/path", "generate", "test prompt", start)

	if r.Path != "/path" {
		t.Errorf("Path = %q, want %q", r.Path, "/path")
	}
	if r.Command != "generate" {
		t.Errorf("Command = %q, want %q", r.Command, "generate")
	}
	if r.Prompt != "test prompt" {
		t.Errorf("Prompt = %q, want %q", r.Prompt, "test prompt")
	}
	if r.ElapsedMs < 100 {
		t.Errorf("ElapsedMs = %d, want >= 100", r.ElapsedMs)
	}
	if r.Params != nil {
		t.Errorf("Params = %v, want nil", r.Params)
	}
}

func TestPrintJSON(t *testing.T) {
	r := Result{
		Path:      "/tmp/test.png",
		Command:   "generate",
		Prompt:    "a cat",
		ElapsedMs: 42,
	}

	output := captureStdout(func() {
		if err := PrintJSON(r); err != nil {
			t.Fatal(err)
		}
	})

	var parsed map[string]any
	if err := json.Unmarshal([]byte(output), &parsed); err != nil {
		t.Fatalf("output is not valid JSON: %v\noutput: %s", err, output)
	}

	if parsed["path"] != "/tmp/test.png" {
		t.Errorf("path = %v, want %q", parsed["path"], "/tmp/test.png")
	}
	if parsed["command"] != "generate" {
		t.Errorf("command = %v, want %q", parsed["command"], "generate")
	}
	if parsed["prompt"] != "a cat" {
		t.Errorf("prompt = %v, want %q", parsed["prompt"], "a cat")
	}
	if parsed["elapsed_ms"] != float64(42) {
		t.Errorf("elapsed_ms = %v, want 42", parsed["elapsed_ms"])
	}
}

func TestPrintJSON_OmitsEmptyParams(t *testing.T) {
	r := Result{
		Path:      "/tmp/test.png",
		Command:   "generate",
		Prompt:    "a cat",
		ElapsedMs: 10,
		Params:    nil,
	}

	output := captureStdout(func() {
		if err := PrintJSON(r); err != nil {
			t.Fatal(err)
		}
	})

	if strings.Contains(output, `"params"`) {
		t.Errorf("output should not contain \"params\" when Params is nil, got:\n%s", output)
	}
}

func TestPrintJSONMulti(t *testing.T) {
	results := []Result{
		{Path: "/a.png", Command: "generate", Prompt: "one", ElapsedMs: 1},
		{Path: "/b.png", Command: "edit", Prompt: "two", ElapsedMs: 2},
		{Path: "/c.png", Command: "icon", Prompt: "three", ElapsedMs: 3},
	}

	output := captureStdout(func() {
		if err := PrintJSONMulti(results); err != nil {
			t.Fatal(err)
		}
	})

	var parsed []map[string]any
	if err := json.Unmarshal([]byte(output), &parsed); err != nil {
		t.Fatalf("output is not valid JSON array: %v\noutput: %s", err, output)
	}

	if len(parsed) != 3 {
		t.Errorf("len = %d, want 3", len(parsed))
	}
}

func TestPrintJSONMulti_Empty(t *testing.T) {
	output := captureStdout(func() {
		if err := PrintJSONMulti([]Result{}); err != nil {
			t.Fatal(err)
		}
	})

	trimmed := strings.TrimSpace(output)
	if !json.Valid([]byte(trimmed)) {
		t.Errorf("output is not valid JSON: %q", trimmed)
	}
}
