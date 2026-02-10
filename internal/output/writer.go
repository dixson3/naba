// Package output handles file writing, JSON formatting, and system preview for generated images.
package output

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"time"
)

// WriteImage writes image data to a file, returning the absolute path.
func WriteImage(data []byte, mimeType, outputPath, command string, index int) (string, error) {
	if outputPath == "" {
		outputPath = generateFilename(command, mimeType, index)
	} else if index > 0 {
		// Multiple outputs: append index to filename
		ext := filepath.Ext(outputPath)
		base := strings.TrimSuffix(outputPath, ext)
		outputPath = fmt.Sprintf("%s-%d%s", base, index+1, ext)
	}

	// Ensure directory exists
	dir := filepath.Dir(outputPath)
	if dir != "." && dir != "" {
		if err := os.MkdirAll(dir, 0o755); err != nil {
			return "", fmt.Errorf("create output directory: %w", err)
		}
	}

	// Dedup: if file exists, add suffix
	outputPath = dedup(outputPath)

	if err := os.WriteFile(outputPath, data, 0o644); err != nil {
		return "", fmt.Errorf("write image: %w", err)
	}

	absPath, err := filepath.Abs(outputPath)
	if err != nil {
		return outputPath, nil
	}
	return absPath, nil
}

func generateFilename(command, mimeType string, index int) string {
	ext := mimeTypeToExt(mimeType)
	ts := time.Now().Format("20060102-150405")
	if index > 0 {
		return fmt.Sprintf("naba-%s-%s-%d%s", command, ts, index+1, ext)
	}
	return fmt.Sprintf("naba-%s-%s%s", command, ts, ext)
}

func mimeTypeToExt(mimeType string) string {
	switch mimeType {
	case "image/png":
		return ".png"
	case "image/jpeg":
		return ".jpg"
	case "image/gif":
		return ".gif"
	case "image/webp":
		return ".webp"
	default:
		return ".png"
	}
}

func dedup(path string) string {
	if _, err := os.Stat(path); os.IsNotExist(err) {
		return path
	}
	ext := filepath.Ext(path)
	base := strings.TrimSuffix(path, ext)
	for i := 1; i < 1000; i++ {
		candidate := fmt.Sprintf("%s-%d%s", base, i, ext)
		if _, err := os.Stat(candidate); os.IsNotExist(err) {
			return candidate
		}
	}
	return path
}

// ExtForFormat returns the file extension for a given format string.
func ExtForFormat(format string) string {
	switch strings.ToLower(format) {
	case "jpeg", "jpg":
		return ".jpg"
	case "png":
		return ".png"
	default:
		return ".png"
	}
}
