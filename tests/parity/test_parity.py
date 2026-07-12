"""Data-driven parity driver (Issue 1.3).

Loads every ``cases/*.yaml`` case row, replays it through the black-box harness against
the recording mock provider, normalizes the observable output (SPEC-JSON-005), and
compares it to a captured golden under ``golden/<case-id>/``.

The identical suite runs against the Go build (default) and the future Rust build; the
binary is selected by ``$NABA_BIN`` (see ``harness.runner``). Cases that require a
port-only capability (``--provider`` / OpenRouter / per-provider quality) are marked
``requires: [provider]`` and are **skipped** when the binary under test lacks that
capability (the Go binary), and become active for the Rust binary.

Capturing goldens
-----------------
Run with ``--update-golden`` (pytest flag) or ``UPDATE_GOLDEN=1`` (env) to (re)write the
goldens from the current ``$NABA_BIN`` instead of comparing. The exit-code assertion still
runs while updating, so a case whose declared ``exit_code`` is wrong fails loudly even in
update mode.

Case schema
-----------
See ``cases/README.md`` for the full field reference.
"""

from __future__ import annotations

import json
import os
import re
import shutil
import stat as stat_mod
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

import pytest
import yaml

from harness import normalize
from harness.mock_provider import ProviderMock
from harness.pty_runner import run_pty
from harness.runner import NabaRunner

CASES_DIR = Path(__file__).resolve().parent / "cases"
GOLDEN_DIR = Path(__file__).resolve().parent / "golden"
ASSETS_DIR = Path(__file__).resolve().parent / "assets"

# Header names worth pinning on an outgoing request (auth + content type). Everything
# else (host, content-length, user-agent, accept-encoding, ...) is transport noise.
_KEEP_HEADERS = ("x-goog-api-key", "authorization", "content-type")


# --------------------------------------------------------------------------------------
# --update-golden wiring (the pytest_addoption hook lives in conftest.py)
# --------------------------------------------------------------------------------------
def _update_golden(request) -> bool:
    if request.config.getoption("--update-golden"):
        return True
    return os.environ.get("UPDATE_GOLDEN", "") not in ("", "0", "false", "False")


# --------------------------------------------------------------------------------------
# Case model + loader
# --------------------------------------------------------------------------------------
@dataclass
class Case:
    id: str
    file: str
    spec: list[str] = field(default_factory=list)
    argv: list[str] = field(default_factory=list)
    pre_argv: list[list[str]] = field(default_factory=list)
    env: dict[str, str] = field(default_factory=dict)
    config: dict[str, Any] | None = None
    gemini_key: str | None = None
    openrouter_key: str | None = None
    provider_mock: bool = True
    mock_status: int | None = None
    mock_message: str = "boom"
    inputs: list[dict[str, str]] = field(default_factory=list)
    stdin: str | None = None
    mode: str = "piped"  # piped | pty
    preview: bool = False
    requires: list[str] = field(default_factory=list)
    exit_code: int = 0
    # Which streams to snapshot: any subset of {stdout, stderr}. "none" snapshots neither.
    golden: str = "streams"
    request: dict[str, Any] | None = None
    stdout_contains: list[str] = field(default_factory=list)
    stderr_contains: list[str] = field(default_factory=list)
    skills_dest: bool = False  # allocate a temp skills destination + <SKILLS_DEST> token
    # After pre_argv, append text to these <SKILLS_DEST>-relative files (to force a
    # "modified since install" state so the tree-hash detects tampering; SPEC-EMBED-002/3).
    tamper: list[str] = field(default_factory=list)

    @property
    def golden_streams(self) -> set[str]:
        if self.golden in ("none", None, ""):
            return set()
        if self.golden == "streams":
            return {"stdout", "stderr"}
        return {s.strip() for s in self.golden.split("+") if s.strip()}


def load_cases() -> list[Case]:
    cases: list[Case] = []
    seen: set[str] = set()
    for path in sorted(CASES_DIR.glob("*.yaml")):
        doc = yaml.safe_load(path.read_text())
        if not doc:
            continue
        for row in doc.get("cases", []):
            cid = row["id"]
            if cid in seen:
                raise ValueError(f"duplicate case id {cid!r} (in {path.name})")
            seen.add(cid)
            known = {f for f in Case.__dataclass_fields__ if f not in ("id", "file")}
            unknown = set(row) - known - {"id"}
            if unknown:
                raise ValueError(f"case {cid!r}: unknown field(s) {sorted(unknown)}")
            cases.append(Case(id=cid, file=path.name, **{k: row[k] for k in row if k != "id"}))
    return cases


ALL_CASES = load_cases()


# --------------------------------------------------------------------------------------
# Binary capability probe (Go vs Rust): does the binary expose --provider?
# --------------------------------------------------------------------------------------
def _binary_capabilities(runner: NabaRunner) -> set[str]:
    caps: set[str] = set()
    res = runner.run(["generate", "--help"])
    if "--provider" in res.stdout or "--provider" in res.stderr:
        caps.add("provider")
    return caps


# --------------------------------------------------------------------------------------
# Golden IO
# --------------------------------------------------------------------------------------
def _golden_path(case: Case, name: str) -> Path:
    return GOLDEN_DIR / case.id / name


def _read_or_update(path: Path, actual: str, *, update: bool) -> None:
    if update:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(actual)
        return
    if not path.exists():
        pytest.fail(
            f"missing golden {path} -- run with --update-golden to capture it first"
        )
    expected = path.read_text()
    assert actual == expected, (
        f"golden mismatch for {path.name}\n--- expected ---\n{expected}\n"
        f"--- actual ---\n{actual}"
    )


# --------------------------------------------------------------------------------------
# Request canonicalization
# --------------------------------------------------------------------------------------
# Keys carrying a base64 image blob in an OUTGOING request. Redacted so the golden pins
# the request STRUCTURE (prompt, imageConfig, headers) without a giant, brittle base64
# literal. The prompt text is never redacted -- it is the load-bearing assertion.
_BLOB_KEYS = {"data", "b64_json"}


def _redact_blobs(value: Any) -> Any:
    """Redact base64 image blobs (by key name) so a request golden stays reviewable."""
    if isinstance(value, dict):
        return {
            k: ("<IMAGE_DATA>" if k in _BLOB_KEYS and isinstance(v, str) else _redact_blobs(v))
            for k, v in value.items()
        }
    if isinstance(value, list):
        return [_redact_blobs(v) for v in value]
    return value


def _canonical_requests(mock: ProviderMock) -> list[dict[str, Any]]:
    out: list[dict[str, Any]] = []
    for r in mock.requests:
        if r.method != "POST":  # skip the models.list GET
            continue
        body = _redact_blobs(r.json) if r.json is not None else r.body_text
        headers = {k: r.headers[k] for k in _KEEP_HEADERS if k in r.headers}
        out.append(
            {
                "provider": r.provider,
                "method": r.method,
                "path": r.path,
                "headers": headers,
                "body": body,
            }
        )
    return out


# --------------------------------------------------------------------------------------
# The parametrized driver
# --------------------------------------------------------------------------------------
@pytest.fixture(scope="session")
def caps(naba_bin: str) -> set[str]:
    return _binary_capabilities(NabaRunner(naba_bin))


def _copy_input(spec: dict[str, str], cwd: Path) -> None:
    name = spec["name"]
    dest = cwd / name
    dest.parent.mkdir(parents=True, exist_ok=True)
    if "asset" in spec:
        shutil.copyfile(ASSETS_DIR / spec["asset"], dest)
    else:
        dest.write_text(spec.get("content", ""))


def _preview_stub(tmp_path: Path) -> tuple[Path, dict[str, str], Path]:
    bindir = tmp_path / "previewbin"
    bindir.mkdir()
    log_path = tmp_path / "preview-calls.log"
    script = (
        "#!/bin/sh\n"
        'printf "%s %s\\n" "$(basename "$0")" "$*" >> "$NABA_PREVIEW_LOG"\n'
        "exit 0\n"
    )
    for name in ("open", "xdg-open", "start"):
        p = bindir / name
        p.write_text(script)
        p.chmod(p.stat().st_mode | stat_mod.S_IXUSR | stat_mod.S_IXGRP | stat_mod.S_IXOTH)
    return bindir, {"NABA_PREVIEW_LOG": str(log_path)}, log_path


@pytest.mark.parametrize("case", ALL_CASES, ids=[c.id for c in ALL_CASES])
def test_parity(case: Case, request, runner, caps, httpserver, tmp_path):
    # Capability gate: skip port-only cases on a binary that lacks the capability.
    missing = set(case.requires) - caps
    if missing:
        pytest.skip(f"binary lacks capability {sorted(missing)} (Go binary); rust-only case")

    update = _update_golden(request)

    # --- isolation dirs -------------------------------------------------------------
    cwd = tmp_path / "cwd"
    cwd.mkdir()
    config_dir = tmp_path / "config"
    config_dir.mkdir()
    output_dir = tmp_path / "output"
    output_dir.mkdir()

    replacements: dict[str, str] = {}

    def add_repl(p: Path, token: str) -> None:
        for variant in {str(p), os.path.realpath(str(p))}:
            replacements[variant] = token

    add_repl(cwd, "<CWD>")
    add_repl(config_dir, "<CONFIG_DIR>")
    add_repl(output_dir, "<OUTPUT_DIR>")

    skills_dest: Path | None = None
    if case.skills_dest:
        skills_dest = tmp_path / "skills_dest"
        skills_dest.mkdir()
        add_repl(skills_dest, "<SKILLS_DEST>")

    # Substitute a {SKILLS_DEST} placeholder inside argv/pre_argv with the real path.
    def subst(tokens: list[str]) -> list[str]:
        if skills_dest is None:
            return list(tokens)
        return [t.replace("{SKILLS_DEST}", str(skills_dest)) for t in tokens]

    # --- config file ----------------------------------------------------------------
    if case.config:
        (config_dir / "config.yaml").write_text(yaml.safe_dump(case.config, sort_keys=False))

    # --- input files ----------------------------------------------------------------
    for spec in case.inputs:
        _copy_input(spec, cwd)

    # --- provider mock / error injection --------------------------------------------
    mock: ProviderMock | None = None
    gemini_base = openrouter_base = None
    root = f"http://{httpserver.host}:{httpserver.port}"
    add_repl(Path(root), "<MOCK>")  # harmless; root has no realpath meaning
    replacements[root] = "<MOCK>"
    if case.mock_status is not None:
        errbody = json.dumps(
            {"error": {"code": case.mock_status, "message": case.mock_message,
                       "status": "ERROR"}}
        )
        httpserver.expect_request(
            re.compile(r"^/models/[^/]+:generateContent$"), method="POST"
        ).respond_with_data(errbody, status=case.mock_status,
                            content_type="application/json")
        gemini_base = root
        openrouter_base = f"{root}/api/v1"
    elif case.provider_mock:
        mock = ProviderMock(httpserver)
        gemini_base = mock.gemini_base_url
        openrouter_base = mock.openrouter_base_url

    # --- preview stub ---------------------------------------------------------------
    env = dict(case.env)
    path_prepend = None
    preview_log: Path | None = None
    if case.preview:
        bindir, penv, preview_log = _preview_stub(tmp_path)
        env.update(penv)
        path_prepend = [bindir]

    common = dict(
        cwd=cwd,
        env=env or None,
        config_dir=config_dir,
        output_dir=output_dir,
        gemini_base_url=gemini_base,
        openrouter_base_url=openrouter_base,
        gemini_api_key=case.gemini_key,
        openrouter_api_key=case.openrouter_key,
        path_prepend=path_prepend,
    )

    # --- pre-steps (setup invocations, output ignored) ------------------------------
    for pre in case.pre_argv:
        runner.run(subst(pre), **common)

    # Tamper with installed files (after setup) to exercise modification detection.
    if case.tamper and skills_dest is not None:
        for rel in case.tamper:
            target = skills_dest / rel
            with target.open("a") as fh:
                fh.write("\n<!-- tampered -->\n")

    # A recording mock may have captured pre-step requests; drop them so the request
    # golden reflects only the case's own argv.
    if mock is not None:
        mock.requests.clear()

    # --- main invocation ------------------------------------------------------------
    argv = subst(case.argv)
    if case.mode == "pty":
        result = run_pty(argv, binary=runner.binary, **common)
    else:
        result = runner.run(argv, stdin=case.stdin, **common)

    # --- exit-code assertion (always, even while updating) --------------------------
    assert result.returncode == case.exit_code, (
        f"exit code: expected {case.exit_code}, got {result.returncode}\n"
        f"stdout={result.stdout!r}\nstderr={result.stderr!r}"
    )

    # --- substring assertions -------------------------------------------------------
    for needle in case.stdout_contains:
        assert needle in result.stdout, f"expected {needle!r} in stdout: {result.stdout!r}"
    for needle in case.stderr_contains:
        assert needle in result.stderr, f"expected {needle!r} in stderr: {result.stderr!r}"

    # --- stream goldens -------------------------------------------------------------
    streams = case.golden_streams
    if "stdout" in streams:
        norm = normalize(result.stdout, replacements=replacements)
        _read_or_update(_golden_path(case, "stdout.txt"), norm, update=update)
    if "stderr" in streams and case.mode != "pty":
        norm = normalize(result.stderr, replacements=replacements)
        _read_or_update(_golden_path(case, "stderr.txt"), norm, update=update)

    # --- request golden -------------------------------------------------------------
    if case.request is not None and mock is not None:
        canon = _canonical_requests(mock)
        canon = normalize(canon, replacements=replacements)
        rendered = json.dumps(canon, indent=2, sort_keys=True) + "\n"
        _read_or_update(_golden_path(case, "request.json"), rendered, update=update)

    # --- preview assertion ----------------------------------------------------------
    if case.preview:
        import time as _t

        deadline = _t.monotonic() + 5.0
        calls: list[str] = []
        while _t.monotonic() < deadline:
            if preview_log and preview_log.exists():
                calls = [ln for ln in preview_log.read_text().splitlines() if ln.strip()]
            if calls:
                break
            _t.sleep(0.05)
        assert calls, "expected the preview stub to record a viewer invocation"


def test_case_table_is_nonempty():
    """Guard: the driver actually discovered cases (a glob typo would silently pass)."""
    assert ALL_CASES, "no cases discovered under cases/*.yaml"
