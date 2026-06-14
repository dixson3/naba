package naba

import (
	"bytes"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestEmbeddedSkillsPresent(t *testing.T) {
	names, err := SkillNames()
	if err != nil {
		t.Fatal(err)
	}
	if len(names) == 0 {
		t.Fatal("no embedded skills")
	}
	files, err := SkillFiles("naba")
	if err != nil {
		t.Fatal(err)
	}
	var hasSkillMd bool
	for _, f := range files {
		if f == "SKILL.md" {
			hasSkillMd = true
		}
	}
	if !hasSkillMd {
		t.Error("embedded naba skill missing SKILL.md")
	}
}

func TestRepoSourceMarkerFree(t *testing.T) {
	content, err := ReadSkillFile("naba", "SKILL.md")
	if err != nil {
		t.Fatal(err)
	}
	if bytes.Contains(content, []byte(MarkerPrefix)) {
		t.Error("embedded (repo source) SKILL.md must not carry the integrity marker")
	}
}

func TestMarkerRoundTrip(t *testing.T) {
	orig, err := ReadSkillFile("naba", "SKILL.md")
	if err != nil {
		t.Fatal(err)
	}
	hash, err := EmbeddedTreeHash("naba")
	if err != nil {
		t.Fatal(err)
	}
	marker := FormatMarker("9.9.9", hash)
	injected := InjectMarker(orig, marker)

	if !bytes.Contains(injected, []byte(MarkerPrefix)) {
		t.Fatal("marker not injected")
	}
	// Frontmatter must still parse: the marker is placed after the closing --- line.
	if !bytes.HasPrefix(injected, []byte("---\n")) {
		t.Fatal("frontmatter prefix lost")
	}
	rest := injected[len("---\n"):]
	if !bytes.Contains(rest[:bytes.Index(rest, []byte("\n---\n"))], []byte("name: naba")) {
		t.Error("marker appears to have landed inside the frontmatter block")
	}
	// Strip restores the original byte-for-byte.
	if !bytes.Equal(StripMarker(injected), orig) {
		t.Fatal("strip(inject(x)) != x")
	}
	if ParseMarkerHash(injected) != hash {
		t.Errorf("ParseMarkerHash = %q, want %q", ParseMarkerHash(injected), hash)
	}
	// Injection is idempotent (no double marker).
	double := InjectMarker(injected, FormatMarker("1.0.0", hash))
	if strings.Count(string(double), MarkerPrefix) != 1 {
		t.Errorf("expected exactly one marker after double-inject, got %d", strings.Count(string(double), MarkerPrefix))
	}
	if !bytes.Equal(StripMarker(double), orig) {
		t.Error("double-inject then strip != original")
	}
}

func TestDeployedHashEqualsEmbedded(t *testing.T) {
	hash, err := EmbeddedTreeHash("naba")
	if err != nil {
		t.Fatal(err)
	}
	dir := t.TempDir()
	rels, _ := SkillFiles("naba")
	marker := FormatMarker("1.2.3", hash)
	for _, rel := range rels {
		b, err := ReadSkillFile("naba", rel)
		if err != nil {
			t.Fatal(err)
		}
		if rel == "SKILL.md" {
			b = InjectMarker(b, marker)
		}
		p := filepath.Join(dir, filepath.FromSlash(rel))
		if err := os.MkdirAll(filepath.Dir(p), 0o755); err != nil {
			t.Fatal(err)
		}
		if err := os.WriteFile(p, b, 0o644); err != nil {
			t.Fatal(err)
		}
	}
	dh, err := DeployedTreeHash(dir)
	if err != nil {
		t.Fatal(err)
	}
	if dh != hash {
		t.Errorf("deployed (marked) hash %s != embedded %s", dh, hash)
	}

	// Tamper a non-SKILL file -> hash diverges.
	if err := os.WriteFile(filepath.Join(dir, "README.md"), []byte("changed"), 0o644); err != nil {
		t.Fatal(err)
	}
	if dh2, _ := DeployedTreeHash(dir); dh2 == hash {
		t.Error("tampered tree should not match embedded hash")
	}
}

func TestStripMarker_NoMarker(t *testing.T) {
	in := []byte("no marker here\nline two\n")
	if !bytes.Equal(StripMarker(in), in) {
		t.Error("StripMarker should be a no-op when no marker present")
	}
}
