package cli

import (
	"fmt"
	"io/fs"
	"os"
	"os/exec"
	"path/filepath"
	"strings"

	naba "github.com/dixson3/naba"
	"github.com/dixson3/naba/internal/gemini"
	"github.com/spf13/cobra"
)

var (
	skillsScope   string
	skillsSurface string
	skillsTarget  string
	skillsDryRun  bool
)

func init() {
	skillsCmd.PersistentFlags().StringVar(&skillsScope, "scope", "user",
		"user → $HOME; project → git root (else cwd)")
	skillsCmd.PersistentFlags().StringVar(&skillsSurface, "surface", "claude",
		"claude → <root>/.claude/skills; agents → <root>/.agents/skills")
	skillsCmd.PersistentFlags().StringVar(&skillsTarget, "target", "",
		"override skills destination directory (takes precedence over scope/surface)")
	skillsCmd.PersistentFlags().BoolVar(&skillsDryRun, "dry-run", false,
		"print the actions that would be taken; change nothing")
	skillsCmd.AddCommand(skillsInstallCmd, skillsUpgradeCmd, skillsRemoveCmd, skillsStatusCmd)
	rootCmd.AddCommand(skillsCmd)
}

var skillsCmd = &cobra.Command{
	Use:   "skills",
	Short: "Install, upgrade, remove, or check naba's binary-embedded skills",
	Long: "Manage the naba skill files that ship embedded in this binary (offline, " +
		"version-matched). Supersedes the legacy install.sh/install.py.",
}

var skillsInstallCmd = &cobra.Command{
	Use:   "install",
	Short: "Install embedded skills to the resolved destination",
	Args:  cobra.NoArgs,
	RunE:  func(cmd *cobra.Command, args []string) error { return runSkills(deployInstall) },
}

var skillsUpgradeCmd = &cobra.Command{
	Use:   "upgrade",
	Short: "Rewrite installed skills from the embedded tree and prune stale files",
	Args:  cobra.NoArgs,
	RunE:  func(cmd *cobra.Command, args []string) error { return runSkills(deployUpgrade) },
}

var skillsRemoveCmd = &cobra.Command{
	Use:   "remove",
	Short: "Remove installed skills from the destination",
	Args:  cobra.NoArgs,
	RunE:  func(cmd *cobra.Command, args []string) error { return runSkills(deployRemove) },
}

var skillsStatusCmd = &cobra.Command{
	Use:   "status",
	Short: "Report whether installed skills are up-to-date, complete, and unmodified",
	Args:  cobra.NoArgs,
	RunE: func(cmd *cobra.Command, args []string) error {
		dest, err := resolveSkillsDest()
		if err != nil {
			return exitError(gemini.ExitGeneral, err.Error())
		}
		names, err := naba.SkillNames()
		if err != nil {
			return exitError(gemini.ExitGeneral, err.Error())
		}
		for _, name := range names {
			st, err := SkillStatus(name, dest)
			if err != nil {
				return exitError(gemini.ExitGeneral, err.Error())
			}
			fmt.Println(st.Line(dest))
		}
		return nil
	},
}

type deployMode int

const (
	deployInstall deployMode = iota
	deployUpgrade
	deployRemove
)

func runSkills(mode deployMode) error {
	dest, err := resolveSkillsDest()
	if err != nil {
		return exitError(gemini.ExitGeneral, err.Error())
	}
	names, err := naba.SkillNames()
	if err != nil {
		return exitError(gemini.ExitGeneral, err.Error())
	}
	for _, name := range names {
		switch mode {
		case deployRemove:
			if err := removeSkill(name, dest); err != nil {
				return exitError(gemini.ExitFileIO, err.Error())
			}
		default:
			if err := deploySkill(name, dest, mode == deployUpgrade); err != nil {
				return exitError(gemini.ExitFileIO, err.Error())
			}
		}
	}
	if !skillsDryRun && !flagQuiet {
		fmt.Printf("Destination: %s\n", dest)
	}
	return nil
}

// resolveSkillsDest resolves the destination from the `naba skills` flags.
func resolveSkillsDest() (string, error) {
	return resolveDest(skillsScope, skillsSurface, skillsTarget)
}

// resolveDest mirrors the legacy installer's destination resolution: an explicit target
// wins; otherwise the anchor is $HOME (user scope) or the git root / cwd (project scope),
// joined with .<surface>/skills. Shared by `naba skills` and `naba doctor`.
func resolveDest(scope, surface, target string) (string, error) {
	if target != "" {
		return target, nil
	}
	var anchor string
	if scope == "project" {
		anchor = gitRootOrCwd()
	} else {
		home, err := os.UserHomeDir()
		if err != nil {
			return "", err
		}
		anchor = home
	}
	return filepath.Join(anchor, "."+surface, "skills"), nil
}

func gitRootOrCwd() string {
	if out, err := exec.Command("git", "rev-parse", "--show-toplevel").Output(); err == nil {
		return strings.TrimSpace(string(out))
	}
	wd, _ := os.Getwd()
	return wd
}

// deploySkill writes an embedded skill's tree to <skillsDest>/<name>/, injecting a fresh
// integrity marker into SKILL.md. With prune=true (upgrade) it also removes dest files
// absent from the embed (rsync --delete parity). Injection strips any existing marker
// first, so it is idempotent and never double-marks.
func deploySkill(name, skillsDest string, prune bool) error {
	destDir := filepath.Join(skillsDest, name)
	rels, err := naba.SkillFiles(name)
	if err != nil {
		return err
	}
	hash, err := naba.EmbeddedTreeHash(name)
	if err != nil {
		return err
	}
	marker := naba.FormatMarker(Version, hash)

	if skillsDryRun {
		fmt.Printf("(dry run) would write %d file(s) of %q -> %s\n", len(rels), name, destDir)
		if prune {
			fmt.Println("(dry run) would prune dest files absent from the embed")
		}
		return nil
	}

	for _, rel := range rels {
		data, err := naba.ReadSkillFile(name, rel)
		if err != nil {
			return err
		}
		if rel == "SKILL.md" {
			data = naba.InjectMarker(data, marker)
		}
		dest := filepath.Join(destDir, filepath.FromSlash(rel))
		if err := os.MkdirAll(filepath.Dir(dest), 0o755); err != nil {
			return err
		}
		if err := os.WriteFile(dest, data, 0o644); err != nil {
			return err
		}
	}
	if prune {
		if err := pruneStale(name, destDir); err != nil {
			return err
		}
	}
	if !flagQuiet {
		fmt.Printf("OK: %s -> %s (%d files)\n", name, destDir, len(rels))
	}
	return nil
}

// pruneStale removes files under destDir that are not part of the embedded skill tree.
func pruneStale(name, destDir string) error {
	rels, err := naba.SkillFiles(name)
	if err != nil {
		return err
	}
	want := make(map[string]bool, len(rels))
	for _, r := range rels {
		want[r] = true
	}
	return filepath.WalkDir(destDir, func(p string, d fs.DirEntry, err error) error {
		if err != nil || d.IsDir() {
			return err
		}
		rel, err := filepath.Rel(destDir, p)
		if err != nil {
			return err
		}
		if !want[filepath.ToSlash(rel)] {
			if err := os.Remove(p); err != nil {
				return err
			}
			if !flagQuiet {
				fmt.Printf("  pruned stale: %s\n", filepath.ToSlash(rel))
			}
		}
		return nil
	})
}

func removeSkill(name, skillsDest string) error {
	destDir := filepath.Join(skillsDest, name)
	if _, err := os.Stat(destDir); os.IsNotExist(err) {
		if !flagQuiet {
			fmt.Printf("absent: %s\n", destDir)
		}
		return nil
	}
	if skillsDryRun {
		fmt.Printf("(dry run) would remove %s\n", destDir)
		return nil
	}
	if err := os.RemoveAll(destDir); err != nil {
		return err
	}
	if !flagQuiet {
		fmt.Printf("removed: %s\n", destDir)
	}
	return nil
}

// SkillStatusResult reports the health of one deployed skill against the binary's embed.
type SkillStatusResult struct {
	Name       string
	Installed  bool // SKILL.md present at the destination
	UpToDate   bool // deployed marker's tree hash == this binary's embedded hash
	Complete   bool // every embedded file is present at the destination
	Unmodified bool // recomputed deployed hash (marker stripped) == embedded hash
}

// OK reports whether the skill is installed, current, complete, and unmodified.
func (s SkillStatusResult) OK() bool {
	return s.Installed && s.UpToDate && s.Complete && s.Unmodified
}

// Line renders a one-line human summary of the status.
func (s SkillStatusResult) Line(dest string) string {
	path := filepath.Join(dest, s.Name)
	if !s.Installed {
		return fmt.Sprintf("%s: not installed (%s)", s.Name, path)
	}
	var flags []string
	flags = append(flags, boolFlag("up-to-date", s.UpToDate))
	flags = append(flags, boolFlag("complete", s.Complete))
	flags = append(flags, boolFlag("unmodified", s.Unmodified))
	return fmt.Sprintf("%s: %s (%s)", s.Name, strings.Join(flags, " "), path)
}

func boolFlag(label string, ok bool) string {
	if ok {
		return "✓" + label
	}
	return "✗" + label
}

// SkillStatus computes the deployment status of an embedded skill at skillsDest. It is
// shared by `naba skills status` and `naba doctor`.
func SkillStatus(name, skillsDest string) (SkillStatusResult, error) {
	st := SkillStatusResult{Name: name}
	destDir := filepath.Join(skillsDest, name)

	content, err := os.ReadFile(filepath.Join(destDir, "SKILL.md"))
	if os.IsNotExist(err) {
		return st, nil
	}
	if err != nil {
		return st, err
	}
	st.Installed = true

	embHash, err := naba.EmbeddedTreeHash(name)
	if err != nil {
		return st, err
	}
	st.UpToDate = naba.ParseMarkerHash(content) == embHash

	rels, err := naba.SkillFiles(name)
	if err != nil {
		return st, err
	}
	st.Complete = true
	for _, rel := range rels {
		if _, err := os.Stat(filepath.Join(destDir, filepath.FromSlash(rel))); err != nil {
			st.Complete = false
			break
		}
	}

	if st.Complete {
		dh, err := naba.DeployedTreeHash(destDir)
		if err != nil {
			return st, err
		}
		st.Unmodified = dh == embHash
	}
	return st, nil
}
