package output

import "testing"

func TestPreview_DoesNotPanic(t *testing.T) {
	// Preview may return an error for a nonexistent file,
	// but it must not panic.
	_ = Preview("/nonexistent/path/file.png")
}
