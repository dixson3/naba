# INV-4 — UX regression-suite + SPEC.md strategy

**Scope correction (important):** `storyboard`, `batch`, `brand-kit` are **NOT CLI
subcommands** — they are **skill-layer composites** (`AGENTS.md:63-64`,
`README.md:255-260`): the `/naba` skill dispatches a subagent that orchestrates
multiple real CLI calls. The binary parity surface is **12 real command groups**,
not the 15 the plan's scope #1 implied. Actual `rootCmd.AddCommand` tree:
`generate, edit, restore, icon, pattern, diagram, story, config (get/set), doctor,
skills (install/upgrade/remove/status), mcp, version` (+ root help/no-args). Also
`skills` is a real subcommand the plan omitted. **Fix scope #1 before PLAN.**

## Recommended harness: Python + pytest (language-neutral black-box)

`$NABA_BIN` selects the binary (Go build for golden capture, Rust build for
replay); CI runs the same suite twice. Rejected alternatives: Rust
`assert_cmd`/`insta` (violates the Go-capture-first rider; can't run before Rust
exists — allowed only as a supplementary Rust-internal unit layer); `bats` (weak at
structural JSON + mock server; OK as a thin optional smoke layer, not the contract).

pytest wins on: structural JSON diffing, nondeterministic-field normalization,
first-class local HTTP mock (`pytest-httpserver`), PTY support (`pty`/`pexpect`) for
TTY cases, table-driven `parametrize`, and an official MCP Python SDK for the MCP
harness.

### Directory layout
```
tests/parity/
  conftest.py  runner.py  mock_provider.py
  cases/  golden/  fixtures/{images,configs}
  test_cli.py  test_tty.py  test_preview.py  test_mcp.py  test_migration.py
```
`--update-golden` runs against the Go binary and writes normalized goldens; plain
`pytest` (Rust) asserts equality. Each case in isolated tmp CWD + per-case
`NABA_CONFIG_DIR`.

## Mocking the provider

Real local HTTP server (separate process → in-process mocking impossible). Point
binary via `GEMINI_BASE_URL` (already honored) + the port's NEW `OPENROUTER_BASE_URL`.
Gemini routes: `POST /models/{model}:generateContent`, `GET /models?pageSize=1000`.
OpenRouter route (per INV-2): `POST /api/v1/images` (Bearer). Return a tiny canned
base64 image; return `image/jpeg` to exercise `.png→.jpg` reconciliation +
`requested_format`/`actual_format` divergence. The mock **records inbound requests**
so tests assert the OUTGOING JSON (enriched prompt, `imageConfig`,
`responseModalities`) against golden *request* fixtures — pins the wire contract
across the language switch.

Response profiles for the exit matrix: 200+image; 200 no-image (→5); blockReason
(→5); 401/403 (→3+hint); 429 (→4); 500 (→5+hint).

## Cannot be black-boxed (out-of-band)

- **TTY/auto-JSON**: piped subprocess is always non-TTY → always auto-JSON. Human
  branch + the auto-enable logic need a **PTY** subset (`pexpect`). Two runner
  modes: piped (JSON goldens, bulk) + pty (human goldens, targeted).
- **`--preview`**: shells `open`/`xdg-open`/`start`. Use a **PATH stub** that logs
  args; assert invocation, never launch. Rust must resolve opener via PATH identically.
- **MCP server**: stdio JSON-RPC — **separate** harness (`test_mcp.py`, MCP Python
  SDK): `initialize` → `tools/list` (assert 8 tools + params/enums/defaults from
  `tools.go`) → `tools/call` (mock provider) → `resources/read` (`file://` template).
- **`doctor`/`skills`**: embedded-skill integrity hashes are Go-impl-specific — pin
  **check semantics + JSON envelope**, NOT raw hashes. `doctor` network call mocked.
- **`version`**: ldflags → regex assert.
- **`--help`**: cobra vs clap formatting diverges (sanctioned) → pin inventory via
  contains/regex, not full snapshot.

## Exit-code subtlety (must pin)

`main.go`: exit = `err.ExitCode()` if implemented, **else 1**. So **cobra
arg-count/parse errors (ExactArgs, unknown flag/command) → exit 1, NOT 2.** Only
explicitly-constructed `exitError(ExitUsage,…)` yields 2 (invalid
aspect/resolution/quality enum, `story --steps` out of range, `config set` unknown
key). Validation enums are **case-sensitive** ("1k" rejected).

## Precedence + config facts to pin

- Model: `--model` > `--quality` > config `model` > config `quality` > `DefaultModel`
  (`gemini-3.1-flash-image`), via cobra `Changed()`.
- quality→model: `fast`→`gemini-3.1-flash-image`, `high`→`gemini-3-pro-image` —
  **duplicated** in `gemini.ModelForQuality` AND `config.modelForQuality` (lockstep
  hazard; SPEC pins values, port collapses duplication).
- Output dir asymmetry: `NABA_OUTPUT_DIR` affects only the **MCP server**; CLI
  commands write to `flagOutput` or CWD. Pin this.
- Env vars: `GEMINI_API_KEY`, `GEMINI_BASE_URL`, `NABA_CONFIG_DIR`, `NABA_OUTPUT_DIR`.

## SPEC.md outline (stable clause IDs `SPEC-<AREA>-NNN`, CI-enforced traceability)

1. Overview/scope (12 command groups; storyboard/batch/brand-kit = skill composites,
   out of binary scope). 2. Invocation model + TTY auto-behavior. 3. Per-command
reference. 4. Configuration (schema, precedence, quality→model, validation,
CLI-vs-MCP output-dir asymmetry). 5. Provider layer (port addition: selection
precedence, `--model` requires `--provider`, env autodetect, `*_BASE_URL`). 6.
Output contract (Result/array shapes, doctor envelope, nondeterministic-field
policy). 7. Exit-code contract (incl. cobra-parse→1 subtlety). 8. Error messages
(verbatim). 9. MCP surface. 10. Config auto-migration (port addition). 11.
Sanctioned divergences (help text, skill hashes, version — inventory-pinned). 12.
Traceability table (SPEC-ID ↔ case-id, CI fails on orphans either direction).

## Config auto-migration tests (Rust-only, spec-driven — no Go golden)

Fixture pairs `old-config.yaml → expected-migrated.yaml`. Assert: (1) no data loss
(all 6 Go keys preserved); (2) new `provider`/fields added w/ correct
defaults/inference; (3) original backed up (`.bak`) unchanged; (4) idempotent
(second run no re-migrate); (5) edges (empty/missing/already-new/malformed → graceful,
no partial write); (6) trigger on any `config.Load()`; `config get <oldkey>` returns
preserved value.

## Implications for the plan

- **Fix scope #1**: 12 command groups; storyboard/batch/brand-kit are skill composites.
- **Port must add `OPENROUTER_BASE_URL`** early (hard prereq for deterministic
  mocked OpenRouter tests) — belongs in an early implementation epic.
- Suite ordering validates the plan's UX riders: SPEC.md → pytest harness →
  Go-capture goldens (before any Rust) → implement Rust → replay.
- Enumerate sanctioned-divergence zones in SPEC (help formatting, skill hashes,
  version) — byte-identical goldens there = false failures.

Source: `internal/gemini/client.go` (parseAPIError, exit consts),
`internal/gemini/imageconfig.go`, `internal/cli/imageopts.go`,
`internal/config/config.go`, `internal/output/{json,writer}.go`,
`internal/cli/root.go`, `internal/mcp/tools.go`.
