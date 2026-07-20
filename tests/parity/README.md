# naba parity harness (`tests/parity/`)

Black-box regression harness for the shipped **naba Rust CLI**. It drives the binary under
test as an opaque process and inspects only observable behavior — stdout, stderr, exit
code, files written, and the outgoing HTTP requests captured by a mock provider. It imports
**no** naba internals, so it is a pure golden/behavioral suite over the binary.

This directory delivers the **infrastructure** (Issue 1.2). The case table
(`cases/*.yaml`, Issue 1.3) and captured goldens (Issue 1.4) are separate; the runner is
designed so a later case table can drive it (map a case row onto `NabaRunner.run` kwargs).

## Running the suite

The harness is a `uv`-managed Python project pinned by `pyproject.toml`. `uv` creates the
venv on first run.

```bash
# Build the shipped Rust binary the suite runs against:
make build                       # from the repo root -> ./naba (RUST, shipped)

# Run the whole suite (from this directory):
cd tests/parity
uv run pytest -x

# ...or from the repo root, pointing uv at this project:
uv run --project tests/parity pytest tests/parity -x

# ...or via the Makefile helper (build + run against the Rust binary):
make parity
```

Smoke-test-only run: `uv run pytest test_harness.py -x`.

## Selecting the binary: `$NABA_BIN`

The runner picks the binary under test from the `NABA_BIN` environment variable. When it
is unset, it defaults to `./naba` at the repo root — the shipped **Rust** binary
(`make build`).

```bash
# Test the shipped Rust build (default ./naba):
NABA_BIN=/path/to/target/release/naba uv run pytest
```

The `naba_bin` fixture fails fast with a clear message if the binary is missing.

## SPEC ↔ test traceability check (`check_traceability.py`)

`check_traceability.py` (Issue 5.3) asserts that every **[PINNED]** and **[NEW]** clause in
`SPEC.md` maps to at least one test — a parity case (`cases/*.yaml` `spec:` field), a pytest
module that cites the clause id (`test_mcp.py` / `test_parity.py` / `test_harness.py`), or a
justified exemption in `traceability_exemptions.yaml`. It exits non-zero (listing the
uncovered clauses) if coverage is incomplete, and is wired into CI.

```bash
# From the repo root:
uv run tests/parity/check_traceability.py          # human report
uv run tests/parity/check_traceability.py --json    # machine-readable
```

The script has PEP 723 inline deps (`pyyaml`), so `uv run` needs no project. Coverage as of
Issues 1.3/1.4/4.5: 120 clauses total (117 PINNED/NEW required, 3 DIVERGENCE) — most covered
by parity cases, the MCP surface credited to `test_mcp.py`, and the remainder carried by
`traceability_exemptions.yaml`.

**Exemptions** (`traceability_exemptions.yaml`) each carry a concrete reason and cover clauses
verified **outside** the case YAMLs: MCP tool schemas asserted structurally by `test_mcp.py`
(without a literal id cite); Rust-only machinery exercised by **cargo unit tests**
(config migration `SPEC-MIGRATE-*`, provider selection `SPEC-PROVIDER-006/008`, moderation
mapping `SPEC-ERR-017`); help-prose clauses whose wording is `[SPEC-DIVERGE-001]` (pinned by
flag inventory, not snapshots); and the three `[DIVERGENCE]` clauses (pinned at the semantics
level by doctor/skills/version cases). The checker rejects a blank reason or an exemption that
names a non-existent clause.

### Rust divergence cases

Cases that assert a sanctioned Rust-port divergence — clap parse-error wording
(`[SPEC-DIVERGE-001]`), the `provider` key added to the valid-keys list (`[SPEC-DIVERGE-004]`),
or the model-aware `512` rejection (`[SPEC-IMG-007]`) — carry `requires: [provider]`. That
marker historically let them skip on the (now-retired) Go baseline; on the shipped Rust binary
the `--provider` capability is present, so they are **active**. Run the suite as CI does:

```bash
NABA_BIN="$PWD/target/release/naba"    uv run --project tests/parity pytest tests/parity
```

## Fixture inventory (`conftest.py`)

| Fixture         | Scope    | Purpose |
|:----------------|:---------|:--------|
| `naba_bin`      | session  | Resolves `$NABA_BIN` (the built `naba` binary) and asserts it exists. |
| `runner`        | function | A `NabaRunner` bound to `naba_bin`. |
| `config_dir`    | function | Isolated temp `NABA_CONFIG_DIR`. |
| `output_dir`    | function | Isolated temp `NABA_OUTPUT_DIR` (MCP output path; SPEC-CFGSCHEMA-005). |
| `work_cwd`      | function | Isolated temp CWD for CLI auto-named output files. |
| `provider_mock` | function | Recording Gemini + OpenRouter mock over `pytest-httpserver`. |
| `preview_stub`  | function | PATH-stub faking `open`/`xdg-open`/`start` that records instead of launching a viewer. |

## Harness modules (`harness/`)

- **`runner.py`** — `NabaRunner` / `RunResult`. Invokes the binary (pipes, i.e. non-TTY),
  captures stdout/stderr/exit code, and per-case sets CWD, `NABA_CONFIG_DIR`,
  `NABA_OUTPUT_DIR`, `GEMINI_BASE_URL`/`OPENROUTER_BASE_URL`, and API keys. Host-provided
  provider env is scrubbed so cases are hermetic.
- **`pty_runner.py`** — `run_pty(...)`. Runs the binary under a pseudo-terminal so stdout
  is a chardevice, exercising the TTY branch of SPEC-GLOBAL-003 (piped forces `--json`;
  PTY does not). stdout+stderr merge onto the single PTY stream.
- **`mock_provider.py`** — `ProviderMock`. Serves and **records**:
  - Gemini `POST /models/{model}:generateContent` -> canned inline-data PNG.
  - Gemini `GET /models?pageSize=1000` -> two-model list (doctor / list_models).
  - OpenRouter `POST /api/v1/images` -> canned `data[].b64_json` PNG (the OpenRouter provider path).

  Every request's method, path, query, headers, and parsed JSON body are recorded so tests
  can assert the outgoing request shape (enriched prompt, `imageConfig`, `x-goog-api-key` /
  `Authorization: Bearer`).
- **`normalize.py`** — `normalize(...)`. Canonicalizes nondeterministic fields per
  SPEC-JSON-005: `elapsed_ms` -> `<ELAPSED_MS>`, timestamped auto-names
  `naba-<cmd>-YYYYMMDD-HHMMSS[-N].<ext>` -> `naba-<cmd>-<TIMESTAMP>.<ext>`, and
  version/commit/date in both the `version` and `doctor` formats. An optional
  `replacements` map stabilizes case-specific literals (e.g. a temp CWD) first.

## Canned image

`assets/canned.png` is a minimal valid 1x1 PNG, returned base64-encoded in each provider's
response shape.

## MCP conformance (`test_mcp.py`)

`test_mcp.py` is a self-contained MCP-protocol harness (Issue 1.4) that drives
`$NABA_BIN mcp` as a **stdio MCP server** through the official MCP Python SDK (`mcp`,
pinned in `pyproject.toml`) and validates the tool + resource surface against SPEC §11
(SPEC-MCP-001..013). It launches the server as a subprocess with a hermetic env (temp
`NABA_OUTPUT_DIR`, scrubbed provider knobs, a mock-provider base URL for the tools that
call Gemini), does the `initialize` handshake, and exercises `tools/list`, `tools/call`,
`list_images`, and the resource surface. It **skips gracefully** (pytest skip) if the
`mcp` SDK import fails.

```bash
# From the repo root (the invocation the suite is designed for):
uv run --project tests/parity pytest tests/parity/test_mcp.py

# From this directory:
cd tests/parity
uv run pytest test_mcp.py

# Against the Rust build (once it exists):
NABA_BIN=/path/to/target/release/naba uv run pytest tests/parity/test_mcp.py
```

### Tool-schema golden

`tools/list` is asserted two ways: (1) explicit per-tool structural checks (inventory,
descriptions, enums, defaults, bounds, required-ness) cited to SPEC-MCP-002..011, and
(2) a normalized snapshot at **`golden/mcp/tools.json`** — the canonical `inputSchema`
of all 8 tools sorted by name, so the Rust server can be byte-diffed against the
Go-captured expectation. The `quality` param description is normalized to
`<QUALITY_DESC>` because it is a [DIVERGENCE] under multi-provider (SPEC-MCP-003) —
inventory/enums are pinned, the exact prose is not. Recapture with the same flag the
parity driver uses: `--update-golden` (or `UPDATE_GOLDEN=1`).

### What `test_mcp.py` covers

- **SPEC-MCP-001** — `initialize`: server identity `naba` + a version; tool & resource
  capabilities registered.
- **SPEC-MCP-002..011** — `tools/list`: exactly the 8 pinned tools with the pinned param
  surface (structural asserts + `golden/mcp/tools.json`).
- **SPEC-MCP-004/013** — `generate_image` happy path against the recording mock: text
  path + `Format: <mime>` note + `file://` resource link, image written under
  `NABA_OUTPUT_DIR`, enriched prompt observed on the outgoing request.
- **SPEC-MCP-005/006** — `edit_image` / `restore_image` over a temp input file.
- **SPEC-MCP-004/005/006/009/013** — validation results as **tool-level errors** (not
  process crashes): missing `prompt`, `count` out of 1..8, `steps` out of 2..8, missing
  `file`, `file not found: <path>`, and the missing-key `GEMINI_API_KEY not set` message.
- **SPEC-MCP-011** — `list_images` (MCP-only): newest-first ordering, `limit` default 20
  + clamp (`<1` → 20), the `naba-*`/extension filter, and the empty / missing-dir /
  `no output directory configured` messages.
- **SPEC-MCP-012** — `resources/templates/list`: the `file:///{path}` template metadata
  (uri / name / description / `image/*` MIME).

### `resources/read` slash-path handling (dynamic xfail)

The shipped Rust binary serves `resources/read` of a real generated path (`file:///var/.../
naba-...png`) using a slash-matching / RFC 6570 reserved-expansion template, so the test
asserts the blob (base64) + MIME by extension and **passes**. The test keeps a dynamic
`xfail` guard for any server whose template uses *simple* expansion (whose regexp does not
match `/`, rejecting the read with `resource not found`) — that path applied to the
now-retired Go build and no longer fires on the Rust binary. Separately,
`no output directory configured` is only reachable with an **empty** `HOME`, which the test
forces explicitly.

## SPEC clauses the harness already exercises

Via `test_harness.py` (smoke self-test):

- **SPEC-VERSION-001** — `naba version` output format.
- **SPEC-GEN-002** — `generate` flag inventory (`--style`) via `--help`.
- **SPEC-GEN-005** — `EnrichGeneratePrompt` join (`"an apple. Style: watercolor"`),
  asserted against the recorded outgoing request.
- **SPEC-PROVIDER-002** — Gemini endpoint (`:generateContent`), `x-goog-api-key` header,
  and `responseModalities = ["TEXT","IMAGE"]`.
- **SPEC-GLOBAL-003** — piped stdout forces `--json` (object) while a PTY stdout does not
  (human text), compared directly (piped runner vs `run_pty`).
- **SPEC-DOCTOR-004** — `doctor` reaches the `models.list` endpoint.
- **SPEC-JSON-005** — the normalizer stabilizes `elapsed_ms`, auto-name timestamps, and
  version/commit/date (both formats).
- **SPEC-GLOBAL-005** — a per-command `--preview` launches the system viewer (captured by
  the PATH stub).

The full clause-to-case mapping is the job of Issue 1.3's case table.
