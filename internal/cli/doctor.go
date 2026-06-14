package cli

import (
	"encoding/json"
	"errors"
	"fmt"

	naba "github.com/dixson3/naba"
	"github.com/dixson3/naba/internal/config"
	"github.com/dixson3/naba/internal/gemini"
	"github.com/spf13/cobra"
)

var (
	doctorScope   string
	doctorSurface string
	doctorTarget  string
)

func init() {
	doctorCmd.Flags().StringVar(&doctorScope, "scope", "user",
		"skills scope to check: user → $HOME; project → git root (else cwd)")
	doctorCmd.Flags().StringVar(&doctorSurface, "surface", "claude",
		"skills surface to check: claude or agents")
	doctorCmd.Flags().StringVar(&doctorTarget, "target", "",
		"explicit skills destination to check (overrides scope/surface)")
	rootCmd.AddCommand(doctorCmd)
}

var doctorCmd = &cobra.Command{
	Use:   "doctor",
	Short: "Check naba's environment health (skills, API key, model, config)",
	Long: "Validate the naba environment: embedded skills installed and matching the " +
		"binary, GEMINI_API_KEY present and live, the configured model reachable, config " +
		"parseable, and the binary version. Exits non-zero if any check fails.",
	Args: cobra.NoArgs,
	RunE: runDoctor,
}

const (
	statusPass = "pass"
	statusWarn = "warn"
	statusFail = "fail"
)

// doctorCheck is one health check result.
type doctorCheck struct {
	Name   string `json:"name"`
	Status string `json:"status"`
	Detail string `json:"detail"`
}

func runDoctor(cmd *cobra.Command, args []string) error {
	return reportDoctor(doctorChecks())
}

// doctorChecks runs every health check and returns the results. It is separated from
// runDoctor so tests can drive it via env (GEMINI_API_KEY / GEMINI_BASE_URL /
// NABA_CONFIG_DIR) and the doctor* destination flags without capturing stdout.
func doctorChecks() []doctorCheck {
	var checks []doctorCheck
	add := func(name, status, detail string) {
		checks = append(checks, doctorCheck{Name: name, Status: status, Detail: detail})
	}

	// 1. Binary version (informational).
	add("version", statusPass, fmt.Sprintf("naba %s (commit %s, built %s)", Version, Commit, Date))

	// 2. Config parseable.
	cfg, cfgErr := config.Load()
	if cfgErr != nil {
		add("config", statusFail, fmt.Sprintf("config not parseable: %v", cfgErr))
		cfg = &config.Config{}
	} else {
		add("config", statusPass, fmt.Sprintf("parseable (%s)", config.ConfigPath()))
	}

	// 3. API key present.
	apiKey := config.ResolveAPIKey()
	if apiKey == "" {
		add("api_key", statusFail, "GEMINI_API_KEY not set (env or config); generation will fail")
	} else {
		add("api_key", statusPass, "present")
	}

	// Resolve the model that would be used by default.
	model, modelErr := cfg.ResolveModel()
	if modelErr != nil {
		add("model_config", statusFail, modelErr.Error())
	}
	if model == "" {
		model = gemini.DefaultModel
	}

	// 4 & 5. Live key check + model reachability (a single models.list call, no image cost).
	if apiKey != "" {
		client := gemini.NewClient(apiKey, model)
		available, listErr := client.ListModels()
		switch {
		case listErr == nil:
			add("api_live", statusPass, "key validated via models.list")
			if gemini.ModelReachable(model, available) {
				add("model_reachable", statusPass, fmt.Sprintf("%q is available", model))
			} else {
				add("model_reachable", statusFail,
					fmt.Sprintf("configured model %q is not in models.list (retired or wrong id)", model))
			}
		case isAuthError(listErr):
			add("api_live", statusFail, fmt.Sprintf("key rejected: %v", listErr))
		default:
			// Network/transient error: degrade to presence-only rather than failing hard.
			add("api_live", statusWarn, fmt.Sprintf("could not reach API (offline?): %v", listErr))
			add("model_reachable", statusWarn, "skipped (API unreachable)")
		}
	}

	// 6. Skills installed and matching the embedded binary.
	dest, destErr := resolveDest(doctorScope, doctorSurface, doctorTarget)
	if destErr != nil {
		add("skills", statusFail, fmt.Sprintf("cannot resolve skills destination: %v", destErr))
	} else {
		names, err := naba.SkillNames()
		if err != nil {
			add("skills", statusFail, fmt.Sprintf("cannot read embedded skills: %v", err))
		}
		for _, name := range names {
			st, err := SkillStatus(name, dest)
			if err != nil {
				add("skills:"+name, statusFail, err.Error())
				continue
			}
			switch {
			case !st.Installed:
				add("skills:"+name, statusFail, fmt.Sprintf("not installed at %s (run: naba skills install)", dest))
			case !st.UpToDate:
				add("skills:"+name, statusFail, "installed copy is outdated vs this binary (run: naba skills upgrade)")
			case !st.Complete:
				add("skills:"+name, statusFail, "installed copy is missing files (run: naba skills upgrade)")
			case !st.Unmodified:
				add("skills:"+name, statusFail, "installed copy was modified since install (run: naba skills upgrade)")
			default:
				add("skills:"+name, statusPass, fmt.Sprintf("installed, up-to-date, complete, unmodified (%s)", dest))
			}
		}
	}

	return checks
}

// isAuthError reports whether err is a Gemini auth failure (key rejected), as opposed to
// a transient/network error.
func isAuthError(err error) bool {
	var apiErr *gemini.APIError
	if errors.As(err, &apiErr) {
		return apiErr.ExitCode == gemini.ExitAuth
	}
	return false
}

// reportDoctor prints the checks (JSON when --json/piped, else human) and returns a
// non-zero exitCodeError if any check failed.
func reportDoctor(checks []doctorCheck) error {
	failed := 0
	for _, c := range checks {
		if c.Status == statusFail {
			failed++
		}
	}

	if flagJSON {
		out := struct {
			OK     bool          `json:"ok"`
			Failed int           `json:"failed"`
			Checks []doctorCheck `json:"checks"`
		}{OK: failed == 0, Failed: failed, Checks: checks}
		data, err := json.MarshalIndent(out, "", "  ")
		if err != nil {
			return exitError(gemini.ExitGeneral, err.Error())
		}
		fmt.Println(string(data))
	} else {
		for _, c := range checks {
			fmt.Printf("[%s] %s: %s\n", doctorSymbol(c.Status), c.Name, c.Detail)
		}
		if failed == 0 {
			fmt.Println("\nAll checks passed.")
		} else {
			fmt.Printf("\n%d check(s) failed.\n", failed)
		}
	}

	if failed > 0 {
		return exitError(gemini.ExitGeneral, fmt.Sprintf("doctor: %d check(s) failed", failed))
	}
	return nil
}

func doctorSymbol(status string) string {
	switch status {
	case statusPass:
		return "✓"
	case statusWarn:
		return "!"
	default:
		return "✗"
	}
}
