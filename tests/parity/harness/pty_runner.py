"""PTY runner mode.

Runs the binary under a pseudo-terminal so its stdout (and stdin) look like a character
device. This exercises the TTY branch of SPEC-GLOBAL-003: under a PTY, stdout *is* a
chardevice, so ``--json`` is **not** force-enabled; piped (the default
:meth:`harness.runner.NabaRunner.run`) forces it on. A parity test can compare the two
modes to pin the autodetect behavior.

Because a PTY multiplexes onto a single stream, stdout and stderr are merged into
``RunResult.stdout`` (there is no separate stderr channel over one PTY). Line endings
arrive as CRLF from the terminal discipline and are normalized to ``\\n``.
"""

from __future__ import annotations

import errno
import os
import pty
import select
import subprocess
from pathlib import Path
from typing import Mapping, Sequence

from harness.runner import NabaRunner, RunResult, default_naba_bin


def run_pty(
    args: Sequence[str],
    *,
    binary: str | None = None,
    cwd: str | os.PathLike | None = None,
    env: Mapping[str, str] | None = None,
    config_dir: str | os.PathLike | None = None,
    output_dir: str | os.PathLike | None = None,
    gemini_base_url: str | None = None,
    openrouter_base_url: str | None = None,
    gemini_api_key: str | None = None,
    openrouter_api_key: str | None = None,
    path_prepend: Sequence[str | os.PathLike] | None = None,
    timeout: float = 60.0,
    inherit_env: bool = True,
) -> RunResult:
    """Run the binary with stdout+stdin+stderr attached to a PTY (a chardevice)."""
    bin_path = binary or default_naba_bin()
    runner = NabaRunner(bin_path)
    child_env = runner.build_env(
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
    run_cwd = str(cwd) if cwd is not None else os.getcwd()

    master_fd, slave_fd = pty.openpty()
    argv = [bin_path, *[str(a) for a in args]]
    try:
        proc = subprocess.Popen(
            argv,
            cwd=run_cwd,
            env=child_env,
            stdin=slave_fd,
            stdout=slave_fd,
            stderr=slave_fd,
            close_fds=True,
        )
    finally:
        os.close(slave_fd)

    chunks: list[bytes] = []
    try:
        while True:
            ready, _, _ = select.select([master_fd], [], [], timeout)
            if not ready:
                proc.kill()
                raise TimeoutError(f"PTY run timed out after {timeout}s: {argv}")
            try:
                data = os.read(master_fd, 65536)
            except OSError as exc:
                # EOF on a PTY master surfaces as EIO on some platforms (Linux).
                if exc.errno == errno.EIO:
                    break
                raise
            if not data:
                break
            chunks.append(data)
    finally:
        os.close(master_fd)

    returncode = proc.wait(timeout=timeout)
    text = b"".join(chunks).decode(errors="replace").replace("\r\n", "\n")
    return RunResult(
        args=argv,
        returncode=returncode,
        stdout=text,
        stderr="",  # merged into stdout over the single PTY stream
        cwd=run_cwd,
        env=child_env,
    )
