# naba — Exit Codes & Errors Specification

Clause IDs (`SPEC-<AREA>-NNN`) are stable and are never renumbered; append only.

## §7 Exit-code matrix (SPEC-EXIT)

- **SPEC-EXIT-001** [PINNED] Exit codes: `1` General, `2` Usage, `3` Auth, `4` RateLimit,
  `5` API, `10` FileIO.
- **SPEC-EXIT-002** [PINNED] Dispatch: on error the top-level handler prints the error to
  stderr and exits with the error's `ExitCode()` if it implements one, else **1**.
- **SPEC-EXIT-003** [PINNED] **Cobra/clap parse errors exit 1**, not 2. With
  `SilenceErrors/SilenceUsage`, a flag/arg parse error has no `ExitCode()` and falls to the
  default 1. Only explicit in-code usage errors exit 2 (e.g. `steps must be between 2 and
  8`, `unknown key`, invalid aspect/resolution/quality, `--model` without `--provider`).
  The Rust port must replicate: argument-parse failures exit **1**, not clap's default 2.
- **SPEC-EXIT-004** [PINNED] HTTP→exit mapping (Gemini): 401/403 → 3 (Auth); 429 → 4
  (RateLimit); ≥500 → 5 (API, message rewritten); other non-2xx → 5. Prompt-block / no-image
  → 5. Input-image read failure → 10 (FileIO). OpenRouter maps analogously: 401/403 → 3,
  429 → 4 (honoring `Retry-After`), moderation-403/content-policy → 3-or-5 per §ERR, ≥500 →
  5.
- **SPEC-EXIT-005** [PINNED] `doctor` with any failing check exits **1** (not 2), message
  `doctor: %d check(s) failed`.

---

## §9 Verbatim error strings (SPEC-ERR)

All [PINNED] unless the wording is provider-dependent (marked [DIVERGENCE]).

- **SPEC-ERR-001** API key unset (CLI image cmds): `GEMINI_API_KEY not set.\n\nSet it with:
  export GEMINI_API_KEY=<your-key>\nOr run: naba config set api_key <your-key>` → exit 3.
  [DIVERGENCE] under multi-provider the message names the selected provider's key
  (`OPENROUTER_API_KEY` when the provider is openrouter). The suite pins exit 3 + the
  "not set" shape, not the exact key name for the openrouter case.
- **SPEC-ERR-002** Input file missing (edit/restore): `input file not found: %s` → exit 10.
- **SPEC-ERR-003** story steps: `steps must be between 2 and 8` → exit 2.
- **SPEC-ERR-004** invalid aspect: `invalid aspect ratio %q\n\nValid values: <list>` → 2.
- **SPEC-ERR-005** invalid resolution: `invalid resolution %q\n\nValid values: <list>` → 2.
- **SPEC-ERR-006** invalid quality (flag/MCP): `invalid quality %q\n\nValid values: fast,
  high` → 2.
- **SPEC-ERR-007** invalid quality (config): `invalid quality %q in config (valid: fast,
  high)`.
- **SPEC-ERR-008** config get unset: `key %q is not set\n\nValid keys: <list>` → 1.
- **SPEC-ERR-009** config set unknown key: `unknown key %q\n\nValid keys: <list>` → 2.
- **SPEC-ERR-010** Gemini auth (401/403): `authentication failed: %s\n\nSet GEMINI_API_KEY
  or run: naba config set api_key <your-key>` → 3.
- **SPEC-ERR-011** rate limit (429): `rate limit exceeded: %s\n\nWait a moment and try
  again.` → 4.
- **SPEC-ERR-012** server (≥500): `Gemini server error: %s\n\nThis is a temporary issue. Try
  again shortly.` → 5. [DIVERGENCE] OpenRouter uses an analogous provider-named string.
- **SPEC-ERR-013** prompt blocked: `prompt blocked: %s` → 5.
- **SPEC-ERR-014** no images: `no images in response` → 5.
- **SPEC-ERR-015** read image file: `read image file %q: %v` → 10.
- **SPEC-ERR-016** [NEW] `--model` without `--provider`: usage error → exit 2.
- **SPEC-ERR-017** [NEW] OpenRouter moderation/content-policy (403): a content-policy error
  string → exit 3 (auth-class) or 5 per the live-key smoke; `Retry-After` honored on 429.
