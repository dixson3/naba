"""$NABA_BIN process runner.

Invokes the binary under test as a black box and captures stdout, stderr, and the exit
code. Per-invocation it can set an isolated working directory, ``NABA_CONFIG_DIR``,
``NABA_OUTPUT_DIR``, and the provider base-URL overrides (``GEMINI_BASE_URL`` /
``OPENROUTER_BASE_URL``) so a mock provider can intercept outgoing requests.

The runner is deliberately data-driven: a future case table (Issue 1.3) can build a
``RunResult`` from a case row by mapping row fields onto :meth:`NabaRunner.run` kwargs.
"""

from __future__ import annotations

import json
import os
import subprocess
from dataclasses import dataclass, field
from pathlib import Path
from typing import Mapping, Sequence

# Repo layout: this file is <repo>/tests/parity/harness/runner.py, so the repo root is
# three parents up. The default binary is the Go build produced by `make build`.
REPO_ROOT = Path(__file__).resolve().parents[3]


def default_naba_bin() -> str:
    """Return the binary path under test.

    ``NABA_BIN`` selects the implementation (Go build vs the future Rust build); when
    unset it defaults to the Go build at the repo root (``make build`` output).
    """
    override = os.environ.get("NABA_BIN")
    if override:
        return override
    return str(REPO_ROOT / "naba")


@dataclass
class RunResult:
    """Outcome of one binary invocation."""

    args: list[str]
    returncode: int
    stdout: str
    stderr: str
    cwd: str
    env: dict[str, str] = field(default_factory=dict)

    def json(self):
        """Parse stdout as JSON (single object or array). Raises on invalid JSON."""
        return json.loads(self.stdout)

    def __repr__(self) -> str:  # pragma: no cover - debugging aid
        return (
            f"RunResult(args={self.args!r}, returncode={self.returncode}, "
            f"stdout={self.stdout!r}, stderr={self.stderr!r})"
        )


class NabaRunner:
    """Invokes ``$NABA_BIN`` with per-case isolation."""

    def __init__(self, binary: str | None = None):
        self.binary = binary or default_naba_bin()

    def build_env(
        self,
        *,
        env: Mapping[str, str] | None = None,
        config_dir: str | os.PathLike | None = None,
        output_dir: str | os.PathLike | None = None,
        gemini_base_url: str | None = None,
        openrouter_base_url: str | None = None,
        gemini_api_key: str | None = None,
        openrouter_api_key: str | None = None,
        path_prepend: Sequence[str | os.PathLike] | None = None,
        inherit: bool = True,
    ) -> dict[str, str]:
        """Compose the child environment for an invocation.

        By default the parent environment is inherited, then the explicit overrides
        below are layered on. A stray ``GEMINI_API_KEY`` / ``OPENROUTER_API_KEY`` /
        ``GEMINI_BASE_URL`` / ``NABA_*`` from the developer's shell is scrubbed so a
        case is reproducible regardless of who runs it — set them back explicitly via
        the kwargs.
        """
        result: dict[str, str] = dict(os.environ) if inherit else {}
        # Scrub host-provided naba/provider knobs so a case is hermetic.
        for key in (
            "GEMINI_API_KEY",
            "OPENROUTER_API_KEY",
            "GEMINI_BASE_URL",
            "OPENROUTER_BASE_URL",
            "NABA_CONFIG_DIR",
            "NABA_OUTPUT_DIR",
        ):
            result.pop(key, None)

        if env:
            result.update({k: str(v) for k, v in env.items()})
        if config_dir is not None:
            result["NABA_CONFIG_DIR"] = str(config_dir)
        if output_dir is not None:
            result["NABA_OUTPUT_DIR"] = str(output_dir)
        if gemini_base_url is not None:
            result["GEMINI_BASE_URL"] = gemini_base_url
        if openrouter_base_url is not None:
            result["OPENROUTER_BASE_URL"] = openrouter_base_url
        if gemini_api_key is not None:
            result["GEMINI_API_KEY"] = gemini_api_key
        if openrouter_api_key is not None:
            result["OPENROUTER_API_KEY"] = openrouter_api_key
        if path_prepend:
            prefix = os.pathsep.join(str(p) for p in path_prepend)
            result["PATH"] = prefix + os.pathsep + result.get("PATH", "")
        return result

    def run(
        self,
        args: Sequence[str],
        *,
        cwd: str | os.PathLike | None = None,
        env: Mapping[str, str] | None = None,
        config_dir: str | os.PathLike | None = None,
        output_dir: str | os.PathLike | None = None,
        gemini_base_url: str | None = None,
        openrouter_base_url: str | None = None,
        gemini_api_key: str | None = None,
        openrouter_api_key: str | None = None,
        path_prepend: Sequence[str | os.PathLike] | None = None,
        stdin: str | bytes | None = None,
        timeout: float = 60.0,
        inherit_env: bool = True,
    ) -> RunResult:
        """Run the binary once and capture the result.

        stdin/stdout/stderr are pipes (not a TTY) — this is the "piped" mode that
        exercises SPEC-GLOBAL-003's stdout-not-a-chardevice branch (forces ``--json``).
        Use :func:`harness.pty_runner.run_pty` for the TTY branch.
        """
        run_cwd = str(cwd) if cwd is not None else os.getcwd()
        child_env = self.build_env(
            env=env,
            config_dir=config_dir,
            output_dir=output_dir,
            gemini_base_url=gemini_base_url,
            openrouter_base_url=openrouter_base_url,
            gemini_api_key=gemini_api_key,
            openrouter_api_key=openrouter_api_key,
            path_prepend=path_prepend,
            inherit=inherit_env,
        )

        stdin_bytes: bytes | None
        if stdin is None:
            stdin_bytes = None
        elif isinstance(stdin, str):
            stdin_bytes = stdin.encode()
        else:
            stdin_bytes = stdin

        argv = [self.binary, *[str(a) for a in args]]
        proc = subprocess.run(
            argv,
            cwd=run_cwd,
            env=child_env,
            input=stdin_bytes,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=timeout,
            check=False,
        )
        return RunResult(
            args=argv,
            returncode=proc.returncode,
            stdout=proc.stdout.decode(errors="replace"),
            stderr=proc.stderr.decode(errors="replace"),
            cwd=run_cwd,
            env=child_env,
        )
