# Parity case table (`cases/*.yaml`)

Data-driven black-box parity cases for the naba **Go -> Rust port** (Issue 1.3). Each row
is replayed by `../test_parity.py` against the binary under test (`$NABA_BIN`), the
observable output is normalized (SPEC-JSON-005), and compared to a captured golden under
`../golden/<case-id>/`. The same table runs unchanged against the Go build and the future
Rust build.

One YAML file per command group (`generate.yaml`, `edit.yaml`, ...) plus cross-cutting
files: `exit_codes.yaml`, `precedence.yaml`, `json_shapes.yaml`, `inventory.yaml`,
`mcp.yaml`. Every file is a mapping with a single `cases:` list.

## Capturing / updating goldens

```bash
# from tests/parity/ -- point at the Go build (absolute path; the runner changes cwd):
NABA_BIN="$(cd ../.. && pwd)/naba" uv run pytest test_parity.py --update-golden
# then confirm the binary passes its own freshly-captured goldens:
NABA_BIN="$(cd ../.. && pwd)/naba" uv run pytest test_parity.py
```

`--update-golden` (or `UPDATE_GOLDEN=1`) writes goldens instead of comparing. The
`exit_code` assertion still runs while updating, so a wrong expected code fails loudly.

## Case schema

| Field | Type | Default | Meaning |
|:--|:--|:--|:--|
| `id` | str (required) | — | Unique case id; also the `golden/<id>/` directory name. |
| `spec` | list[str] | `[]` | SPEC clause ids this case covers (traceability). |
| `argv` | list[str] | `[]` | Arguments passed to the binary (after the exe name). |
| `pre_argv` | list[list[str]] | `[]` | Setup invocations run first, same isolation; output ignored (e.g. `skills install`, `config set`). |
| `env` | map | `{}` | Extra environment variables. |
| `config` | map | none | Written verbatim as `config.yaml` in the temp `NABA_CONFIG_DIR`. |
| `gemini_key` | str | none | `GEMINI_API_KEY` value (omit to test the missing-key path). |
| `openrouter_key` | str | none | `OPENROUTER_API_KEY` value (port-only). |
| `provider_mock` | bool | `true` | Wire `GEMINI_BASE_URL`/`OPENROUTER_BASE_URL` at the recording mock. Set `false` for commands that make no HTTP call (config/skills/version/parse-errors). |
| `mock_status` | int | none | Inject an HTTP error status on the Gemini generate endpoint (401/429/500 ...) to pin the HTTP->exit mapping (SPEC-EXIT-004). |
| `mock_message` | str | `boom` | The `error.message` body returned with `mock_status`. |
| `inputs` | list | `[]` | Files created in CWD before the run. Each `{name, asset}` (copy from `../assets/`) or `{name, content}`. |
| `stdin` | str | none | Data piped to stdin. |
| `mode` | `piped`\|`pty` | `piped` | `pty` runs under a pseudo-terminal (stdout is a chardevice -> `--json` NOT forced; stderr merges into stdout) — SPEC-GLOBAL-003. |
| `preview` | bool | `false` | Install a PATH-stub viewer and assert `--preview` invoked it (SPEC-GLOBAL-005). |
| `skills_dest` | bool | `false` | Allocate a temp skills destination; `{SKILLS_DEST}` in `argv`/`pre_argv` expands to it, and the path is tokenized to `<SKILLS_DEST>` in goldens. |
| `tamper` | list[str] | `[]` | After `pre_argv`, append to these `<SKILLS_DEST>`-relative files to force a "modified since install" state (SPEC-EMBED-002/003). |
| `requires` | list[str] | `[]` | Capability tags. `provider` => needs the port-only `--provider` flag; the case is **skipped** on a binary lacking it (the Go binary), **active** for Rust. |
| `exit_code` | int | `0` | Asserted process exit code (always checked, even while updating goldens). |
| `golden` | str | `streams` | Which streams to snapshot: `streams` (stdout+stderr), `stdout`, `stderr`, `stdout+stderr`, or `none`. |
| `request` | map | none | Present (even `{}`) => snapshot the outgoing POST request(s) recorded by the mock to `golden/<id>/request.json`. |
| `stdout_contains` / `stderr_contains` | list[str] | `[]` | Substring assertions (used where a full byte-golden would be help-format-divergent, e.g. `mcp --help`). |

## Determinism

Every case is hermetic: a temp CWD, temp `NABA_CONFIG_DIR`, temp `NABA_OUTPUT_DIR`, the
provider mock (or an injected error), and — for `--preview` — a PATH-stubbed viewer. Host
`GEMINI_*`/`OPENROUTER_*`/`NABA_*` env is scrubbed by the runner. Before comparison the
driver tokenizes the per-case temp paths (`<CWD>`, `<CONFIG_DIR>`, `<OUTPUT_DIR>`,
`<SKILLS_DEST>`, `<MOCK>`) and applies `harness/normalize.py` (`elapsed_ms`, timestamped
auto-names, version/commit/date). In request goldens the base64 image blob
(`inlineData.data` / `b64_json`) is redacted to `<IMAGE_DATA>`; the prompt text is never
redacted — it is the load-bearing assertion.

## Go vs Rust; port-only cases

The driver probes the binary once (`generate --help` mentions `--provider`?) to detect
capabilities. Cases tagged `requires: [provider]` (OpenRouter, per-provider quality) are
skipped on the Go binary and become active for the Rust binary. `mcp.yaml` is a CLI-level
`--help` smoke only — the MCP stdio-protocol harness is Issue 1.4.
