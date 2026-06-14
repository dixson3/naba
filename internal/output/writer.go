// Package output handles file writing, JSON formatting, and system preview for generated images.
package output

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"time"
)

// WriteResult is the outcome of writing an image, including any extension correction
// applied so the on-disk file matches the API's response mimeType.
type WriteResult struct {
	// Path is the absolute path actually written (extension already reconciled).
	Path string
	// RequestedFormat is the normalized format the caller's -o extension implied
	// (e.g. "png"), or "" when no output path was given (auto-named).
	RequestedFormat string
	// ActualFormat is the normalized format actually written, derived from the
	// response mimeType (e.g. "jpeg").
	ActualFormat string
	// Corrected is true when the on-disk extension was changed from the requested
	// one to match the response mimeType (e.g. hero.png -> hero.jpg).
	Corrected bool
}

// WriteImage writes image data to a file, returning the absolute path. It reconciles
// the on-disk extension to the response mimeType (see WriteImageResult). Retained for
// back-compat; callers that need the requested-vs-actual format should use
// WriteImageResult.
func WriteImage(data []byte, mimeType, outputPath, command string, index int) (string, error) {
	res, err := WriteImageResult(data, mimeType, outputPath, command, index)
	return res.Path, err
}

// WriteImageResult writes image data and reports any extension correction. The Gemini
// image API returns JPEG, so a user-supplied "-o foo.png" (or a hardcoded .png path)
// would otherwise mislabel a JPEG. When the requested extension disagrees with the
// response mimeType, the extension is corrected on disk and Corrected is set so the
// caller can warn and surface requested-vs-actual format.
func WriteImageResult(data []byte, mimeType, outputPath, command string, index int) (WriteResult, error) {
	res := WriteResult{ActualFormat: formatFromMIME(mimeType)}

	if outputPath == "" {
		outputPath = generateFilename(command, mimeType, index)
	} else {
		res.RequestedFormat = formatFromExt(filepath.Ext(outputPath))
		// Reconcile the extension to the response mimeType before any index suffix.
		if res.RequestedFormat != "" && res.RequestedFormat != res.ActualFormat {
			ext := filepath.Ext(outputPath)
			outputPath = strings.TrimSuffix(outputPath, ext) + mimeTypeToExt(mimeType)
			res.Corrected = true
		}
		if index > 0 {
			// Multiple outputs: append index to filename
			ext := filepath.Ext(outputPath)
			base := strings.TrimSuffix(outputPath, ext)
			outputPath = fmt.Sprintf("%s-%d%s", base, index+1, ext)
		}
	}

	// Ensure directory exists
	dir := filepath.Dir(outputPath)
	if dir != "." && dir != "" {
		if err := os.MkdirAll(dir, 0o755); err != nil {
			return res, fmt.Errorf("create output directory: %w", err)
		}
	}

	// Dedup: if file exists, add suffix
	outputPath = dedup(outputPath)

	if err := os.WriteFile(outputPath, data, 0o644); err != nil {
		return res, fmt.Errorf("write image: %w", err)
	}

	if absPath, err := filepath.Abs(outputPath); err == nil {
		res.Path = absPath
	} else {
		res.Path = outputPath
	}
	return res, nil
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

// OutputPath returns a full file path by joining a directory with an auto-generated filename.
// If dir is empty, returns empty string (caller uses CWD via WriteImage default).
func OutputPath(dir, command, mimeType string) string {
	if dir == "" {
		return ""
	}
	return filepath.Join(dir, generateFilename(command, mimeType, 0))
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

// formatFromExt returns a normalized format name for a file extension (".jpg" and
// ".jpeg" both -> "jpeg", ".png" -> "png"). An empty or unrecognized extension -> "".
func formatFromExt(ext string) string {
	switch strings.ToLower(ext) {
	case ".jpg", ".jpeg":
		return "jpeg"
	case ".png":
		return "png"
	case ".gif":
		return "gif"
	case ".webp":
		return "webp"
	default:
		return ""
	}
}

// formatFromMIME returns the normalized format name for a response mimeType, kept in
// lockstep with mimeTypeToExt so a reconciliation decision and the written extension
// never disagree.
func formatFromMIME(mimeType string) string {
	switch mimeType {
	case "image/png":
		return "png"
	case "image/jpeg":
		return "jpeg"
	case "image/gif":
		return "gif"
	case "image/webp":
		return "webp"
	default:
		return "png"
	}
}
