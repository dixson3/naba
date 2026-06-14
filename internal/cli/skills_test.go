package cli

import (
	"os"
	"path/filepath"
	"testing"

	naba "github.com/dixson3/naba"
)

// withQuietInstall runs deploySkill into dest with output suppressed.
func deployForTest(t *testing.T, dest string, prune bool) {
	t.Helper()
	prevQuiet, prevDry := flagQuiet, skillsDryRun
	flagQuiet, skillsDryRun = true, false
	defer func() { flagQuiet, skillsDryRun = prevQuiet, prevDry }()
	if err := deploySkill("naba", dest, prune); err != nil {
		t.Fatalf("deploySkill: %v", err)
	}
}

func TestSkillsInstallStatusRemove(t *testing.T) {
	dest := t.TempDir()
	deployForTest(t, dest, false)

	// SKILL.md present at destination, marker injected, source still marker-free.
	deployed, err := os.ReadFile(filepath.Join(dest, "naba", "SKILL.md"))
	if err != nil {
		t.Fatal(err)
	}
	if naba.ParseMarkerHash(deployed) == "" {
		t.Error("deployed SKILL.md missing integrity marker")
	}

	st, err := SkillStatus("naba", dest)
	if err != nil {
		t.Fatal(err)
	}
	if !st.OK() {
		t.Errorf("expected healthy status, got %+v", st)
	}

	// Remove and confirm gone.
	prevQuiet := flagQuiet
	flagQuiet = true
	defer func() { flagQuiet = prevQuiet }()
	if err := removeSkill("naba", dest); err != nil {
		t.Fatal(err)
	}
	if _, err := os.Stat(filepath.Join(dest, "naba")); !os.IsNotExist(err) {
		t.Error("skill dir should be removed")
	}
}

func TestSkillStatus_Tampered(t *testing.T) {
	dest := t.TempDir()
	deployForTest(t, dest, false)

	// Append to a deployed file -> unmodified should flip false.
	p := filepath.Join(dest, "naba", "commands", "edit.md")
	f, err := os.OpenFile(p, os.O_APPEND|os.O_WRONLY, 0o644)
	if err != nil {
		t.Fatal(err)
	}
	_, _ = f.WriteString("\ntampered\n")
	_ = f.Close()

	st, err := SkillStatus("naba", dest)
	if err != nil {
		t.Fatal(err)
	}
	if !st.Installed || !st.UpToDate || !st.Complete {
		t.Errorf("install/up-to-date/complete should hold, got %+v", st)
	}
	if st.Unmodified {
		t.Error("tampered install should report Unmodified=false")
	}
	if st.OK() {
		t.Error("tampered install should not be OK")
	}
}

func TestSkillStatus_NotInstalled(t *testing.T) {
	dest := t.TempDir()
	st, err := SkillStatus("naba", dest)
	if err != nil {
		t.Fatal(err)
	}
	if st.Installed || st.OK() {
		t.Errorf("expected not installed, got %+v", st)
	}
}

func TestSkillStatus_Incomplete(t *testing.T) {
	dest := t.TempDir()
	deployForTest(t, dest, false)
	// Delete a non-SKILL.md file -> complete should flip false (SKILL.md still present).
	if err := os.Remove(filepath.Join(dest, "naba", "commands", "icon.md")); err != nil {
		t.Fatal(err)
	}
	st, err := SkillStatus("naba", dest)
	if err != nil {
		t.Fatal(err)
	}
	if st.Complete {
		t.Error("missing file should report Complete=false")
	}
}

func TestDeploySkill_UpgradePrunesStale(t *testing.T) {
	dest := t.TempDir()
	deployForTest(t, dest, false)
	stale := filepath.Join(dest, "naba", "commands", "STALE.md")
	if err := os.WriteFile(stale, []byte("old"), 0o644); err != nil {
		t.Fatal(err)
	}
	deployForTest(t, dest, true) // upgrade with prune
	if _, err := os.Stat(stale); !os.IsNotExist(err) {
		t.Error("upgrade should prune stale file")
	}
	// And the install is still healthy after prune.
	st, _ := SkillStatus("naba", dest)
	if !st.OK() {
		t.Errorf("post-prune status not OK: %+v", st)
	}
}

func TestResolveDest(t *testing.T) {
	// target wins outright
	got, err := resolveDest("user", "claude", "/explicit/dir")
	if err != nil || got != "/explicit/dir" {
		t.Fatalf("target should win: %q %v", got, err)
	}
	// user scope -> $HOME/.<surface>/skills
	home, _ := os.UserHomeDir()
	got, err = resolveDest("user", "agents", "")
	if err != nil {
		t.Fatal(err)
	}
	if want := filepath.Join(home, ".agents", "skills"); got != want {
		t.Errorf("got %q want %q", got, want)
	}
}
