package output

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestWriteImageResult_CorrectsExtension(t *testing.T) {
	dir := t.TempDir()
	out := filepath.Join(dir, "hero.png")
	// API returned JPEG but the user asked for .png.
	res, err := WriteImageResult([]byte("jpeg-bytes"), "image/jpeg", out, "generate", 0)
	if err != nil {
		t.Fatal(err)
	}
	if !res.Corrected {
		t.Error("expected Corrected = true for .png path with jpeg response")
	}
	if res.RequestedFormat != "png" || res.ActualFormat != "jpeg" {
		t.Errorf("requested=%q actual=%q, want png/jpeg", res.RequestedFormat, res.ActualFormat)
	}
	if !strings.HasSuffix(res.Path, ".jpg") {
		t.Errorf("expected corrected .jpg path, got %s", res.Path)
	}
	if _, err := os.Stat(res.Path); err != nil {
		t.Errorf("corrected file not written: %v", err)
	}
}

func TestWriteImageResult_NoCorrectionWhenMatch(t *testing.T) {
	dir := t.TempDir()
	out := filepath.Join(dir, "ok.jpg")
	res, err := WriteImageResult([]byte("d"), "image/jpeg", out, "generate", 0)
	if err != nil {
		t.Fatal(err)
	}
	if res.Corrected {
		t.Error("jpg path + jpeg response should not be corrected")
	}
	if res.Path != out {
		t.Errorf("path changed unexpectedly: %s", res.Path)
	}
}

func TestWriteImageResult_JpgJpegEquivalent(t *testing.T) {
	dir := t.TempDir()
	out := filepath.Join(dir, "x.jpeg")
	res, err := WriteImageResult([]byte("d"), "image/jpeg", out, "generate", 0)
	if err != nil {
		t.Fatal(err)
	}
	if res.Corrected {
		t.Error(".jpeg and image/jpeg are equivalent; no correction expected")
	}
}

func TestWriteImageResult_AutoNamedNoRequestedFormat(t *testing.T) {
	dir := t.TempDir()
	t.Chdir(dir)
	res, err := WriteImageResult([]byte("d"), "image/jpeg", "", "generate", 0)
	if err != nil {
		t.Fatal(err)
	}
	if res.RequestedFormat != "" {
		t.Errorf("auto-named output should have no requested format, got %q", res.RequestedFormat)
	}
	if res.ActualFormat != "jpeg" || !strings.HasSuffix(res.Path, ".jpg") {
		t.Errorf("expected jpeg/.jpg, got %s / %s", res.ActualFormat, res.Path)
	}
	if res.Corrected {
		t.Error("auto-named output is never a correction")
	}
}

func TestFormatHelpers(t *testing.T) {
	if formatFromExt(".JPG") != "jpeg" || formatFromExt(".png") != "png" || formatFromExt(".xyz") != "" {
		t.Error("formatFromExt mismatch")
	}
	if formatFromMIME("image/jpeg") != "jpeg" || formatFromMIME("image/png") != "png" {
		t.Error("formatFromMIME mismatch")
	}
}
