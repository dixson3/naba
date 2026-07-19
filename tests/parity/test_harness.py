"""Smoke self-test for the parity harness.

Proves the harness plumbing works against a real binary before the case table (Issue 1.3)
and goldens (Issue 1.4) are authored. These are infrastructure tests, not parity cases.
"""

from __future__ import annotations

import json
import re
import time
from pathlib import Path

from harness import normalize, run_pty

# --- runner plumbing --------------------------------------------------------------


def test_version_through_runner(runner):
    """Human line (PTY) matches SPEC-VERSION-001; piped emits the SPEC-JSON-006 envelope."""
    # PTY: --json is NOT force-enabled on a chardevice, so the human line prints.
    human = run_pty(["version"], binary=runner.binary)
    assert human.returncode == 0, human.stdout
    # naba <Version> (commit: <Commit>, built: <Date>)
    assert re.match(
        r"^naba \S+ \(commit: \S+, built: \S+\)$", human.stdout.strip()
    ), human.stdout
    # Piped: the universal --json envelope carries the same line under data.line.
    piped = runner.run(["version"])
    assert piped.returncode == 0, piped.stderr
    payload = json.loads(piped.stdout)
    assert payload["status"] == "ok"
    assert re.match(
        r"^naba \S+ \(commit: \S+, built: \S+\)$", payload["data"]["line"].strip()
    ), payload


def test_generate_help_through_runner(runner):
    """`generate --help` renders and lists a known flag (SPEC-GEN-002)."""
    result = runner.run(["generate", "--help"])
    assert result.returncode == 0, result.stderr
    assert "generate" in result.stdout
    assert "--style" in result.stdout


# --- Gemini mock wiring, end-to-end -----------------------------------------------


def test_generate_hits_gemini_mock(runner, provider_mock, work_cwd):
    """A `generate` call routed at the mock records the enriched prompt + auth header.

    Exercises SPEC-PROVIDER-002 (endpoint/headers), SPEC-GEN-005 (prompt enrichment),
    and SPEC-GLOBAL-003 (piped stdout forces --json).
    """
    result = runner.run(
        ["generate", "an apple", "--style", "watercolor"],
        cwd=work_cwd,
        config_dir=work_cwd / "cfg",
        gemini_base_url=provider_mock.gemini_base_url,
        gemini_api_key="smoke-key",
    )
    assert result.returncode == 0, f"stderr={result.stderr!r} stdout={result.stdout!r}"

    # A request reached the mock, on the generateContent endpoint.
    generate_reqs = provider_mock.generate_requests()
    assert len(generate_reqs) == 1, provider_mock.requests
    req = generate_reqs[0]
    assert req.method == "POST"
    assert req.path.endswith(":generateContent")

    # Auth header carried the key (SPEC-PROVIDER-002).
    assert req.headers.get("x-goog-api-key") == "smoke-key"

    # Enriched prompt is exactly the SPEC-GEN-005 join.
    assert provider_mock.last_prompt() == "an apple. Style: watercolor"

    # responseModalities is always [TEXT, IMAGE] (SPEC-PROVIDER-002).
    assert req.json["generationConfig"]["responseModalities"] == ["TEXT", "IMAGE"]

    # Piped stdout forced --json: stdout parses as a Result object with a written path.
    payload = result.json()
    assert payload["command"] == "generate"
    assert payload["prompt"] == "an apple"
    assert Path(payload["path"]).exists()


def test_gemini_list_models_mock(runner, provider_mock, work_cwd):
    """`doctor` reaches the models.list endpoint (SPEC-DOCTOR-004 / list_models)."""
    result = runner.run(
        ["doctor"],
        cwd=work_cwd,
        config_dir=work_cwd / "cfg",
        gemini_base_url=provider_mock.gemini_base_url,
        gemini_api_key="smoke-key",
    )
    # doctor exits 0 (all pass) or 1 (some check failed) depending on environment;
    # either way the harness captured a real exit code. The point is the mock was hit.
    assert result.returncode in (0, 1), result.stderr
    list_calls = [r for r in provider_mock.gemini_requests() if r.method == "GET"]
    assert list_calls, "expected doctor to call the models.list endpoint"
    assert list_calls[0].path == "/models"


# --- normalizer -------------------------------------------------------------------


def test_normalizer_stabilizes_nondeterministic_fields():
    result = {
        "path": "/tmp/x/naba-generate-20260711-105300-2.png",
        "command": "generate",
        "elapsed_ms": 1234,
        "params": {"note": "naba-icon-20260101-000000.jpg"},
    }
    out = normalize(result)
    assert out["elapsed_ms"] == "<ELAPSED_MS>"
    assert out["path"] == "/tmp/x/naba-generate-<TIMESTAMP>.png"
    assert out["params"]["note"] == "naba-icon-<TIMESTAMP>.jpg"


def test_normalizer_version_both_formats():
    v = normalize("naba v0.5.0-2-g9b2aa0b (commit: 9b2aa0b, built: 2026-07-11T17:53:43Z)")
    assert v == "naba <VERSION> (commit: <COMMIT>, built: <DATE>)"
    # doctor variant (no colons) — SPEC-VERSION-002
    d = normalize("naba v0.5.0 (commit 9b2aa0b, built 2026-07-11T17:53:43Z)")
    assert d == "naba <VERSION> (commit <COMMIT>, built <DATE>)"


def test_normalizer_replacements_stabilize_case_paths():
    out = normalize(
        {"path": "/private/tmp/case1/naba-edit-20260101-000000.jpg"},
        replacements={"/private/tmp/case1": "<CWD>"},
    )
    assert out["path"] == "<CWD>/naba-edit-<TIMESTAMP>.jpg"


# --- PTY vs piped (SPEC-GLOBAL-003) -----------------------------------------------


def test_pty_vs_piped_json_autodetect(runner, provider_mock, work_cwd):
    """Piped stdout forces --json (object); a PTY stdout does not (human text)."""
    common = dict(
        cwd=work_cwd,
        config_dir=work_cwd / "cfg",
        gemini_base_url=provider_mock.gemini_base_url,
        gemini_api_key="smoke-key",
    )
    piped = runner.run(["generate", "an apple"], **common)
    assert piped.returncode == 0, piped.stderr
    # Forced JSON: stdout is a parseable Result object.
    json.loads(piped.stdout)

    pty_res = run_pty(
        ["generate", "an apple"],
        binary=runner.binary,
        cwd=work_cwd,
        config_dir=work_cwd / "cfg2",
        gemini_base_url=provider_mock.gemini_base_url,
        gemini_api_key="smoke-key",
    )
    assert pty_res.returncode == 0, pty_res.stdout
    # Under a PTY, --json is NOT forced: stdout is human text ("Saved: ..."), not JSON.
    stripped = pty_res.stdout.strip()
    assert stripped, "expected human output under PTY"
    try:
        json.loads(stripped)
        is_json = True
    except json.JSONDecodeError:
        is_json = False
    assert not is_json, f"PTY stdout should be human text, got JSON: {stripped!r}"


# --- preview PATH-stub (SPEC-GLOBAL-005 / --preview) ------------------------------


def test_preview_stub_records_invocation(runner, provider_mock, work_cwd, preview_stub):
    """`--preview` invokes the faked system viewer, which records instead of launching."""
    result = runner.run(
        ["generate", "an apple", "--preview"],
        cwd=work_cwd,
        config_dir=work_cwd / "cfg",
        gemini_base_url=provider_mock.gemini_base_url,
        gemini_api_key="smoke-key",
        env=preview_stub.env,
        path_prepend=[preview_stub.dir],
    )
    assert result.returncode == 0, result.stderr
    # The opener is launched detached (Go `exec.Command(...).Start()`), so the recording
    # write can land just after naba exits — poll briefly.
    deadline = time.monotonic() + 5.0
    calls: list[str] = []
    while time.monotonic() < deadline:
        calls = preview_stub.calls()
        if calls:
            break
        time.sleep(0.05)
    assert calls, "expected the preview stub to record a viewer invocation"
    # The recorded line names the faked opener and the generated file path.
    assert "naba-generate-" in calls[0]
