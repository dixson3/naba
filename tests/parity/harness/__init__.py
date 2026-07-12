"""naba parity harness — black-box test infrastructure for the Go->Rust port.

The harness invokes the binary under test (``$NABA_BIN``) as an opaque process and
inspects only its observable behavior (stdout / stderr / exit code / files written /
outgoing HTTP requests). It imports no Go or Rust internals, so the same suite runs
unchanged against either implementation.

Public surface:

- :class:`~harness.runner.NabaRunner` / :class:`~harness.runner.RunResult` — invoke the
  binary, capture output, isolate CWD / config-dir / output-dir / provider base URLs.
- :func:`~harness.pty_runner.run_pty` — invoke the binary under a pseudo-terminal so
  TTY-dependent behavior (SPEC-GLOBAL-003) can be exercised.
- :class:`~harness.mock_provider.ProviderMock` — a recording mock for the Gemini and
  OpenRouter HTTP providers (SPEC-PROVIDER-002/004).
- :func:`~harness.normalize.normalize` — canonicalize nondeterministic fields
  (SPEC-JSON-005).
"""

from harness.mock_provider import ProviderMock
from harness.normalize import normalize, normalize_result, normalize_text
from harness.pty_runner import run_pty
from harness.runner import NabaRunner, RunResult, default_naba_bin

__all__ = [
    "NabaRunner",
    "RunResult",
    "default_naba_bin",
    "run_pty",
    "ProviderMock",
    "normalize",
    "normalize_result",
    "normalize_text",
]
