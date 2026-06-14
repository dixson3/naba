// Package naba is the module-root package. Its sole job is to embed the skills/ tree
// into the binary (a //go:embed directive cannot reference a parent directory, so the
// embed must live here rather than under cmd/naba) and to expose a canonical tree-hash
// used by `naba skills` and `naba doctor` to verify a deployed install against the binary.
package naba

import (
	"bytes"
	"crypto/sha256"
	"embed"
	"encoding/hex"
	"fmt"
	"io/fs"
	"os"
	"path/filepath"
	"sort"
	"strings"
)

// skillsFS embeds the skills/ tree. The default embed pattern excludes dotfiles (e.g.
// .gitignore), matching the prior installer's --exclude=.gitignore and keeping the
// canonical tree deterministic.
//
//go:embed skills
var skillsFS embed.FS

// MarkerPrefix opens the hidden integrity marker injected into a deployed SKILL.md.
const MarkerPrefix = "<!-- naba-skills:"

// SkillNames returns the embedded skill names (immediate subdirectories of skills/),
// sorted.
func SkillNames() ([]string, error) {
	entries, err := skillsFS.ReadDir("skills")
	if err != nil {
		return nil, err
	}
	var names []string
	for _, e := range entries {
		if e.IsDir() {
			names = append(names, e.Name())
		}
	}
	sort.Strings(names)
	return names, nil
}

// SkillFiles returns the file paths within an embedded skill, relative to the skill's
// own directory (e.g. "SKILL.md", "commands/edit.md"), sorted.
func SkillFiles(name string) ([]string, error) {
	root := "skills/" + name
	var rels []string
	err := fs.WalkDir(skillsFS, root, func(p string, d fs.DirEntry, err error) error {
		if err != nil {
			return err
		}
		if d.IsDir() {
			return nil
		}
		rel, err := filepath.Rel(root, p)
		if err != nil {
			return err
		}
		rels = append(rels, filepath.ToSlash(rel))
		return nil
	})
	if err != nil {
		return nil, err
	}
	sort.Strings(rels)
	return rels, nil
}

// ReadSkillFile returns the bytes of a file within an embedded skill, addressed by a
// skill-relative path.
func ReadSkillFile(name, rel string) ([]byte, error) {
	return skillsFS.ReadFile("skills/" + name + "/" + filepath.ToSlash(rel))
}

// EmbeddedTreeHash computes the canonical hash of an embedded skill tree (marker-free,
// since the repo source carries no marker). See hashTree for the exact algorithm.
func EmbeddedTreeHash(name string) (string, error) {
	rels, err := SkillFiles(name)
	if err != nil {
		return "", err
	}
	read := func(rel string) ([]byte, error) { return ReadSkillFile(name, rel) }
	return hashTree(rels, read)
}

// DeployedTreeHash computes the canonical hash of a deployed skill directory on disk.
// The marker line is stripped from SKILL.md before hashing so a marked install hashes
// identically to its marker-free embedded source.
func DeployedTreeHash(dir string) (string, error) {
	var rels []string
	err := filepath.WalkDir(dir, func(p string, d fs.DirEntry, err error) error {
		if err != nil {
			return err
		}
		if d.IsDir() {
			return nil
		}
		rel, err := filepath.Rel(dir, p)
		if err != nil {
			return err
		}
		rels = append(rels, filepath.ToSlash(rel))
		return nil
	})
	if err != nil {
		return "", err
	}
	sort.Strings(rels)
	read := func(rel string) ([]byte, error) {
		return os.ReadFile(filepath.Join(dir, filepath.FromSlash(rel)))
	}
	return hashTree(rels, read)
}

// hashTree is the canonical digest: sha256 over, for each file sorted by relative path,
// the relative-path bytes then the file bytes — raw, with no line-ending or
// trailing-newline normalization. SKILL.md has its marker line stripped first so an
// embedded (marker-free) tree and a deployed (marked) tree hash identically.
func hashTree(rels []string, read func(string) ([]byte, error)) (string, error) {
	h := sha256.New()
	for _, rel := range rels {
		data, err := read(rel)
		if err != nil {
			return "", err
		}
		if rel == "SKILL.md" {
			data = StripMarker(data)
		}
		h.Write([]byte(rel))
		h.Write(data)
	}
	return hex.EncodeToString(h.Sum(nil)), nil
}

// StripMarker removes the first anchored `<!-- naba-skills: ... -->` line and its single
// newline terminator, restoring the embedded original byte-for-byte. Content with no
// marker is returned unchanged.
func StripMarker(content []byte) []byte {
	lines := bytes.SplitAfter(content, []byte("\n"))
	for i, line := range lines {
		trimmed := strings.TrimRight(string(line), "\n")
		if strings.HasPrefix(strings.TrimSpace(trimmed), MarkerPrefix) && strings.HasSuffix(trimmed, "-->") {
			out := bytes.Join(append(append([][]byte{}, lines[:i]...), lines[i+1:]...), nil)
			return out
		}
	}
	return content
}

// FormatMarker builds the single-line integrity marker for a version and tree hash.
func FormatMarker(version, treeHash string) string {
	return fmt.Sprintf("%s v=%s tree=%s -->", MarkerPrefix, version, treeHash)
}

// InjectMarker returns SKILL.md content with the integrity marker inserted immediately
// after the closing line of the YAML frontmatter (so it does not break the frontmatter
// parse). Any existing marker is stripped first, making injection idempotent. If no
// frontmatter is present, the marker is prepended.
func InjectMarker(content []byte, marker string) []byte {
	content = StripMarker(content)
	markerLine := append([]byte(marker), '\n')

	// Find the end of the YAML frontmatter (the second "---" line).
	if bytes.HasPrefix(content, []byte("---\n")) {
		rest := content[len("---\n"):]
		if idx := bytes.Index(rest, []byte("\n---\n")); idx >= 0 {
			cut := len("---\n") + idx + len("\n---\n")
			out := make([]byte, 0, len(content)+len(markerLine))
			out = append(out, content[:cut]...)
			out = append(out, markerLine...)
			out = append(out, content[cut:]...)
			return out
		}
	}
	return append(markerLine, content...)
}

// ParseMarkerHash extracts the tree hash from a deployed SKILL.md's marker line, or ""
// when no marker is present.
func ParseMarkerHash(content []byte) string {
	for _, line := range bytes.SplitAfter(content, []byte("\n")) {
		trimmed := strings.TrimSpace(strings.TrimRight(string(line), "\n"))
		if !strings.HasPrefix(trimmed, MarkerPrefix) {
			continue
		}
		for _, field := range strings.Fields(trimmed) {
			if strings.HasPrefix(field, "tree=") {
				return strings.TrimPrefix(field, "tree=")
			}
		}
	}
	return ""
}
