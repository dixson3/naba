package gemini

import (
	"strings"
	"testing"
)

func TestEnrichGeneratePrompt(t *testing.T) {
	t.Run("basic prompt", func(t *testing.T) {
		got := EnrichGeneratePrompt("a cat", "", nil)
		if got != "a cat" {
			t.Errorf("expected %q, got %q", "a cat", got)
		}
	})

	t.Run("with style", func(t *testing.T) {
		got := EnrichGeneratePrompt("a cat", "watercolor", nil)
		if !strings.Contains(got, "a cat") || !strings.Contains(got, "watercolor") {
			t.Errorf("expected prompt with style, got %q", got)
		}
	})

	t.Run("with variations", func(t *testing.T) {
		got := EnrichGeneratePrompt("a cat", "", []string{"lighting", "angle"})
		if !strings.Contains(got, "lighting") || !strings.Contains(got, "angle") {
			t.Errorf("expected prompt with variations, got %q", got)
		}
	})

	t.Run("with style and variations", func(t *testing.T) {
		got := EnrichGeneratePrompt("a cat", "anime", []string{"mood"})
		if !strings.Contains(got, "anime") || !strings.Contains(got, "mood") {
			t.Errorf("expected prompt with style and variations, got %q", got)
		}
	})
}

func TestEnrichEditPrompt(t *testing.T) {
	got := EnrichEditPrompt("make it blue")
	if !strings.Contains(got, "make it blue") {
		t.Errorf("expected edit prompt, got %q", got)
	}
}

func TestEnrichRestorePrompt(t *testing.T) {
	t.Run("empty prompt", func(t *testing.T) {
		got := EnrichRestorePrompt("")
		if !strings.Contains(got, "Restore") {
			t.Errorf("expected default restore prompt, got %q", got)
		}
	})

	t.Run("custom prompt", func(t *testing.T) {
		got := EnrichRestorePrompt("fix the colors")
		if !strings.Contains(got, "fix the colors") {
			t.Errorf("expected custom restore prompt, got %q", got)
		}
	})
}

func TestEnrichIconPrompt(t *testing.T) {
	got := EnrichIconPrompt("music note", "modern", 256, "transparent", "rounded")
	if !strings.Contains(got, "music note") {
		t.Error("expected prompt text")
	}
	if !strings.Contains(got, "modern") {
		t.Error("expected style")
	}
	if !strings.Contains(got, "256x256") {
		t.Error("expected size")
	}
	if !strings.Contains(got, "transparent") {
		t.Error("expected background")
	}
	if !strings.Contains(got, "ounded") {
		t.Error("expected corners")
	}
}

func TestEnrichPatternPrompt(t *testing.T) {
	got := EnrichPatternPrompt("leaves", "floral", "colorful", "dense", "512x512", "tile")
	if !strings.Contains(got, "leaves") {
		t.Error("expected prompt text")
	}
	if !strings.Contains(got, "floral") {
		t.Error("expected style")
	}
	if !strings.Contains(got, "colorful") {
		t.Error("expected colors")
	}
	if !strings.Contains(got, "dense") {
		t.Error("expected density")
	}
}

func TestEnrichStoryPrompt(t *testing.T) {
	t.Run("first frame", func(t *testing.T) {
		got := EnrichStoryPrompt("cat adventure", 1, 4, "consistent", "smooth")
		if !strings.Contains(got, "frame 1 of 4") {
			t.Error("expected frame number")
		}
		if !strings.Contains(got, "opening scene") {
			t.Error("expected opening scene for first frame")
		}
	})

	t.Run("last frame", func(t *testing.T) {
		got := EnrichStoryPrompt("cat adventure", 4, 4, "consistent", "smooth")
		if !strings.Contains(got, "final scene") {
			t.Error("expected final scene for last frame")
		}
	})

	t.Run("middle frame", func(t *testing.T) {
		got := EnrichStoryPrompt("cat adventure", 2, 4, "evolving", "dramatic")
		if !strings.Contains(got, "scene 2") {
			t.Error("expected scene number for middle frame")
		}
		if !strings.Contains(got, "evolve") {
			t.Error("expected evolving style")
		}
		if !strings.Contains(got, "dramatic") {
			t.Error("expected dramatic transition")
		}
	})
}

func TestEnrichDiagramPrompt(t *testing.T) {
	got := EnrichDiagramPrompt("auth flow", "flowchart", "professional", "hierarchical", "detailed", "accent")
	if !strings.Contains(got, "auth flow") {
		t.Error("expected prompt text")
	}
	if !strings.Contains(got, "flowchart") {
		t.Error("expected diagram type")
	}
	if !strings.Contains(got, "professional") {
		t.Error("expected style")
	}
	if !strings.Contains(got, "hierarchical") {
		t.Error("expected layout")
	}
}
