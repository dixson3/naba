package output

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestWriteImage(t *testing.T) {
	tmpDir := t.TempDir()

	t.Run("auto-generated filename", func(t *testing.T) {
		path, err := WriteImage([]byte("png-data"), "image/png", "", "generate", 0)
		if err != nil {
			t.Fatal(err)
		}
		defer os.Remove(path)

		if !filepath.IsAbs(path) {
			t.Error("expected absolute path")
		}
		if !strings.Contains(path, "nba-generate-") {
			t.Errorf("expected auto-generated name, got %s", path)
		}
		if !strings.HasSuffix(path, ".png") {
			t.Errorf("expected .png extension, got %s", path)
		}

		data, err := os.ReadFile(path)
		if err != nil {
			t.Fatal(err)
		}
		if string(data) != "png-data" {
			t.Error("data mismatch")
		}
	})

	t.Run("specified output path", func(t *testing.T) {
		outPath := filepath.Join(tmpDir, "out.png")
		path, err := WriteImage([]byte("data"), "image/png", outPath, "generate", 0)
		if err != nil {
			t.Fatal(err)
		}
		if path != outPath {
			t.Errorf("expected %s, got %s", outPath, path)
		}
	})

	t.Run("creates directories", func(t *testing.T) {
		outPath := filepath.Join(tmpDir, "subdir", "deep", "out.png")
		path, err := WriteImage([]byte("data"), "image/png", outPath, "generate", 0)
		if err != nil {
			t.Fatal(err)
		}
		if path != outPath {
			t.Errorf("expected %s, got %s", outPath, path)
		}
	})

	t.Run("dedup existing file", func(t *testing.T) {
		outPath := filepath.Join(tmpDir, "dup.png")
		// Write first file
		_, err := WriteImage([]byte("data1"), "image/png", outPath, "generate", 0)
		if err != nil {
			t.Fatal(err)
		}
		// Write second file — should dedup
		path2, err := WriteImage([]byte("data2"), "image/png", outPath, "generate", 0)
		if err != nil {
			t.Fatal(err)
		}
		if path2 == outPath {
			t.Error("expected deduped filename")
		}
		if !strings.Contains(path2, "dup-1.png") {
			t.Errorf("expected dup-1.png, got %s", filepath.Base(path2))
		}
	})

	t.Run("multiple outputs with index", func(t *testing.T) {
		outPath := filepath.Join(tmpDir, "multi.png")
		path, err := WriteImage([]byte("data"), "image/png", outPath, "generate", 1)
		if err != nil {
			t.Fatal(err)
		}
		if !strings.Contains(filepath.Base(path), "multi-2") {
			t.Errorf("expected indexed filename, got %s", filepath.Base(path))
		}
	})
}

func TestMimeTypeToExt(t *testing.T) {
	tests := []struct {
		mime string
		ext  string
	}{
		{"image/png", ".png"},
		{"image/jpeg", ".jpg"},
		{"image/gif", ".gif"},
		{"image/webp", ".webp"},
		{"image/unknown", ".png"},
	}
	for _, tt := range tests {
		got := mimeTypeToExt(tt.mime)
		if got != tt.ext {
			t.Errorf("mimeTypeToExt(%q) = %q, want %q", tt.mime, got, tt.ext)
		}
	}
}

func TestExtForFormat(t *testing.T) {
	tests := []struct {
		format string
		ext    string
	}{
		{"png", ".png"},
		{"jpeg", ".jpg"},
		{"jpg", ".jpg"},
		{"PNG", ".png"},
		{"unknown", ".png"},
	}
	for _, tt := range tests {
		got := ExtForFormat(tt.format)
		if got != tt.ext {
			t.Errorf("ExtForFormat(%q) = %q, want %q", tt.format, got, tt.ext)
		}
	}
}

func TestWriteImage_JPEG(t *testing.T) {
	path, err := WriteImage([]byte("jpeg-data"), "image/jpeg", "", "generate", 0)
	if err != nil {
		t.Fatal(err)
	}
	defer os.Remove(path)

	if !strings.HasSuffix(path, ".jpg") {
		t.Errorf("expected path ending in .jpg, got %s", path)
	}
}

func TestWriteImage_EmptyData(t *testing.T) {
	tmpDir := t.TempDir()
	outPath := filepath.Join(tmpDir, "empty.png")

	path, err := WriteImage([]byte{}, "image/png", outPath, "generate", 0)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	info, err := os.Stat(path)
	if err != nil {
		t.Fatalf("file does not exist: %v", err)
	}
	if info.Size() != 0 {
		t.Errorf("expected zero-length file, got %d bytes", info.Size())
	}
}

func TestGenerateFilename_WithIndex(t *testing.T) {
	name := generateFilename("generate", "image/png", 2)

	if !strings.Contains(name, "-3") {
		t.Errorf("expected name to contain '-3' (index+1), got %s", name)
	}
	if !strings.HasSuffix(name, ".png") {
		t.Errorf("expected .png suffix, got %s", name)
	}
}

func TestGenerateFilename_ZeroIndex(t *testing.T) {
	name := generateFilename("generate", "image/png", 0)

	// With zero index, there should be no trailing number before .png.
	// The format is "nba-generate-YYYYMMDD-HHMMSS.png" — no index suffix.
	withoutExt := strings.TrimSuffix(name, ".png")
	if !strings.HasSuffix(name, ".png") {
		t.Errorf("expected .png suffix, got %s", name)
	}
	// The last character before .png should not be a digit from an index suffix.
	// The timestamp ends with digits, but there should be no "-N" index appended.
	// We verify by checking the format directly: should not match "-\d+.png" after timestamp.
	parts := strings.Split(withoutExt, "-")
	// Expected parts: ["nba", "generate", "YYYYMMDD", "HHMMSS"]
	if len(parts) != 4 {
		t.Errorf("expected 4 dash-separated parts (nba-generate-date-time), got %d: %s", len(parts), name)
	}
}

func TestDedup_MultipleConflicts(t *testing.T) {
	tmpDir := t.TempDir()

	// Create conflicting files: test.png, test-1.png, test-2.png
	for _, name := range []string{"test.png", "test-1.png", "test-2.png"} {
		f, err := os.Create(filepath.Join(tmpDir, name))
		if err != nil {
			t.Fatal(err)
		}
		f.Close()
	}

	result := dedup(filepath.Join(tmpDir, "test.png"))
	if !strings.HasSuffix(result, "test-3.png") {
		t.Errorf("expected path ending in test-3.png, got %s", result)
	}
}
