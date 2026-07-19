"""Universal ``--json`` envelope enumeration test (Issue 2.4 — SPEC-JSON-006).

This module ENUMERATES every top-level subcommand the binary advertises (parsed from
``naba --help``) and asserts that "universal ``--json`` support" is *enforced*, not
aspirational:

- Every advertised subcommand is CLASSIFIED into exactly one JSON contract:
  - **universal envelope** (``{status, data, error?}``) — ``version``, ``config``,
    ``skills``, ``provider``, ``models`` (SPEC-JSON-006);
  - **Result shape** (SPEC-JSON-001..003) — ``generate``, ``edit``, ``restore``, ``icon``,
    ``pattern``, ``diagram``, ``story``;
  - **doctor envelope** (SPEC-JSON-004) — ``doctor``;
  - **self envelope** — ``self`` (SPEC-SELF-003 refused/update JSON envelope);
  - **server** (no ``--json`` stdout contract; blocking) — ``mcp``.
  A subcommand appearing in ``--help`` that is NOT classified fails the guard, so a future
  command cannot silently ship without a documented envelope.

- The universal-envelope commands are RUN with ``--json`` (piped, so SPEC-GLOBAL-003
  auto-enables it) and asserted to emit ``{ "status": "ok", "data": … }``.

The Result-shape / doctor / self / server commands keep their own documented JSON contracts
(pinned by the golden parity cases in ``test_parity.py`` — generate*/edit*/doctor*/version*),
so this module does not re-run them; it only proves they are accounted for.

Clauses exercised here: SPEC-JSON-006, SPEC-PROVIDER-009, SPEC-PROVIDER-010, SPEC-PROVIDER-011,
SPEC-GLOBAL-003, SPEC-INV-001.
"""

from __future__ import annotations

import json
import re
from pathlib import Path

import pytest

from harness.mock_provider import ProviderMock
from harness.runner import NabaRunner

# Classification of every top-level subcommand into its documented --json contract.
UNIVERSAL_ENVELOPE = {"version", "config", "skills", "provider", "models"}
RESULT_SHAPE = {"generate", "edit", "restore", "icon", "pattern", "diagram", "story"}
DOCTOR_ENVELOPE = {"doctor"}
SELF_ENVELOPE = {"self"}
SERVER = {"mcp"}

CLASSIFIED = (
    UNIVERSAL_ENVELOPE | RESULT_SHAPE | DOCTOR_ENVELOPE | SELF_ENVELOPE | SERVER
)


def _advertised_subcommands(runner: NabaRunner) -> set[str]:
    """Parse the ``Commands:`` block of ``naba --help`` into a set of subcommand names."""
    res = runner.run(["--help"])
    text = res.stdout or res.stderr
    names: set[str] = set()
    in_commands = False
    for line in text.splitlines():
        if re.match(r"^[A-Za-z].*:$", line.strip()) and "command" in line.lower():
            in_commands = True
            continue
        if in_commands:
            # A new unindented section header ends the Commands block.
            if line and not line[0].isspace():
                break
            m = re.match(r"\s+([a-z][a-z0-9-]*)", line)
            if m:
                names.add(m.group(1))
    names.discard("help")  # clap's built-in help pseudo-command
    return names


def test_every_subcommand_is_classified(runner: NabaRunner):
    """SPEC-INV-001 / SPEC-JSON-006: every advertised subcommand has a documented envelope."""
    advertised = _advertised_subcommands(runner)
    assert advertised, "parsed no subcommands from `naba --help`"
    unclassified = advertised - CLASSIFIED
    assert not unclassified, (
        f"subcommand(s) {sorted(unclassified)} advertised by `naba --help` but not classified "
        f"into a documented --json contract (SPEC-JSON-006) — add them to a classification set."
    )
    # And every universal-envelope command is actually advertised (no stale expectation).
    missing = UNIVERSAL_ENVELOPE - advertised
    assert not missing, f"expected universal-envelope commands missing from --help: {sorted(missing)}"


def _assert_ok_envelope(stdout: str, cmd: str) -> None:
    """Assert ``stdout`` is the universal ``{status: ok, data: …}`` envelope (SPEC-JSON-006)."""
    try:
        payload = json.loads(stdout)
    except json.JSONDecodeError as exc:  # pragma: no cover - failure path
        pytest.fail(f"{cmd}: --json output is not valid JSON: {exc}\n{stdout!r}")
    assert isinstance(payload, dict), f"{cmd}: envelope must be a JSON object, got {type(payload)}"
    assert payload.get("status") == "ok", f"{cmd}: expected status 'ok', got {payload.get('status')!r}"
    assert "data" in payload, f"{cmd}: universal envelope must carry a 'data' key: {payload!r}"


def test_version_json_envelope(runner: NabaRunner):
    """SPEC-JSON-006: `version --json` (piped) emits the universal envelope."""
    res = runner.run(["version"])
    assert res.returncode == 0
    _assert_ok_envelope(res.stdout, "version")


def test_config_json_envelope(runner: NabaRunner, tmp_path: Path):
    """SPEC-JSON-006: `config set`/`config get --json` emit the universal envelope."""
    config_dir = tmp_path / "config"
    config_dir.mkdir()
    set_res = runner.run(["config", "set", "aspect", "16:9"], config_dir=config_dir)
    assert set_res.returncode == 0
    _assert_ok_envelope(set_res.stdout, "config set")
    get_res = runner.run(["config", "get", "aspect"], config_dir=config_dir)
    assert get_res.returncode == 0
    _assert_ok_envelope(get_res.stdout, "config get")


def test_skills_json_envelope(runner: NabaRunner, tmp_path: Path):
    """SPEC-JSON-006: `skills install`/`status --json` emit the universal envelope."""
    dest = tmp_path / "skills"
    dest.mkdir()
    inst = runner.run(["skills", "install", "--target", str(dest)])
    assert inst.returncode == 0, inst.stderr
    _assert_ok_envelope(inst.stdout, "skills install")
    st = runner.run(["skills", "status", "--target", str(dest)])
    assert st.returncode == 0
    _assert_ok_envelope(st.stdout, "skills status")


def test_provider_json_envelope(runner: NabaRunner, tmp_path: Path):
    """SPEC-PROVIDER-010 / SPEC-JSON-006: `provider --json` emits the universal envelope."""
    config_dir = tmp_path / "config"
    config_dir.mkdir()
    res = runner.run(["provider"], config_dir=config_dir)
    assert res.returncode == 0, res.stderr
    _assert_ok_envelope(res.stdout, "provider")


def test_models_json_envelope(runner: NabaRunner, httpserver, tmp_path: Path):
    """SPEC-PROVIDER-011 / SPEC-JSON-006: `models --json` emits the universal envelope."""
    mock = ProviderMock(httpserver)
    config_dir = tmp_path / "config"
    config_dir.mkdir()
    res = runner.run(
        ["models", "--provider", "gemini"],
        config_dir=config_dir,
        gemini_api_key="smoke-key",
        gemini_base_url=mock.gemini_base_url,
    )
    assert res.returncode == 0, res.stderr
    _assert_ok_envelope(res.stdout, "models")
    payload = json.loads(res.stdout)
    assert payload["data"]["provider"] == "gemini"
    assert payload["data"]["models"], "models list should be non-empty from the mock"
