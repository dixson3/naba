"""pytest fixtures for the naba parity harness.

Fixture inventory (see ``README.md`` for details):

- ``naba_bin``       (session) -> path to the binary under test (``$NABA_BIN`` or Go build).
- ``runner``         (function) -> a :class:`harness.runner.NabaRunner` bound to ``naba_bin``.
- ``config_dir``     (function) -> an isolated temp ``NABA_CONFIG_DIR``.
- ``output_dir``     (function) -> an isolated temp ``NABA_OUTPUT_DIR``.
- ``work_cwd``       (function) -> an isolated temp CWD for auto-named output files.
- ``provider_mock``  (function) -> a recording :class:`harness.mock_provider.ProviderMock`
                                    over the ``pytest-httpserver`` ``httpserver`` fixture.
- ``preview_stub``   (function) -> a PATH-stub that captures ``open``/``xdg-open``/``start``
                                    invocations instead of launching a viewer.
"""

from __future__ import annotations

import os
import stat
from dataclasses import dataclass
from pathlib import Path

import pytest

from harness.mock_provider import ProviderMock
from harness.runner import NabaRunner, default_naba_bin


def pytest_addoption(parser):
    """`--update-golden`: (re)capture goldens from $NABA_BIN instead of comparing.

    Consumed by the data-driven parity driver (test_parity.py). Also honored via the
    UPDATE_GOLDEN=1 environment variable.
    """
    parser.addoption(
        "--update-golden",
        action="store_true",
        default=False,
        help="Capture/overwrite goldens from the current $NABA_BIN instead of comparing.",
    )


@pytest.fixture(scope="session")
def naba_bin() -> str:
    """Path to the binary under test. Fails fast if it is missing."""
    path = default_naba_bin()
    if not os.path.exists(path):
        pytest.fail(
            f"naba binary not found at {path!r}. Build it with `make build` (Go) or set "
            f"NABA_BIN to the binary you want to test."
        )
    return path


@pytest.fixture
def runner(naba_bin: str) -> NabaRunner:
    return NabaRunner(naba_bin)


@pytest.fixture
def config_dir(tmp_path: Path) -> Path:
    d = tmp_path / "config"
    d.mkdir()
    return d


@pytest.fixture
def output_dir(tmp_path: Path) -> Path:
    d = tmp_path / "output"
    d.mkdir()
    return d


@pytest.fixture
def work_cwd(tmp_path: Path) -> Path:
    """A clean temp working directory for CLI auto-named output (SPEC-CFGSCHEMA-005)."""
    d = tmp_path / "cwd"
    d.mkdir()
    return d


@pytest.fixture
def provider_mock(httpserver) -> ProviderMock:
    """A recording Gemini+OpenRouter mock over pytest-httpserver's ``httpserver``."""
    return ProviderMock(httpserver)


@dataclass
class PreviewStub:
    """A temp PATH dir whose fake viewer scripts record their invocations.

    Prepend ``dir`` to ``PATH`` (pass ``path_prepend=[stub.dir]`` to the runner) and set
    the ``env`` mapping so the scripts know where to log. ``calls()`` returns the recorded
    invocation lines (one per launch the binary attempted).
    """

    dir: Path
    log_path: Path
    env: dict[str, str]

    def calls(self) -> list[str]:
        if not self.log_path.exists():
            return []
        return [
            line for line in self.log_path.read_text().splitlines() if line.strip()
        ]


@pytest.fixture
def preview_stub(tmp_path: Path) -> PreviewStub:
    """Fake ``open``/``xdg-open``/``start`` on ``PATH`` that record instead of launching.

    Used by ``--preview`` tests (SPEC-GLOBAL-005 / per-command ``--preview``) so a preview
    call is observable but never pops a window.
    """
    bindir = tmp_path / "previewbin"
    bindir.mkdir()
    log_path = tmp_path / "preview-calls.log"
    script = (
        "#!/bin/sh\n"
        '# Fake system viewer: record the invocation, launch nothing.\n'
        'printf "%s %s\\n" "$(basename "$0")" "$*" >> "$NABA_PREVIEW_LOG"\n'
        "exit 0\n"
    )
    for name in ("open", "xdg-open", "start"):
        p = bindir / name
        p.write_text(script)
        p.chmod(p.stat().st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
    return PreviewStub(
        dir=bindir,
        log_path=log_path,
        env={"NABA_PREVIEW_LOG": str(log_path)},
    )
