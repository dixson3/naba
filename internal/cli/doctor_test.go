package cli

import (
	"net/http"
	"net/http/httptest"
	"testing"
)

// modelsServer returns an httptest server that serves a models.list payload.
func modelsServer(t *testing.T, body string, status int) *httptest.Server {
	t.Helper()
	return httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if status != 0 {
			w.WriteHeader(status)
		}
		_, _ = w.Write([]byte(body))
	}))
}

func findCheck(checks []doctorCheck, name string) (doctorCheck, bool) {
	for _, c := range checks {
		if c.Name == name {
			return c, true
		}
	}
	return doctorCheck{}, false
}

func setDoctorEnv(t *testing.T, baseURL, key string) {
	t.Helper()
	t.Setenv("NABA_CONFIG_DIR", t.TempDir())
	t.Setenv("GEMINI_BASE_URL", baseURL)
	t.Setenv("GEMINI_API_KEY", key)
}

func TestDoctorChecks_AllPass(t *testing.T) {
	srv := modelsServer(t, `{"models":[{"name":"models/gemini-3.1-flash-image"}]}`, 0)
	defer srv.Close()
	setDoctorEnv(t, srv.URL, "test-key")

	// Install skills to a temp dest and point doctor at it.
	dest := t.TempDir()
	deployForTest(t, dest, false)
	prevTarget := doctorTarget
	doctorTarget = dest
	defer func() { doctorTarget = prevTarget }()

	checks := doctorChecks()
	for _, name := range []string{"api_key", "api_live", "model_reachable", "skills:naba", "config"} {
		c, ok := findCheck(checks, name)
		if !ok {
			t.Fatalf("missing check %q", name)
		}
		if c.Status != statusPass {
			t.Errorf("check %q = %s (%s), want pass", name, c.Status, c.Detail)
		}
	}
}

func TestDoctorChecks_ModelUnreachable(t *testing.T) {
	// models.list omits the default model -> model_reachable should fail.
	srv := modelsServer(t, `{"models":[{"name":"models/some-other-model"}]}`, 0)
	defer srv.Close()
	setDoctorEnv(t, srv.URL, "test-key")
	dest := t.TempDir()
	deployForTest(t, dest, false)
	doctorTarget = dest
	defer func() { doctorTarget = "" }()

	checks := doctorChecks()
	c, ok := findCheck(checks, "model_reachable")
	if !ok || c.Status != statusFail {
		t.Errorf("expected model_reachable fail, got %+v (ok=%v)", c, ok)
	}
}

func TestDoctorChecks_AuthFails(t *testing.T) {
	srv := modelsServer(t, `{"error":{"code":401,"message":"bad key"}}`, http.StatusUnauthorized)
	defer srv.Close()
	setDoctorEnv(t, srv.URL, "bad-key")
	doctorTarget = t.TempDir()
	defer func() { doctorTarget = "" }()

	checks := doctorChecks()
	c, ok := findCheck(checks, "api_live")
	if !ok || c.Status != statusFail {
		t.Errorf("expected api_live fail on auth error, got %+v (ok=%v)", c, ok)
	}
}

func TestDoctorChecks_SkillsNotInstalled(t *testing.T) {
	srv := modelsServer(t, `{"models":[{"name":"models/gemini-3.1-flash-image"}]}`, 0)
	defer srv.Close()
	setDoctorEnv(t, srv.URL, "test-key")
	doctorTarget = t.TempDir() // empty -> not installed
	defer func() { doctorTarget = "" }()

	checks := doctorChecks()
	c, ok := findCheck(checks, "skills:naba")
	if !ok || c.Status != statusFail {
		t.Errorf("expected skills:naba fail when not installed, got %+v (ok=%v)", c, ok)
	}
}

func TestDoctorChecks_NoKey(t *testing.T) {
	setDoctorEnv(t, "http://127.0.0.1:0", "")
	doctorTarget = t.TempDir()
	defer func() { doctorTarget = "" }()

	checks := doctorChecks()
	c, ok := findCheck(checks, "api_key")
	if !ok || c.Status != statusFail {
		t.Errorf("expected api_key fail with no key, got %+v (ok=%v)", c, ok)
	}
	// api_live should be skipped entirely when no key is present.
	if _, ok := findCheck(checks, "api_live"); ok {
		t.Error("api_live should be skipped when no key present")
	}
}
